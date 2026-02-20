use core::{
    ops::Deref,
    ptr,
    ffi,
};

use nox_mem::slice::AllocSlice;

use crate::{
    vk,
    prelude::VkResult,
};

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkDevice.html>
///
/// A version of [`ash::Device`] with slight modifications.
#[derive(Clone)]
pub struct Device {
    pub(crate) device: ash::Device,
}

impl Deref for Device {

    type Target = ash::Device;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl Device {

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyDevice.html>
    ///
    /// # Safety
    /// All resources created via this [`Device`] must be calling this function.
    pub unsafe fn destroy_device(&self, allocation_callbacks: Option<&vk::AllocationCallbacks<'_>>) {
        unsafe {
            self.device.destroy_device(allocation_callbacks);
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkAllocateCommandBuffers.html>
    ///
    /// # Safety
    /// All Vulkan calls are inherently unsafe, because [`ash`] doesn't do any error checking by
    /// itself.
    pub unsafe fn allocate_command_buffers(
        &self,
        info: &vk::CommandBufferAllocateInfo<'_>,
        p_command_buffers: *mut vk::CommandBuffer,
    ) -> VkResult<()>
    {
        unsafe {
            (self.fp_v1_0().allocate_command_buffers)(
                self.handle(),
                info,
                p_command_buffers,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkAllocateDescriptorSets.html>
    ///
    /// # Safety
    /// All Vulkan calls are inherently unsafe, because [`ash`] doesn't do any error checking by
    /// itself.
    pub unsafe fn allocate_descriptor_sets(
        &self,
        info: &vk::DescriptorSetAllocateInfo,
        p_descriptor_sets: *mut vk::DescriptorSet
    ) -> VkResult<()>
    {
        unsafe {
            (self.fp_v1_0().allocate_descriptor_sets)(
                self.handle(),
                info,
                p_descriptor_sets,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPipelineCacheData.html>
    ///
    /// # Safety
    /// All Vulkan calls are inherently unsafe, because [`ash`] doesn't do any error checking by
    /// itself.
    pub unsafe fn get_pipeline_cache_data(
        &self,
        pipeline_cache: vk::PipelineCache,
    ) -> VkResult<Box<[u8]>>
    {
        unsafe {
            let mut data_size = 0;
            (self.device.fp_v1_0().get_pipeline_cache_data)(
                self.handle(),
                pipeline_cache,
                &mut data_size,
                ptr::null_mut(),
            ).result()?;
            let mut data = Box::uninit_slice(data_size);
            (self.device.fp_v1_0().get_pipeline_cache_data)(
                self.handle(),
                pipeline_cache,
                &mut data_size,
                data.as_mut_ptr() as *mut ffi::c_void,
            ).result_with_success(data)
        }
    }
}
