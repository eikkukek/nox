use ash::vk;

use super::Error;

pub trait DeviceMemory: 'static + Send + Sync {

    fn device_memory(&self) -> vk::DeviceMemory;

    fn offset(&self) -> vk::DeviceSize;

    fn size(&self) -> vk::DeviceSize;

    unsafe fn free_memory(&self);
}

pub trait MemoryBinder {

    type Memory: DeviceMemory;

    fn bind_image_memory(&mut self, image: vk::Image) -> Result<Self::Memory, Error>;

    fn bind_buffer_memory(&mut self, buffer: vk::Buffer) -> Result<Self::Memory, Error>;
}
