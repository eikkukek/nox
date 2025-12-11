use ash::vk;

use nox_mem::{impl_as_raw_bit_op, slot_map::SlotIndex, AsRaw};

use crate::dev::{
    error::{Location, caller},
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
pub struct ResourceId {
    pub(crate) index: SlotIndex<ImageId>,
    pub(crate) image_id: ImageId,
    pub(crate) format: vk::Format,
    pub(crate) samples: MSAA,
    pub(crate) flags: u32,
    pub(crate) loc: Location,
}

impl ResourceId {

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
            image_id: Default::default(),
            format: Default::default(),
            samples: Default::default(),
            flags: 0,
            loc: caller!(),
        }
    }
}
