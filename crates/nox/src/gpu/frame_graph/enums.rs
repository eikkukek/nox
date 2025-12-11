use nox_mem::AsRaw;

use ash::vk;

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum AttachmentLoadOp {
    #[default]
    Load = vk::AttachmentLoadOp::LOAD.as_raw(),
    Clear = vk::AttachmentLoadOp::CLEAR.as_raw(),
    DontCare = vk::AttachmentLoadOp::DONT_CARE.as_raw(),
}

impl From<AttachmentLoadOp> for vk::AttachmentLoadOp {

    fn from(value: AttachmentLoadOp) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum AttachmentStoreOp {
    #[default]
    Store = vk::AttachmentStoreOp::STORE.as_raw(),
    DontCare = vk::AttachmentStoreOp::DONT_CARE.as_raw(),
}

impl From<AttachmentStoreOp> for vk::AttachmentStoreOp {

    fn from(value: AttachmentStoreOp) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug)]
pub enum ResolveMode {
    FirstSample = vk::ResolveModeFlags::SAMPLE_ZERO.as_raw(),
    Average = vk::ResolveModeFlags::AVERAGE.as_raw(),
    Min = vk::ResolveModeFlags::MIN.as_raw(),
    Max = vk::ResolveModeFlags::MAX.as_raw(),
}

impl From<ResolveMode> for vk::ResolveModeFlags {

    fn from(value: ResolveMode) -> Self {
        Self::from_raw(value.as_raw())
    }
}
