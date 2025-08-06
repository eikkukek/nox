use nox_mem::AsRaw;

use ash::vk;

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum AttachmentLoadOp {
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum AttachmentStoreOp {
    Store = vk::AttachmentStoreOp::STORE.as_raw(),
    DontCare = vk::AttachmentStoreOp::DONT_CARE.as_raw(),
}

impl From<AttachmentStoreOp> for vk::AttachmentStoreOp {

    fn from(value: AttachmentStoreOp) -> Self {
        Self::from_raw(value.as_raw())
    }
}
