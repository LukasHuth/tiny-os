#![no_std]
#![no_main]
#![feature(ptr_internals)]
#![feature(const_trait_impl)]
#![feature(const_ops)]
use core::panic::PanicInfo;

use multiboot2::{BootInformation, BootInformationHeader, MemoryAreaType};

extern crate linked_list_allocator;
extern crate multiboot2;

use linked_list_allocator::LockedHeap;

const DEBUG_PRINTS: bool = true;

use crate::memory::FrameAllocator as _;

// Define a global allocator
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Later, once you know your heap region (from the memory map):
pub fn init_heap(start: *mut u8, size: usize) {
    unsafe {
        ALLOCATOR.lock().init(start, size);
    }
}

#[macro_use]
mod vga_buffer;
mod memory;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) -> ! {
    vga_buffer::clear_screen();
    if DEBUG_PRINTS {
        println!("Hello World{}", "!");
    }

    let boot_info = unsafe {
        BootInformation::load(multiboot_information_address as *const BootInformationHeader)
            .expect("Invalid multiboot format")
    };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let areas = memory_map_tag.memory_areas();

    if DEBUG_PRINTS {
        println!("memory areas: {:?}", areas);
        for area in memory_map_tag.memory_areas() {
            if area.typ() == MemoryAreaType::Available {
                println!(
                    "    start: 0x{:x}, length: 0x{:x}",
                    area.start_address(),
                    area.size()
                );
            }
        }
    }
    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("Elf-sections tag required");

    if DEBUG_PRINTS {
        println!("kernel sections:");
        for section in elf_sections_tag.sections() {
            println!(
                "    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
                section.start_address(),
                section.size(),
                section.flags()
            );
        }
    }

    let kernel_start = elf_sections_tag
        .sections()
        .map(|s| s.start_address())
        .min()
        .unwrap();
    let kernel_end = elf_sections_tag
        .sections()
        .map(|s| s.start_address() + s.size())
        .max()
        .unwrap();

    let multiboot_start = multiboot_information_address;
    let multiboot_end = multiboot_start + (boot_info.total_size() as usize);

    if DEBUG_PRINTS {
        println!(
            "kernel_start: {:x} kernel_end: {:x}",
            kernel_start, kernel_end
        );
        println!(
            "multiboot_start: {:x} multiboot_end: {:x}",
            multiboot_start, multiboot_end
        );
        println!("memory_area count: {}", memory_map_tag.memory_areas().len());
    }
    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        multiboot_start,
        multiboot_end,
        memory_map_tag.memory_areas(),
    );
    // for i in 1.. {
    //     let frame = frame_allocator.allocate_frame();
    //     if let None = frame {
    //         if DEBUG_PRINTS {
    //             println!("allocated {} frames", i);
    //         }
    //         break;
    //     }
    // }

    memory::test_paging(&mut frame_allocator);

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!("PANIC in {} at line {}", location.file(), location.line());
    }
    println!("    {}", info.message());
    loop {}
}
