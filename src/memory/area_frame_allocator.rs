use multiboot2::{MemoryArea, MemoryAreaType};

use super::{Frame, FrameAllocator};

pub struct AreaFrameAllocator<'a> {
    next_free_frame: Frame,
    current_area: Option<&'a MemoryArea>,
    areas: &'a [MemoryArea],
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}
impl<'a> FrameAllocator for AreaFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            let frame = self.next_free_frame;
            let current_area_last_frame = {
                let address = area.start_address() + area.size() - 1;
                Frame::containing_address(address as usize)
            };
            // println!("frame: {:?} last_frame: {:?}", frame, current_area_last_frame);
            let in_kernel = frame >= self.kernel_start && frame <= self.kernel_end;
            let in_multiboot = frame >= self.multiboot_start && frame <= self.multiboot_end;
            if frame > current_area_last_frame {
                self.choose_next_area();
                self.allocate_frame()
            } else if in_kernel && in_multiboot {
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1,
                };
                self.allocate_frame()
            } else {
                self.next_free_frame.number += 1;
                Some(frame)
            }
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        unimplemented!()
    }
}
impl<'a> AreaFrameAllocator<'a> {
    fn choose_next_area(&mut self) {
        self.current_area = self
            .areas
            .into_iter()
            .filter(|area| {
                let address = area.start_address() + area.size() - 1;
                Frame::containing_address(address as usize) >= self.next_free_frame
            })
            .filter(|area|area.typ() == MemoryAreaType::Available)
            .min_by_key(|area| area.start_address());

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.start_address() as usize);
            // println!(
            //     "Found frame: {:?} end: {:?} area: {:?}",
            //     start_frame,
            //     Frame::containing_address((area.size() + area.start_address()) as usize),
            //     area
            // );
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
    pub fn new(
        kernel_start: usize,
        kernel_end: usize,
        multiboot_start: usize,
        multiboot_end: usize,
        memory_areas: &'a [MemoryArea],
    ) -> AreaFrameAllocator<'a> {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_address(0),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end),
        };
        allocator.choose_next_area();
        allocator
    }
}
