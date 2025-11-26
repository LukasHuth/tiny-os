use core::{
    alloc::{AllocError, Allocator, GlobalAlloc},
    ptr::{null_mut, NonNull},
    sync::atomic::AtomicUsize,
};

use alloc::vec::Vec;

#[derive(Debug)]
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: AtomicUsize,
}
impl BumpAllocator {
    pub const fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            next: AtomicUsize::new(heap_start),
        }
    }
}

unsafe impl Sync for BumpAllocator {}

unsafe impl<'a> Allocator for &'a BumpAllocator {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        loop {
            use core::sync::atomic::Ordering;
            let current_next = self.next.load(Ordering::Relaxed);
            let alloc_start = align_up(current_next, layout.align());
            let alloc_end = alloc_start.saturating_add(layout.size());

            if alloc_end <= self.heap_end {
                let ptr = self
                    .next
                    .compare_exchange(
                        current_next,
                        alloc_end,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .ok()
                    .map(|_| NonNull::new(current_next as *mut u8))
                    .flatten();
                if let Some(ptr) = ptr {
                    return Ok(NonNull::slice_from_raw_parts(ptr, layout.size()));
                }
            } else {
                return Err(AllocError);
            }
        }
    }

    unsafe fn deallocate(&self, _ptr: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        // do nothing, leak memory
    }
}
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        <&Self as Allocator>::allocate(&self, layout)
            .map(|slice| slice.as_mut_ptr())
            .unwrap_or(null_mut())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let ptr = NonNull::new(ptr);
        if let Some(ptr) = ptr {
            <&Self as Allocator>::deallocate(&self, ptr, layout);
        }
    }
}
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}
pub const HEAP_SMALL_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SMALL_SIZE: usize = 100 * 1024; // 100 KiB
pub const HEAP2_START: usize = 0o_000_002_000_000_0000;
pub const HEAP2_SIZE: usize = 100 * 1024; // 100 KiB

// #[global_allocator]
// static HEAP_ALLOCATOR: BumpAllocator = BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE);
static HEAP: Mutex<Heap> = Mutex
enum Heap {
    Empty,
    Initialized,
}
