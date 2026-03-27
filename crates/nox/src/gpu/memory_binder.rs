mod default;
mod linear;

use nox_ash::vk;

use crate::sync::Arc;

use crate::{
    error::Error,
    gpu::prelude::*,
};

pub use linear::*;
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

/// A trait for [`vk::DeviceMemory`] objects.
///
/// # Safety
/// - The [`vk::DeviceMemory`] returned by [`DeviceMemory::device_memory`] *should* be valid for at
///   least the lifetime of [`self`].
/// - The offset and size returned by [`DeviceMemory::offset`] and [`DeviceMemory::size`]
///   respectively *should* be within the inner [`vk::DeviceMemory`]'s range.
/// - This *should* not try to remap already mapped [`vk::DeviceMemory`].
pub unsafe trait DeviceMemory: 'static + Send + Sync {

    /// Returns the inner [`vk::DeviceMemory`].
    fn device_memory(&self) -> vk::DeviceMemory;

    /// Returns the offset in the inner [`vk::DeviceMemory`] of this allocation.
    fn offset(&self) -> vk::DeviceSize;

    /// Returns the size of this allocation.
    fn size(&self) -> vk::DeviceSize;

    /// Tries to map the region of this allocation from the inner [`vk::DeviceMemory`].
    ///
    /// If the inner [`vk::DeviceMemory`] is already mapped, this *should* return a slice to the
    /// already mapped memory.
    fn map_memory(&mut self) -> Result<(*mut u8, usize)>;
}

mod device_obj {

    use super::DeviceMemory;

    nox_mem::smallbox!(
        pub Obj: DeviceMemory
    );

    unsafe impl<const N_BUF: usize> Send for Obj<N_BUF> {}
    unsafe impl<const N_BUF: usize> Sync for Obj<N_BUF> {}
}

/// A [`DeviceMemory`] trait object.
///
/// Doesn't allocate extra memory with [`std::alloc`], if the underlying memory object is less than
/// or equal to 46 bytes.
///
/// Created with [`nox_mem::smallbox`].
pub type DeviceMemoryObj = device_obj::Obj<46>;

impl DeviceMemoryObj {

    #[inline(always)]
    pub fn overlaps(&self, other: &Self) -> bool {
        if self.device_memory() == other.device_memory() {
            let src_off = self.offset();
            let dst_off = other.offset();
            src_off < dst_off + other.size() &&
            dst_off < src_off + self.size()
        } else {
            false
        }
    }
}

/// A trait for [`vk::DeviceMemory`] allocators.
///
/// # Safety
/// - [`MemoryBinder::alloc`] *should* always allocate memory according to the
///   [`vk::MemoryRequirements2`] requirements.
/// - Memory *should* never be mapped more than once per [`vk::DeviceMemory`] object.
/// - Memory allocated by this trait *should* be valid even after dropping the [`MemoryBinder`].
pub unsafe trait MemoryBinder: 'static + Send + Sync {

    /// Returns the maximum allocation size supported by this allocator.
    fn max_alloc_size(&self) -> vk::DeviceSize;

    /// Returns whether the memory allocated by this allocator is mapped or mappable.
    fn is_mappable(&self) -> bool;

    /// Allocates device memory.
    ///
    /// # Safety
    /// - `memory_requirements` *must* be a valid [`vk::MemoryRequirements2`] structure.
    unsafe fn alloc(
        &mut self,
        memory_requirements: &vk::MemoryRequirements2,
    ) -> Result<DeviceMemoryObj>;

    /// This *may* release all allocations made by this allocator back to the allocator.
    ///
    /// # Safety
    /// - This *should* never free any [`vk::DeviceMemory`].
    /// - Previous allocations *may* be overwritten by future allocations.
    unsafe fn release_resources(&mut self);
}

/// A simple trait for building [`MemoryBinder`]s.
pub trait MemoryBinderAttributes {

    /// The [`MemoryBinder`] these attributes belong to.
    type Binder: MemoryBinder;

    /// The type name of the allocator.
    const NAME: &str;

    /// Builds an allocator for a [`LogicalDevice`].
    fn build(self, device: LogicalDevice) -> Result<Self::Binder>;
}

pub struct MemoryBinderInfo {
    pub max_alloc_size: vk::DeviceSize,
    pub is_mappable: bool,
}
