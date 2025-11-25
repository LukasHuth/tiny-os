use core::ops::{BitOr, BitOrAssign};

use multiboot2::{ElfSection, ElfSectionFlags};

#[derive(Clone, Copy)]
pub enum EntryFlags {
    Present,
    Writeable,
    UserAccessible,
    WriteThrough,
    NoCache,
    Accessed,
    Dirty,
    HugePage,
    Global,
    NoExecute,
}
impl EntryFlags {
    pub fn from_elf_section_flags(section: &ElfSection) -> u64 {
        let mut flags = 0;

        if section.flags().contains(ElfSectionFlags::ALLOCATED) {
            flags |= EntryFlags::Present;
        }
        if section.flags().contains(ElfSectionFlags::WRITABLE) {
            flags |= EntryFlags::Writeable;
        }
        if !section.flags().contains(ElfSectionFlags::EXECUTABLE) {
            flags |= EntryFlags::NoExecute;
        }
        flags
    }
}
pub trait EntryFlag: const Into<u64> {}
impl EntryFlag for u64 {}
impl EntryFlag for EntryFlags {}
impl const Into<u64> for EntryFlags {
    fn into(self) -> u64 {
        self.value()
    }
}
impl EntryFlags {
    pub const MASK: u64 = Self::Present
        | Self::Writeable
        | Self::UserAccessible
        | Self::WriteThrough
        | Self::NoCache
        | Self::Accessed
        | Self::Dirty
        | Self::HugePage
        | Self::Global
        | Self::NoExecute;
    pub const fn value(&self) -> u64 {
        1 << match self {
            Self::Present => 0,
            Self::Writeable => 1,
            Self::UserAccessible => 2,
            Self::WriteThrough => 3,
            Self::NoCache => 4,
            Self::Accessed => 5,
            Self::Dirty => 6,
            Self::HugePage => 7,
            Self::Global => 8,
            Self::NoExecute => 63,
        }
    }
}
pub trait EntryFlagsHelpers {
    fn is_set(&self, other: EntryFlags) -> bool;
}

impl EntryFlagsHelpers for u64 {
    fn is_set(&self, other: EntryFlags) -> bool {
        self & other.value() != 0
    }
}
impl BitOrAssign<EntryFlags> for u64 {
    fn bitor_assign(&mut self, rhs: EntryFlags) {
        *self |= rhs.value();
    }
}
impl const BitOr<EntryFlags> for u64 {
    type Output = u64;

    fn bitor(self, rhs: EntryFlags) -> Self::Output {
        self | rhs.value()
    }
}
impl<T> const BitOr<T> for EntryFlags where T: EntryFlag {
    type Output = u64;

    fn bitor(self, rhs: T) -> Self::Output {
        T::into(rhs) | self.value()
    }
}
