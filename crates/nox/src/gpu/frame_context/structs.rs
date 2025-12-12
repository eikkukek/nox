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
    SwapchainImage = 0x4,
}

impl_as_raw_bit_op!(ResourceFlags);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ResourceId {
    pub(crate) index: SlotIndex<ImageId>,
    pub(crate) image_id: ImageId,
    pub(crate) format: vk::Format,
    pub(crate) samples: MSAA,
    pub(crate) flags: u32,
    pub(super) loc: Option<Location>,
}

impl ResourceId {

    #[inline(always)]
    pub(crate) fn samples(&self) -> MSAA {
        self.samples
    }

    #[inline(always)]
    pub fn is_swapchain_image(&self) -> bool {
        self.flags & ResourceFlags::SwapchainImage == ResourceFlags::SwapchainImage
    }
}

impl Default for ResourceId {

    #[track_caller]
    fn default() -> Self {
        Self {
            index: Default::default(),
            image_id: Default::default(),
            format: Default::default(),
            samples: Default::default(),
            flags: 0,
            loc: caller!(),
        }
    }
}

impl Tracked for ResourceId {

    #[inline(always)]
    fn location(&self) -> Option<Location> {
        self.loc
    }
}
