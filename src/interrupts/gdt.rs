use core::ops::Range;

use x86_64::{
    instructions::tables::lgdt,
    structures::{gdt::SegmentSelector, tss::TaskStateSegment, DescriptorTablePointer},
    VirtAddr,
};

use crate::interrupts::gdt::descriptor_flags::{DescriptorFlag, DescriptorFlags};

pub struct Gdt {
    table: [u64; 8],
    next_free: usize,
}

impl Gdt {
    pub fn new() -> Self {
        Self {
            table: [0; 8],
            next_free: 1,
        }
    }
    pub fn add_entry(&mut self, entry: Descriptor, ring: x86_64::PrivilegeLevel) -> SegmentSelector {
        let index = match entry {
            Descriptor::CodeDataSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_high) => {
                let index = self.push(value_low);
                self.push(value_high);
                index
            }
        };
        SegmentSelector::new(index as u16, ring)
    }
    fn push(&mut self, value: u64) -> usize {
        if self.next_free < self.table.len() {
            let index = self.next_free;
            self.table[index] = value;
            self.next_free += 1;
            index
        } else {
            panic!("GDT full")
        }
    }
    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            base: VirtAddr::new(self.table.as_ptr() as u64),
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
        };

        unsafe { lgdt(&ptr) };
    }
}
pub enum Descriptor {
    CodeDataSegment(u64),
    SystemSegment(u64, u64),
}
mod descriptor_flags;
impl Descriptor {
    pub fn kernel_code_segment() -> Descriptor {
        use descriptor_flags::DescriptorFlags::*;
        let flags = UserSegment | Present | Executable | LongMode;
        Descriptor::CodeDataSegment(flags)
    }
    pub(crate) fn kernel_data_segment() -> Descriptor {
        use descriptor_flags::DescriptorFlags::*;
        let flags = UserSegment | Present;
        Descriptor::CodeDataSegment(flags)
    }
    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        use core::mem::size_of;

        let ptr = tss as *const _ as u64;

        let mut low = DescriptorFlags::Present.as_u64();
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        low.set_bits(40..44, 0b1001);

        let mut high = 0u64;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }

    pub(crate) fn user_code_segment() -> Descriptor {
        use descriptor_flags::DescriptorFlags::*;
        let flags = UserSegment | Present | Executable | LongMode | Ring3;
        Descriptor::CodeDataSegment(flags)
    }

    pub(crate) fn user_data_segment() -> Descriptor {
        use descriptor_flags::DescriptorFlags::*;
        let flags = UserSegment | Present | Ring3;
        Descriptor::CodeDataSegment(flags)
    }
}
trait BitOps {
    fn set_bits(&mut self, range: Range<u8>, value: Self);
    fn get_bits(&self, range: Range<u8>) -> Self;
}
impl BitOps for u64 {
    fn set_bits(&mut self, range: Range<u8>, value: Self) {
        let mask = (1 << range.len()) - 1;
        // clear
        *self &= !(mask << range.start);
        // set
        *self |= (value & mask) << range.start;
    }

    fn get_bits(&self, range: Range<u8>) -> Self {
        let length = range.len();
        // extract
        (self >> range.start) & ((1 << length) - 1)
    }
}
