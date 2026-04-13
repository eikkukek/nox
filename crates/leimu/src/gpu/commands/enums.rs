use nox_ash::vk;

use nox_proc::Display;

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttachmentLoadOp {
    #[default]
    Load = vk::AttachmentLoadOp::LOAD.as_raw(),
    Clear = vk::AttachmentLoadOp::CLEAR.as_raw(),
    DontCare = vk::AttachmentLoadOp::DONT_CARE.as_raw(),
}

impl AttachmentLoadOp {

    #[inline]
    pub fn as_raw(self) -> i32 {
        self as i32
    }
}

impl From<AttachmentLoadOp> for vk::AttachmentLoadOp {

    fn from(value: AttachmentLoadOp) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttachmentStoreOp {
    #[default]
    Store = vk::AttachmentStoreOp::STORE.as_raw(),
    DontCare = vk::AttachmentStoreOp::DONT_CARE.as_raw(),
}

impl AttachmentStoreOp {

    #[inline]
    pub fn as_raw(self) -> i32 {
        self as i32
    }
}

impl From<AttachmentStoreOp> for vk::AttachmentStoreOp {

    fn from(value: AttachmentStoreOp) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub enum ResolveMode {
    #[display("SampleZero")]
    SampleZero = vk::ResolveModeFlags::SAMPLE_ZERO.as_raw(),
    #[display("Average")]
    Average = vk::ResolveModeFlags::AVERAGE.as_raw(),
    #[display("Min")]
    Min = vk::ResolveModeFlags::MIN.as_raw(),
    #[display("Max")]
    Max = vk::ResolveModeFlags::MAX.as_raw(),
}

impl ResolveMode {

    #[inline]
    pub fn as_raw(self) -> u32 {
        self as u32
    }
}

impl From<ResolveMode> for vk::ResolveModeFlags {

    fn from(value: ResolveMode) -> Self {
        Self::from_raw(value.as_raw())
    }
}
