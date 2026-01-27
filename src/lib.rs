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
#![feature(impl_trait_in_assoc_type)]
use core::panic::PanicInfo;

use alloc::{boxed::Box, string::String};
use multiboot2::{BootInformation, BootInformationHeader};
use x86_64::structures::gdt::SegmentSelector;

extern crate alloc;
extern crate multiboot2;

#[macro_use]
extern crate lazy_static;

extern crate pic8259;

use crate::{
    interrupts::switch_to_user, memory::FrameAllocator as _, vga_buffer::{Color, ColorCode}
};

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
    vga_buffer::disable_cursor();
    if DEBUG_PRINTS {
        println!("Hello World{}", "!");
    }

    let boot_info = unsafe {
        BootInformation::load(multiboot_information_address as *const BootInformationHeader)
            .expect("Invalid multiboot format")
    };
    enable_nxe_bit();
    enable_write_protection_bit();

    let mut memory_controller = memory::init(&boot_info);

    interrupts::init(&mut memory_controller);

    x86_64::instructions::interrupts::enable();

    println!("It did not crash!");

    let user_stack_top = memory_controller.alloc_stack(5).unwrap();
    load_user_program();
    
    unsafe { switch_to_user(memory::USERMODE_CODE_START as u64, user_stack_top.top() as u64) };

    // let tetris = tetris::Tetris::new();
    // tetris.run();

    loop {
        x86_64::instructions::hlt();
    }
}

// TODO: real bin
const TETRIS_BIN: &[u8] = include_bytes!("../tetris_user.bin");
fn load_user_program() {
    unsafe {
        core::ptr::copy_nonoverlapping(TETRIS_BIN.as_ptr(), memory::USERMODE_CODE_START as *mut u8, TETRIS_BIN.len());
    }
}

mod tetris;

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

mod utility;
