use ash::vk;

use super::Error;

pub trait DeviceMemory: 'static {

    fn device_memory(&self) -> vk::DeviceMemory;

    fn size(&self) -> vk::DeviceSize;

    fn offset(&self) -> vk::DeviceSize;

    unsafe fn free_memory(&mut self);
}

pub trait MemoryBinder {

    type Memory: DeviceMemory;

    fn bind_image_memory(&mut self, image: vk::Image) -> Result<Self::Memory, Error>;

    fn bind_buffer_memory(&mut self, buffer: vk::Buffer) -> Result<Self::Memory, Error>;
}
