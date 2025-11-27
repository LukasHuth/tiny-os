#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(ptr_internals)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_ops)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(btreemap_alloc)]
#![feature(abi_x86_interrupt)]
use core::panic::PanicInfo;

use alloc::{boxed::Box, string::String};
use multiboot2::{BootInformation, BootInformationHeader};

extern crate alloc;
extern crate multiboot2;

#[macro_use]
extern crate lazy_static;

use crate::memory::FrameAllocator as _;

const DEBUG_PRINTS: bool = true;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[macro_use]
mod vga_buffer;
mod interrupts;
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
    enable_nxe_bit();
    enable_write_protection_bit();

    memory::init(&boot_info);

    interrupts::init();

    x86_64::instructions::interrupts::int3();

    println!("It did not crash!");

    loop {}
}

fn enable_nxe_bit() {
    use x86_64::registers::control::{Efer, EferFlags};
    unsafe {
        let efer = Efer::read() | EferFlags::NO_EXECUTE_ENABLE;
        Efer::write(efer);
    }
}
fn enable_write_protection_bit() {
    use x86_64::registers::control::{Cr0, Cr0Flags};
    unsafe { Cr0::write(Cr0::read() | Cr0Flags::WRITE_PROTECT) }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!("PANIC in {} at line {}", location.file(), location.line());
    }
    println!("    {}", info.message());
    loop {}
}
