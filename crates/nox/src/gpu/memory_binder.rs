use core::ptr::NonNull;

use ash::vk;

use crate::dev::error::Error;

#[derive(Debug, Error)]
pub enum MemoryBinderError {
    #[display("vulkan error")]
    VulkanError(#[from] #[source] vk::Result),
    #[display("out of device memory with allocation size {size} and alignment {align}")]
    OutOfDeviceMemory { size: u64, align: u64, },
    #[display("allocated memory is unmappable")]
    UnmappableMemory,
    #[display("allocation size was zero")]
    ZeroSizeAlloc,
    #[display("incompatible memory requirements")]
    IncompatibleMemoryRequirements,
    #[display("{0}")]
    Other(Error),
}

pub trait DeviceMemory: 'static + Send + Sync {

    fn device_memory(&self) -> vk::DeviceMemory;

    fn offset(&self) -> vk::DeviceSize;

    fn size(&self) -> vk::DeviceSize;

    unsafe fn free_memory(&self);

    unsafe fn map_memory(&mut self) -> Result<NonNull<u8>, MemoryBinderError>;
}

pub trait MemoryBinder {

    fn bind_image_memory(
        &mut self,
        image: vk::Image,
        fall_back: Option<&mut dyn FnMut(vk::Image) -> Result<Box<dyn DeviceMemory>, MemoryBinderError>>,
    ) -> Result<Box<dyn DeviceMemory>, MemoryBinderError>;

    fn bind_buffer_memory(
        &mut self,
        buffer: vk::Buffer,
        fall_back: Option<&mut dyn FnMut(vk::Buffer) -> Result<Box<dyn DeviceMemory>, MemoryBinderError>>,
    ) -> Result<Box<dyn DeviceMemory>, MemoryBinderError>;
}
