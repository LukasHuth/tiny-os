use crate::memory::{
    paging::entry_flags::{EntryFlags, EntryFlagsHelpers as _},
    Frame,
};

pub struct Entry(u64);

impl Entry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }
    pub fn is_flag_set(&self, other: EntryFlags) -> bool {
        self.0.is_set(other)
    }
    pub fn set(&mut self, frame: Frame, flags: u64) {
        assert!(frame.start_address() & !0x000FFFFF_FFFFF000 == 0);
        self.0 = (frame.start_address() as u64) | flags;
    }
    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.is_flag_set(EntryFlags::Present) {
            Some(Frame::containing_address(
                self.0 as usize & 0x000FFFFF_FFFFF000,
            ))
        } else {
            None
        }
    }
    pub fn flags(&self) -> u64 {
        self.0 & EntryFlags::MASK
    }
}
