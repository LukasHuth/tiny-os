use core::sync::atomic::Ordering;

/// returns the current time in ms
pub fn time() -> u64 {
    crate::interrupts::TIMER.load(Ordering::Relaxed)
}
