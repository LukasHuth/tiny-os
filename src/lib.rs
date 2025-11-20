#![no_std]
#![no_main]
#![feature(ptr_internals)]
use core::panic::PanicInfo;

use multiboot2::{BootInformation, BootInformationHeader, MemoryAreaType};

extern crate linked_list_allocator;
extern crate multiboot2;

use linked_list_allocator::LockedHeap;

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

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) -> ! {
    vga_buffer::clear_screen();
    println!("Hello World{}", "!");

    let boot_info = unsafe {
        BootInformation::load(multiboot_information_address as *const BootInformationHeader)
            .expect("Invalid multiboot format")
    };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    println!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        if area.typ() == MemoryAreaType::Available {
            println!(
                "    start: 0x{:x}, length: 0x{:x}",
                area.start_address(),
                area.size()
            );
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // println!("paniced: {:?}", _info);
    loop {}
}

