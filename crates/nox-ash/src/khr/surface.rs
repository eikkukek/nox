//! VK_KHR_surface instance-extension.
//!
//! <https://docs.vulkan.org/refpages/latest/refpages/source/VK_KHR_surface.html>

pub use ash::khr::surface::*;

use crate::{
    vk,
    prelude::VkResult,
};

use core::ops::Deref;

/// Vk_KHR_surface instance-level functions.
#[derive(Clone)]
pub struct Instance {
    instance: ash::khr::surface::Instance,
}

impl Deref for Instance {

    type Target = ash::khr::surface::Instance;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl Instance {

    #[inline(always)]
    pub fn new(entry: &crate::Entry, instance: &crate::Instance) -> Self {
        Self {
            instance: ash::khr::surface::Instance::new(entry, instance),
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPhysicalDeviceSurfaceFormatsKHR.html>
    ///
    /// # Safety
    /// All Vulkan calls are inherently unsafe, because [`ash`] doesn't do any error checking by
    /// itself.
    #[inline(always)]
    pub unsafe fn get_physical_device_surface_formats_khr(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_format_count: &mut u32,
        p_surface_formats: *mut vk::SurfaceFormatKHR,
    ) -> VkResult<()>
    {
        unsafe {
            (self.fp().get_physical_device_surface_formats_khr)(
                physical_device,
                surface,
                surface_format_count,
                p_surface_formats,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPhysicalDeviceSurfacePresentModesKHR.html>
    /// # Safety
    /// All Vulkan calls are inherently unsafe, because [`ash`] doesn't do any error checking by
    /// itself.
    #[inline(always)]
    pub unsafe fn get_physical_device_surface_present_modes_khr(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        present_mode_count: &mut u32,
        p_present_modes: *mut vk::PresentModeKHR,
    ) -> VkResult<()>
    {
        unsafe {
            (self.fp().get_physical_device_surface_present_modes_khr)(
                physical_device,
                surface,
                present_mode_count,
                p_present_modes,
            )
        }.result()
    }
}
