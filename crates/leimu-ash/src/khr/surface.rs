//! VK_KHR_surface instance-extension.
//!
//! <https://docs.vulkan.org/refpages/latest/refpages/source/VK_KHR_surface.html>

pub use ash::khr::surface::*;

use crate::{
    vk,
    prelude::VkResult,
};

/// Vk_KHR_surface instance-level functions.
#[derive(Clone)]
pub struct Instance {
    instance: ash::khr::surface::Instance,
}

impl Instance {

    #[inline(always)]
    pub fn new(entry: &crate::Entry, instance: &crate::Instance) -> Self {
        Self {
            instance: ash::khr::surface::Instance::new(entry, instance),
        }
    }

    pub fn fp(&self) -> &InstanceFn {
        self.instance.fp()
    }

    pub fn instance(&self) -> vk::Instance {
        self.instance.instance()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroySurfaceKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    pub unsafe fn destroy_surface(
        &self,
        surface: vk::SurfaceKHR,
        allocator: Option<&vk::AllocationCallbacks<'_>>
    ) {
        unsafe {
            self.instance.destroy_surface(
                surface,
                allocator,
            );
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPhysicalDeviceSurfaceSupportKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
        surface: vk::SurfaceKHR,
    ) -> VkResult<bool> {
        unsafe {
            self.instance.get_physical_device_surface_support(
                physical_device, queue_family_index, surface
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPhysicalDeviceSurfacePresentModesKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn get_physical_device_surface_present_modes_len(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> VkResult<u32>
    {
        let mut len = 0;
        unsafe {
            (self.fp().get_physical_device_surface_present_modes_khr)(
                physical_device,
                surface,
                &mut len,
                core::ptr::null_mut(),
            )
        }.result_with_success(len)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPhysicalDeviceSurfacePresentModesKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn get_physical_device_surface_present_modes(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        out: &mut [vk::PresentModeKHR],
    ) -> VkResult<()>
    {
        let mut len = out.len() as u32;
        unsafe {
            (self.fp().get_physical_device_surface_present_modes_khr)(
                physical_device,
                surface,
                &mut len,
                out.as_mut_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPhysicalDeviceSurfaceCapabilitiesKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn get_physical_device_surface_capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> VkResult<vk::SurfaceCapabilitiesKHR> {
        unsafe {
            self.instance.get_physical_device_surface_capabilities(
                physical_device,
                surface
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPhysicalDeviceSurfaceFormatsKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn get_physical_device_surface_formats_len(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> VkResult<u32>
    {
        let mut len = 0;
        unsafe {
            (self.fp().get_physical_device_surface_formats_khr)(
                physical_device,
                surface,
                &mut len,
                core::ptr::null_mut(),
            )
        }.result_with_success(len)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPhysicalDeviceSurfaceFormatsKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn get_physical_device_surface_formats(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        out: &mut [vk::SurfaceFormatKHR]
    ) -> VkResult<()>
    {
        let mut len = out.len() as u32;
        unsafe {
            (self.fp().get_physical_device_surface_formats_khr)(
                physical_device,
                surface,
                &mut len,
                out.as_mut_ptr(),
            )
        }.result()
    }
}
