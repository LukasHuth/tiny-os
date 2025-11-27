#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub number: usize,
}
pub const PAGE_SIZE_LOG_2: u64 = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SIZE_LOG_2;

impl Frame {
    fn containing_address(address: usize) -> Frame {
        Frame {
            number: address >> PAGE_SIZE_LOG_2,
        }
    }
    pub fn start_address(&self) -> usize {
        self.number << PAGE_SIZE_LOG_2
    }
    pub fn clone(&self) -> Frame {
        Frame {
            number: self.number,
        }
    }

    fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter { start, end }
    }
}
struct FrameIter {
    start: Frame,
    end: Frame,
}
impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
}
pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}
mod area_frame_allocator;
pub use area_frame_allocator::*;
mod paging;
use multiboot2::{BootInformation, ElfSection};
use once::assert_has_not_been_called;
pub use paging::remap_the_kernel;

use crate::memory::{heap_allocator::{HEAP2_SIZE, HEAP2_START}, paging::entry_flags::EntryFlags};
// pub use paging::test_paging;
mod heap_allocator;
pub fn init(boot_info: &BootInformation) {
    assert_has_not_been_called!("memory::init must only be called once");
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("Elf-sections tag required");

    let kernel_start = elf_sections_tag
        .sections()
        .filter(ElfSection::is_allocated)
        .map(|s| s.start_address())
        .min()
        .unwrap();
    let kernel_end = elf_sections_tag
        .sections()
        .filter(ElfSection::is_allocated)
        .map(|s| s.start_address() + s.size())
        .max()
        .unwrap();

    if crate::DEBUG_PRINTS {
        println!(
            "kernel_start: {:#x} kernel_end: {:#x}",
            kernel_start, kernel_end
        );
        println!(
            "multiboot_start: {:#x} multiboot_end: {:#x}",
            boot_info.start_address(),
            boot_info.end_address()
        );
        println!("memory_area count: {}", memory_map_tag.memory_areas().len());
    }
    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        boot_info.start_address(),
        boot_info.end_address(),
        memory_map_tag.memory_areas(),
    );

    let mut active_table = remap_the_kernel(&mut frame_allocator, &boot_info);

    {
        use self::paging::Page;
        use heap_allocator::{HEAP_SMALL_SIZE, HEAP_SMALL_START};
        let heap_start_page = Page::containing_address(HEAP_SMALL_START);
        let heap_end_page = Page::containing_address(HEAP_SMALL_START + HEAP_SMALL_SIZE-1);

        for page in Page::range_inclusive(heap_start_page, heap_end_page) {
            active_table.map(page, EntryFlags::Writeable, &mut frame_allocator);
        }
    }
    {
        use self::paging::Page;
        use heap_allocator::{HEAP2_SIZE, HEAP2_START};
        let heap_start_page = Page::containing_address(HEAP2_START);
        let heap_end_page = Page::containing_address(HEAP2_START + HEAP2_SIZE-1);

        for page in Page::range_inclusive(heap_start_page, heap_end_page) {
            active_table.map(page, EntryFlags::Writeable, &mut frame_allocator);
        }
    }
    heap_allocator::HEAP.init(HEAP2_START, HEAP2_SIZE);
}
