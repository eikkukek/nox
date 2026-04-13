use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferProperties {
    pub size: DeviceSize,
    pub usage: BufferUsages,
    pub create_flags: vk::BufferCreateFlags,
}
