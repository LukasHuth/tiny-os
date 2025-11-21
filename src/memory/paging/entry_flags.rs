use core::ops::{BitOr, BitOrAssign};

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
impl const BitOr for EntryFlags {
    type Output = u64;

    fn bitor(self, rhs: EntryFlags) -> Self::Output {
        self.value() | rhs.value()
    }
}
