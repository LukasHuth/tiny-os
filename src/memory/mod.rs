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
        FrameIter {
            start, end
        }
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
pub use paging::remap_the_kernel;
// pub use paging::test_paging;
