#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub unsafe extern "C" fn _start() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}
