use ash::vk;

use nox_mem::{impl_as_raw_bit_op, slot_map::SlotIndex, AsRaw};

use crate::dev::{
    error::{Location, Tracked, caller},
};

use crate::gpu::*;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, AsRaw)]
pub enum ResourceFlags {
    Transient = 0x1,
    Sampleable = 0x2,
}

impl_as_raw_bit_op!(ResourceFlags);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImageSourceId {
    Owned(ImageId),
    SwapchainImage(win::WindowId),
}

impl Default for ImageSourceId {

    fn default() -> Self {
        Self::Owned(Default::default())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ResourceId {
    pub(crate) index: SlotIndex<ImageId>,
    pub(crate) source: ImageSourceId,
    pub(crate) format: vk::Format,
    pub(crate) samples: MSAA,
    pub(crate) flags: u32,
    pub(super) loc: Option<Location>,
}

impl ResourceId {

    #[inline(always)]
    pub fn image_id(&self) -> Option<ImageId> {
        match self.source {
            ImageSourceId::Owned(id) => Some(id),
            ImageSourceId::SwapchainImage(_) => None,
        }
    }

    #[inline(always)]
    pub(crate) fn samples(&self) -> MSAA {
        self.samples
    }
}

impl Default for ResourceId {

    #[track_caller]
    fn default() -> Self {
        Self {
            index: Default::default(),
            source: Default::default(),
            format: Default::default(),
            samples: Default::default(),
            flags: 0,
            loc: Some(caller!()),
        }
    }
}

impl Tracked for ResourceId {

    #[inline(always)]
    fn location(&self) -> Option<Location> {
        self.loc
    }
}
