use core::{
    alloc::{AllocError, Allocator, GlobalAlloc},
    ptr::{null_mut, NonNull},
    sync::atomic::AtomicUsize,
};

use alloc::{collections::btree_set::BTreeSet, vec::Vec};
use spin::Mutex;

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

unsafe impl Allocator for BumpAllocator {
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
pub const HEAP2_SIZE: usize = 1024 * 1024; // 1 MiB

// #[global_allocator]
// static HEAP_ALLOCATOR: BumpAllocator = BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE);
// #[global_allocator]
pub static HEAP: LockedHeap = LockedHeap::new();
pub struct LockedHeap(Mutex<Heap>);
impl LockedHeap {
    pub const fn new() -> Self {
        Self(Mutex::new(Heap::new()))
    }
    pub fn init(&self, heap_start: usize, heap_size: usize) {
        self.0.lock().init(heap_start, heap_size);
    }
}
enum Heap {
    Empty,
    Initialized {
        available: Vec<HeapEntry, BumpAllocator>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeapEntry {
    start: usize,
    size: usize,
}
impl HeapEntry {
    pub const fn new(start: usize, size: usize) -> Self {
        Self { start, size }
    }

    fn end(&self) -> usize {
        self.start + self.size
    }
}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}
impl Heap {
    pub const fn new() -> Self {
        Self::Empty
    }
    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        match self {
            Self::Empty => {
                let alloc =
                    BumpAllocator::new(HEAP_SMALL_START, HEAP_SMALL_START + HEAP_SMALL_SIZE);
                let mut available: Vec<HeapEntry, BumpAllocator> =
                    Vec::with_capacity_in((HEAP_SMALL_SIZE / size_of::<HeapEntry>()) - 10, alloc);
                available.push(HeapEntry::new(heap_start, heap_size));
                *self = Self::Initialized { available }
            }
            Self::Initialized { .. } => panic!("Init must only me called once"),
        }
    }
}
unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.0.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.0.lock().dealloc(ptr, layout)
    }
}
impl HeapEntry {
    pub fn remove(&mut self, size: usize) {
        self.start += size;
        self.size -= size;
    }
    pub fn enough(&self, size: usize, align: usize) -> bool {
        let aligned_start = (self.start + align - 1) & !(align - 1);
        let padding = aligned_start - self.start;
        self.size >= size + padding
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    fn front_remaining(self, size: usize) -> (*mut u8, HeapEntry) {
        let ptr = self.start as *mut u8;
        let start = self.start + size;
        let size = self.size - size;
        (ptr, HeapEntry { start, size })
    }
    fn back_remaining(self, size: usize) -> (*mut u8, HeapEntry) {
        let ptr = (self.start + self.size - size) as *mut u8;
        let start = self.start;
        let size = self.size - size;
        (ptr, HeapEntry { start, size })
    }
    fn split_remaining(self, size: usize, padding: usize) -> (*mut u8, HeapEntry, HeapEntry) {
        let ptr_usize = self.start + padding;
        let ptr = ptr_usize as *mut u8;
        let start_0 = self.start;
        let size_0 = ptr_usize - start_0;
        let start_1 = ptr_usize + size;
        let size_1 = self.start + self.size - start_1;
        (
            ptr,
            HeapEntry {
                start: start_0,
                size: size_0,
            },
            HeapEntry {
                start: start_1,
                size: size_1,
            },
        )
    }
    pub fn remaining(self, size: usize, align: usize) -> (*mut u8, HeapEntry, Option<HeapEntry>) {
        let aligned_start = (self.start + align - 1) & !(align - 1);
        let padding = aligned_start - self.start;
        if padding == 0 {
            let (ptr, entry) = self.front_remaining(size);
            (ptr, entry, None)
        } else if (self.start + self.size - size).is_aligned(align) {
            let (ptr, entry) = self.back_remaining(size);
            (ptr, entry, None)
        } else {
            let (ptr, entry_0, entry_1) = self.split_remaining(size, padding);
            (ptr, entry_0, Some(entry_1))
        }
    }
}
trait IsAligned {
    fn is_aligned(self, align: usize) -> bool;
}
impl IsAligned for usize {
    fn is_aligned(self, align: usize) -> bool {
        // 0b0001_0000 & !(0b0000_1000 -1)
        // 0b0001_0000 & !(0b0000_0111)
        // 0b0001_0000 & 0b1111_1000
        // 0b0001_0000
        self & !(align - 1) == self
    }
}
impl Heap {
    pub fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
        match self {
            Self::Initialized { available } => {
                for i in 0..available.len() {
                    assert!(layout.align().is_power_of_two());
                    let heap_entry = &available[i];

                    let size = layout.size();
                    let align = layout.align();

                    if !heap_entry.enough(size, align) {
                        continue;
                    }
                    let (ptr, updated, new) = heap_entry.remaining(size, align);
                    if updated.is_empty() {
                        if let Some(new) = new {
                            available[i] = new;
                        } else {
                            available.remove(i);
                        }
                    } else {
                        available[i] = updated;
                        if let Some(new) = new {
                            available.push(new);
                        }
                    }
                    return ptr;
                }
                null_mut()
            }
            Self::Empty => panic!("Alloc used before init"),
        }
    }
    pub fn dealloc(&mut self, ptr: *mut u8, layout: core::alloc::Layout) {
        match self {
            Self::Empty => panic!("Dealloc used before init"),
            Self::Initialized { available } => {
                let start = ptr as usize;
                let size = layout.size();
                let mut idx = available.iter().position(|e| e.start >= start).unwrap_or(available.len());
                available.insert(idx, HeapEntry { start, size });

                let idx_start = idx;
                // Coalesce prev
                while idx > 0 && available[idx].start == available[idx-1].end() {
                    available[idx-1].size += available[idx].size;
                    idx -= 1;
                }
                // cleanup
                if idx < idx_start {
                    available.drain(idx+1..=idx_start);
                }
                let idx_start = idx;
                // Coalesce next
                while idx < available.len()-1 && available[idx].end() == available[idx + 1].start {
                    available[idx+1].start = available[idx].start;
                    available[idx+1].size += available[idx].size;
                    idx += 1;
                }
                // cleanup
                if idx_start < idx {
                    available.drain(idx_start..idx);
                }
            }
        }
    }
}
