mod default;
mod linear;

use nox_ash::vk;

use crate::sync::Arc;

use crate::{
    dev::error::Error,
    gpu::Vulkan,
};

pub use linear::LinearBinder;
pub(crate) use default::*;

pub type Result<T> = core::result::Result<T, MemoryBinderError>;

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

    fn map_memory(&mut self) -> Result<&mut [u8]>;
}

pub unsafe trait MemoryBinder: 'static + Send + Sync {

    fn max_alloc_size(&self) -> vk::DeviceSize;

    fn is_mappable(&self) -> bool;

    unsafe fn alloc(
        &mut self,
        memory_requirements: &vk::MemoryRequirements2,
    ) -> Result<Box<dyn DeviceMemory>>;

    unsafe fn release_resources(&mut self);
}

pub trait MemoryBinderAttributes {

    type Binder: MemoryBinder;

    const NAME: &str;

    fn build(self, vulkan: Arc<Vulkan>) -> Result<Self::Binder>;
}
