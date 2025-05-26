#[derive(Clone, Copy)]
pub struct Version {
    version: u32,
}

impl Version {

    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        let version =
            (major << 22) |
            ((minor & 0x3FF) << 12) |
            (patch & 0xFFF);
        Self {
            version,
        }
    }

    pub fn default() -> Self {
        Self::new(1, 0, 0)
    }

    pub fn from(version: u32) -> Self {
        Self {
            version,
        }
    }

    pub fn as_u32(&self) -> u32 {
        self.version
    }

    pub fn major(&self) -> u32 {
        self.version >> 22
    }

    pub fn minor(&self) -> u32 {
        (self.version >> 12) & 0x3FF
    }

    pub fn patch(&self) -> u32 {
        self.version & 0xFFF
    }
}
