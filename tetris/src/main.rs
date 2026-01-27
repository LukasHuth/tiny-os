#![no_std]
#![no_main]

use core::arch::asm;

#[no_mangle]
pub extern "C" fn tetris_user_main() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}
