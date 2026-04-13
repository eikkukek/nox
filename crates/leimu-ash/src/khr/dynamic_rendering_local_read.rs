//! VK_KHR_dynamic_rendering_local_read device extension.
//!
//! <https://docs.vulkan.org/refpages/latest/refpages/source/VK_KHR_dynamic_rendering_local_read.html>

use {
    crate::{
        vk,
        load_fn,
    },
    core::ffi::{CStr, c_void},
};

pub use vk::{
    KHR_DYNAMIC_RENDERING_LOCAL_READ_NAME as NAME,
    KHR_DYNAMIC_RENDERING_LOCAL_READ_SPEC_VERSION as SPEC_VERSION,
};

/// Raw VK_KHR_dynamic_rendering_local_read device-level function pointer.
#[derive(Clone)]
pub struct DeviceFn {
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetRenderingAttachmentLocations.html>
    pub cmd_set_rendering_attachment_locations_khr: vk::PFN_vkCmdSetRenderingAttachmentLocationsKHR,
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetRenderingInputAttachmentIndices.html>
    pub cmd_set_rendering_input_attachment_indices_khr: vk::PFN_vkCmdSetRenderingInputAttachmentIndicesKHR,
}

impl DeviceFn {

    pub fn load<F: FnMut(&CStr) -> *const c_void>(mut f: F) -> Self {
        unsafe { Self {
            cmd_set_rendering_attachment_locations_khr: load_fn!(
                fn cmd_set_rendering_attachment_locations_khr(
                    vk::CommandBuffer,
                    *const vk::RenderingAttachmentLocationInfo,
                ) -> (),
                f,
                c"vkCmdSetRenderingAttachmentLocationsKHR",
                vk::PFN_vkCmdSetRenderingAttachmentLocationsKHR,
            ),
            cmd_set_rendering_input_attachment_indices_khr: load_fn!(
                fn cmd_set_rendering_input_attachment_indices_khr(
                    vk::CommandBuffer,
                    *const vk::RenderingInputAttachmentIndexInfo,
                ) -> (),
                f,
                c"vkCmdSetRenderingInputAttachmentIndicesKHR",
                vk::PFN_vkCmdSetRenderingInputAttachmentIndicesKHR,
            )
        } }
    }
}

unsafe impl Send for DeviceFn {}
unsafe impl Sync for DeviceFn {}

#[derive(Clone)]
pub struct Device {
    fp: DeviceFn,
    handle: vk::Device,
}

impl Device {

    pub fn new(instance: &crate::Instance, device: &crate::Device) -> Self {
        let handle = device.handle();
        let fp = DeviceFn::load(|name| unsafe {
            core::mem::transmute(instance.get_device_proc_addr(handle, name.as_ptr()))
        });
        Self {
            fp,
            handle,
        }
    }

    #[inline(always)]
    pub fn fp(&self) -> &DeviceFn {
        &self.fp
    }

    #[inline(always)]
    pub fn device(&self) -> vk::Device {
        self.handle
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetRenderingAttachmentLocations.html>
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn cmd_set_rendering_attachment_locations_khr(
        &self,
        command_buffer: vk::CommandBuffer,
        location_info: &vk::RenderingAttachmentLocationInfo,
    ) {
        unsafe {
            (self.fp().cmd_set_rendering_attachment_locations_khr)(
                command_buffer,
                location_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetRenderingInputAttachmentIndices.html>
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn cmd_set_rendering_input_attachment_indices_khr(
        &self,
        command_buffer: vk::CommandBuffer,
        input_attachment_index_info: &vk::RenderingInputAttachmentIndexInfo,
    ) {
        unsafe {
            (self.fp().cmd_set_rendering_input_attachment_indices_khr)(
                command_buffer,
                input_attachment_index_info,
            )
        }
    }
}
