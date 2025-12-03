use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static CPU_HAS_RDRAND: AtomicBool = AtomicBool::new(false);
static LAST_VALUE: AtomicU64 = AtomicU64::new(0x123456789ABCDEF0);

#[allow(unused)]
pub fn init() {
    if cpu_has_rdrand() {
        CPU_HAS_RDRAND.store(true, Ordering::Relaxed);
    }
}

fn cpu_has_rdrand() -> bool {
    let cpuid = unsafe { core::arch::x86_64::__cpuid(1) };
    (cpuid.ecx & (1 << 30)) != 0
}
pub fn rand() -> u64 {
    if CPU_HAS_RDRAND.load(Ordering::Relaxed) {
        let mut value: u64 = 0;
        for _ in 0..10 {
            if unsafe { core::arch::x86_64::_rdrand64_step(&mut value) } == 1 {
                return value;
            }
        }
    }
    let mut x = LAST_VALUE.load(Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    LAST_VALUE.swap(x, Ordering::Relaxed)
}
