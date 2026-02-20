//! VK_KHR_swapchain device-extension.
//!
//! <https://docs.vulkan.org/refpages/latest/refpages/source/VK_KHR_swapchain.html>

pub use ash::khr::swapchain::*;

use crate::{
    vk,
    prelude::VkResult,
};

use core::ops::Deref;

/// VK_KHR_swapchain device-level functions.
#[derive(Clone)]
pub struct Device {
    device: ash::khr::swapchain::Device,
}

impl Deref for Device {

    type Target = ash::khr::swapchain::Device;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl Device {
    
    #[inline(always)]
    pub fn new(instance: &crate::Instance, device: &crate::Device) -> Self {
        Self {
            device: ash::khr::swapchain::Device::new(instance, device),
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetSwapchainImagesKHR.html>
    ///
    /// # Safety
    /// All Vulkan calls are inherently unsafe, because [`ash`] doesn't do any error checking by
    /// itself.
    #[inline(always)]
    pub unsafe fn get_swapchain_images_khr(
        &self,
        swapchain: vk::SwapchainKHR,
        image_count: &mut u32,
        p_swapchain_images: *mut vk::Image,
    ) -> VkResult<()> {
        unsafe {
            (self.device.fp().get_swapchain_images_khr)(
                self.device(),
                swapchain,
                image_count,
                p_swapchain_images,
            )
        }.result()
    }
}
