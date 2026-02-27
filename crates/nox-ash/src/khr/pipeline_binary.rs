//! VK_KHR_pipeline_binary device extension.
//!
//! <https://docs.vulkan.org/refpages/latest/refpages/source/VK_KHR_pipeline_binary.html>

use {
    crate::{
        load_fn, prelude::VkResult, vk::{self, ResultKHR},
    },
    core::ffi::{CStr, c_void},
    nox_mem::{
        option::OptionExt, result::ResultExt
    },
};

pub use {
    vk::KHR_PIPELINE_BINARY_NAME as NAME,
    vk::KHR_PIPELINE_BINARY_SPEC_VERSION as SPEC_VERSION,
};

/// Raw VK_KHR_pipeline_binary device-level function pointers.
#[derive(Clone)]
pub struct DeviceFn {
    pub create_pipeline_binaries_khr: vk::PFN_vkCreatePipelineBinariesKHR,
    pub destroy_pipeline_binary_khr: vk::PFN_vkDestroyPipelineBinaryKHR,
    pub get_pipeline_binary_data_khr: vk::PFN_vkGetPipelineBinaryDataKHR,
    pub get_pipeline_key_khr: vk::PFN_vkGetPipelineKeyKHR,
    pub release_captured_pipeline_data_khr: vk::PFN_vkReleaseCapturedPipelineDataKHR,
}

impl DeviceFn {

    pub fn load<F: FnMut(&CStr) -> *const c_void>(mut f: F) -> Self {
        Self {
            create_pipeline_binaries_khr: unsafe { load_fn!(
                fn create_pipeline_binaries_khr(
                    vk::Device,
                    *const vk::PipelineBinaryCreateInfoKHR,
                    *const vk::AllocationCallbacks,
                    *mut vk::PipelineBinaryHandlesInfoKHR,
                ) -> vk::Result,
                f,
                c"vkCreatePipelineBinariesKHR",
                vk::PFN_vkCreatePipelineBinariesKHR,
            ) },
            destroy_pipeline_binary_khr: unsafe { load_fn!(
                fn destroy_pipeline_binary_khr(
                    vk::Device,
                    vk::PipelineBinaryKHR,
                    *const vk::AllocationCallbacks,
                ) -> (),
                f,
                c"vkDestroyPipelineBinaryKHR",
                vk::PFN_vkDestroyPipelineBinaryKHR,
            ) },
            get_pipeline_binary_data_khr: unsafe { load_fn!(
                fn get_pipeline_binary_data_khr(
                    vk::Device,
                    *const vk::PipelineBinaryDataInfoKHR,
                    *mut vk::PipelineBinaryKeyKHR,
                    *mut usize,
                    *mut c_void,
                ) -> vk::Result,
                f,
                c"vkGetPipelineBinaryDataKHR",
                vk::PFN_vkGetPipelineBinaryDataKHR,
            ) },
            get_pipeline_key_khr: unsafe { load_fn!(
                fn get_pipeline_key_khr(
                    vk::Device,
                    *const vk::PipelineCreateInfoKHR,
                    *mut vk::PipelineBinaryKeyKHR,
                ) -> vk::Result,
                f,
                c"vkGetPipelineKeyKHR",
                vk::PFN_vkGetPipelineKeyKHR,
            ) },
            release_captured_pipeline_data_khr: unsafe { load_fn!(
                fn release_captured_pipeline_data_khr(
                    vk::Device,
                    *const vk::ReleaseCapturedPipelineDataInfoKHR,
                    *const vk::AllocationCallbacks,
                ) -> vk::Result,
                f,
                c"vkReleaseCapturedPipelineDataKHR",
                vk::PFN_vkReleaseCapturedPipelineDataKHR,
            ) },
        }
    }
}

unsafe impl Send for DeviceFn {}
unsafe impl Sync for DeviceFn {}

/// VK_KHR_pipeline_binary device-level functions.
#[derive(Clone)]
pub struct Device {
    fp: DeviceFn,
    handle: vk::Device,
}

impl Device {

    #[inline(always)]
    pub fn new(instance: &crate::Instance, device: &crate::Device) -> Self {
        let handle = device.handle();
        let fp = DeviceFn::load(|name| unsafe {
            core::mem::transmute(instance.get_device_proc_addr(handle, name.as_ptr()))
        });
        Self { fp, handle, }
    }

    #[inline(always)]
    pub fn fp(&self) -> &DeviceFn {
        &self.fp
    }

    #[inline(always)]
    pub fn device(&self) -> vk::Device {
        self.handle
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreatePipelineBinariesKHR.html>
    /// # [`Ok`] values
    /// [`vk::Result::INCOMPLETE`]
    /// [`vk::Result::PIPELINE_BARRIER_MISSING`]
    /// [`vk::Result::SUCCESS`]
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn create_pipeline_binaries_khr(
        &self,
        create_info: &vk::PipelineBinaryCreateInfoKHR<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
        binaries: &mut vk::PipelineBinaryHandlesInfoKHR<'_>,
    ) -> VkResult<vk::Result> {
        unsafe {
            (self.fp().create_pipeline_binaries_khr)(
                self.handle,
                create_info,
                allocator.as_ptr(),
                binaries,
            )
        }.result_with_success(vk::Result::SUCCESS)
        .filter_err(|&err| matches!(
            err,
            vk::Result::INCOMPLETE | vk::Result::PIPELINE_BINARY_MISSING_KHR,
        ).then_some(err))
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyPipelineBinaryKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn destroy_pipeline_binary_khr(
        &self,
        pipeline_binary: vk::PipelineBinaryKHR,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fp().destroy_pipeline_binary_khr)(
                self.handle,
                pipeline_binary,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPipelineBinaryDataKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn get_pipeline_binary_data_khr(
        &self,
        info: &vk::PipelineBinaryDataInfoKHR<'_>,
        pipeline_binary_key: &mut vk::PipelineBinaryKeyKHR<'_>,
        pipeline_binary_data_size: &mut usize,
        pipeline_binary_data: *mut u8,
    ) -> VkResult<()> {
        unsafe {
            (self.fp().get_pipeline_binary_data_khr)(
                self.handle,
                info,
                pipeline_binary_key,
                pipeline_binary_data_size,
                pipeline_binary_data.cast::<c_void>(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPipelineKeyKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn get_pipeline_key_khr<'a>(
        &self,
        pipeline_create_info: &vk::PipelineCreateInfoKHR<'_>,
    ) -> VkResult<vk::PipelineBinaryKeyKHR<'a>> {
        let mut key = vk::PipelineBinaryKeyKHR::default();
        unsafe {
            (self.fp().get_pipeline_key_khr)(
                self.handle,
                pipeline_create_info,
                &mut key,
            )
        }.result_with_success(key)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkReleaseCapturedPipelineDataKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn release_captured_pipeline_data_khr(
        &self,
        info: &vk::ReleaseCapturedPipelineDataInfoKHR<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<()> {
        unsafe {
            (self.fp().release_captured_pipeline_data_khr)(
                self.handle,
                info,
                allocator.as_ptr(),
            )
        }.result()
    }
}
