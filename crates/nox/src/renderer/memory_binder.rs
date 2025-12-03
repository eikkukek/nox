use core::ptr::NonNull;

use ash::vk;

#[derive(Debug)]
pub enum MemoryBinderError {
    VulkanError(vk::Result),
    OutOfDeviceMemory { size: u64, align: u64, },
    NonMappableMemory,
    ZeroSizeAlloc,
    IncompatibleMemoryRequirements,
    Other(Box<dyn core::error::Error>),
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

impl core::fmt::Display for MemoryBinderError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VulkanError(e) => write!(f, "{}", e),
            Self::OutOfDeviceMemory { size, align, } => write!(f, "out of device memory with allocation size {} and alignment {}", size, align),
            Self::NonMappableMemory => write!(f, "allocated memory is non-mappable"),
            Self::ZeroSizeAlloc => write!(f, "size of allocation is zero"),
            Self::IncompatibleMemoryRequirements => write!(f, "incompatible memory requirements"),
            Self::Other(err) => write!(f, "{err}"),
        }
    }
}

impl core::error::Error for MemoryBinderError {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::VulkanError(err) => Some(err),
            Self::Other(err) => Some(&**err),
            _ => None,
        }
    }
}

impl From<vk::Result> for MemoryBinderError {

    fn from(value: vk::Result) -> Self {
        Self::VulkanError(value)
    }
}
