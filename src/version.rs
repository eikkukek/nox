use crate::serialization::{WriteLe, ReadLe};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Version {
    pub value: u32,
}

impl Version {

    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        let value =
            (major << 22) |
            ((minor & 0x3FF) << 12) |
            (patch & 0xFFF);
        Self {
            value,
        }
    }

    pub const fn as_u32(&self) -> u32 {
        self.value
    }

    pub const fn major(self) -> u32 {
        self.value >> 22
    }

    pub const fn minor(self) -> u32 {
        (self.value >> 12) & 0x3FF
    }

    pub const fn patch(self) -> u32 {
        self.value & 0xFFF
    }

    pub const fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

impl Into<u32> for Version {

    fn into(self) -> u32 {
        self.value
    }
}

impl From<u32> for Version {

    fn from(value: u32) -> Self {
        Self {
            value
        }
    }
}

impl Default for Version {

    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

impl WriteLe for Version {

    #[inline(always)]
    fn write_le<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.value.write_le(writer)
    }
}

impl ReadLe for Version {

    type Error = std::io::Error;

    #[inline(always)]
    fn read_le<R: std::io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(u32::read_le(reader)?.into())
    }
}
