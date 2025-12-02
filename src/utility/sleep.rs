use core::sync::atomic::Ordering;

use crate::interrupts::TIMER;

/// Sleeps at least for `ms` milliseconds
pub fn sleep(ms: u64) {
    let start = TIMER.load(Ordering::Relaxed);
    while TIMER.load(Ordering::Relaxed) - start < ms {
    }
}
