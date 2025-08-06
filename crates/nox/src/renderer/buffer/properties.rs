use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BufferProperties {
    pub size: vk::DeviceSize,
    pub usage: vk::BufferUsageFlags,
    pub create_flags: vk::BufferCreateFlags,
}
