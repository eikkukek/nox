use ash::vk;

use nox_mem::{AsRaw, impl_as_raw_bit_op};

use crate::renderer::{
    global_resources::ImageSourceID,
    MSAA,
};

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, AsRaw)]
pub enum ResourceFlags {
    Transient = 0x1,
    Sampleable = 0x2,
}

impl_as_raw_bit_op!(ResourceFlags);

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourceID {
    pub(crate) id: ImageSourceID,
    pub(crate) format: vk::Format,
    pub(crate) samples: MSAA,
    pub(crate) flags: u32,
}

impl ResourceID {
    
    #[inline(always)]
    pub(crate) fn vk_format(&self) -> vk::Format {
        self.format
    }

    #[inline(always)]
    pub(crate) fn samples(&self) -> MSAA {
        self.samples
    }
}

impl From<ResourceID> for ImageSourceID {

    fn from(value: ResourceID) -> Self {
        value.id
    }
}
