use core::ops::Deref;

use crate::{
    vk,
    prelude::VkResult,
};

use super::Device;

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkInstance.html>
#[derive(Clone)]
pub struct Instance {
    pub(crate) instance: ash::Instance,
}

impl Deref for Instance {

    type Target = ash::Instance;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl Instance {
    
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateDevice.html>
    ///
    /// # Safety
    /// There is a parent/child relation between [`Instance`] and the resulting [`Device`].
    /// The application must not destroy the parent [`Instance`] object before first destroying the
    /// returned [`Device`] child object. [`Device`] does not implement drop semantics and can only be
    /// destroyed via [`Device::destroy_device()`].
    /// See the [`Entry::create_instance()`] documentation for more destruction ordering rules on Instance.
    #[inline(always)]
    pub unsafe fn create_device(
        &self,
        physical_device: vk::PhysicalDevice,
        create_info: &vk::DeviceCreateInfo<'_>,
        allocation_callbacks: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<Device> {
        let device = unsafe {
            self.instance.create_device(physical_device, create_info, allocation_callbacks)?
        };
        Ok(Device {
            device,
        })
    }
}
