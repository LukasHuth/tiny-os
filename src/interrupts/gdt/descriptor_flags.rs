use core::ops::BitOr;

#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum DescriptorFlags {
    Conforming = 1 << 42,
    Executable = 1 << 43,
    UserSegment = 1 << 44,
    Present = 1 << 47,
    LongMode = 1 << 53,
}
pub trait DescriptorFlag {
    fn as_u64(&self) -> u64;
}
impl DescriptorFlag for DescriptorFlags {
    fn as_u64(&self) -> u64 {
        *self as u64
    }
}
impl DescriptorFlag for u64 {
    #[inline]
    fn as_u64(&self) -> u64 {
        *self
    }
}

impl BitOr<u64> for DescriptorFlags {
    type Output = u64;

    fn bitor(self, rhs: u64) -> Self::Output {
        self.as_u64() | rhs
    }
}
impl BitOr<DescriptorFlags> for u64 {
    type Output = u64;

    fn bitor(self, rhs: DescriptorFlags) -> Self::Output {
        self | rhs.as_u64()
    }
}
impl BitOr<DescriptorFlags> for DescriptorFlags {
    type Output = u64;

    fn bitor(self, rhs: DescriptorFlags) -> Self::Output {
        self.as_u64() | rhs.as_u64()
    }
}
