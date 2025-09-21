use ash::vk;

use nox_mem::{impl_as_raw_bit_op, slot_map::SlotIndex, AsRaw};

use crate::renderer::{
    global_resources::ImageId,
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
pub struct ResourceId {
    pub(crate) index: SlotIndex<ImageId>,
    pub(crate) image_id: ImageId,
    pub(crate) format: vk::Format,
    pub(crate) samples: MSAA,
    pub(crate) flags: u32,
}

impl ResourceId {

    #[inline(always)]
    pub(crate) fn samples(&self) -> MSAA {
        self.samples
    }
}
