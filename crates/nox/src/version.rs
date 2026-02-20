use core::fmt::{self, Display};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Version(pub u32);

impl Version {

    pub const MAX: Version = Version(!0);

    #[inline(always)]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        let value =
            (major << 22) |
            ((minor & 0x3FF) << 12) |
            (patch & 0xFFF);
        Self(value)
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub const fn from_u32(value: u32) -> Self {
        Self(value)
    }

    #[inline(always)]
    pub const fn major(self) -> u32 {
        self.0 >> 22
    }

    #[inline(always)]
    pub const fn minor(self) -> u32 {
        (self.0 >> 12) & 0x3FF
    }

    #[inline(always)]
    pub const fn patch(self) -> u32 {
        self.0 & 0xFFF
    }

    #[inline(always)]
    pub const fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

impl From<Version> for u32 {

    #[inline(always)]
    fn from(value: Version) -> u32 {
        value.0
    }
}

impl From<u32> for Version {

    #[inline(always)]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Default for Version {

    #[inline(always)]
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

impl PartialEq<u32> for Version {

    #[inline(always)]
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<u32> for Version {

    fn partial_cmp(&self, other: &u32) -> Option<core::cmp::Ordering> {
        Some(self.0.cmp(other))
    }
}

impl Display for Version {

    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}",
            self.major(),
            self.minor(),
            self.patch(),
        )
    }
}
