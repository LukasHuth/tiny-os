#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(ptr_internals)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_ops)]
use core::panic::PanicInfo;

use multiboot2::{BootInformation, BootInformationHeader};

extern crate linked_list_allocator;
extern crate multiboot2;

use linked_list_allocator::LockedHeap;

use crate::memory::FrameAllocator as _;

const DEBUG_PRINTS: bool = true;

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
    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("Elf-sections tag required");

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

    memory::remap_the_kernel(&mut frame_allocator, &boot_info);
    frame_allocator.allocate_frame(); // new: try to allocate a frame
    println!("It did not crash!");

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
