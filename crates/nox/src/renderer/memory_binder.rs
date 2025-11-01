use core::ptr::NonNull;

use ash::vk;

use super::Error;

pub trait DeviceMemory: 'static + Send + Sync {

    fn device_memory(&self) -> vk::DeviceMemory;

    fn offset(&self) -> vk::DeviceSize;

    fn size(&self) -> vk::DeviceSize;

    unsafe fn free_memory(&self);

    unsafe fn map_memory(&mut self) -> Option<NonNull<u8>>;
}

pub trait MemoryBinder {


    fn bind_image_memory(
        &mut self,
        image: vk::Image,
        fall_back: Option<&mut dyn FnMut(vk::Image) -> Result<Box<dyn DeviceMemory>, Error>>,
    ) -> Result<Box<dyn DeviceMemory>, Error>;

    fn bind_buffer_memory(
        &mut self,
        buffer: vk::Buffer,
        fall_back: Option<&mut dyn FnMut(vk::Buffer) -> Result<Box<dyn DeviceMemory>, Error>>,
    ) -> Result<Box<dyn DeviceMemory>, Error>;
}
