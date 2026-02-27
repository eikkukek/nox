//! <https://docs.vulkan.org/refpages/latest/refpages/source/VK_KHR_present_wait2.html>
//!
//! VK_KHR_present_wait2 device extension.

use {
    core::ffi::{CStr, c_void},
    crate::{
        vk,
        prelude::VkResult,
    },
    nox_mem::result::ResultExt,
};

pub use {
    vk::KHR_PRESENT_WAIT_2_NAME as NAME,
    vk::KHR_PRESENT_WAIT_2_SPEC_VERSION as SPEC_VERSION,
};

/// Raw VK_KHR_present_wait2 device-level function pointers.
#[derive(Clone)]
pub struct DeviceFn {
    pub wait_for_present2_khr: vk::PFN_vkWaitForPresent2KHR,
}

impl DeviceFn {

    pub fn load<F: FnMut(&CStr) -> *const c_void>(mut f: F) -> Self {
        Self {
            wait_for_present2_khr: unsafe {
                unsafe extern "system" fn wait_for_present2_khr(
                    _device: vk::Device,
                    _swapchain: vk::SwapchainKHR,
                    _p_present_wait2_info: *const vk::PresentWait2InfoKHR,
                ) -> vk::Result {
                    panic!("Unable to load wait_for_present2_khr")
                }
                let cname = c"vkWaitForPresent2KHR";
                let val = f(cname);
                if val.is_null() {
                    wait_for_present2_khr
                } else {
                    core::mem::transmute::<
                        *const c_void,
                        vk::PFN_vkWaitForPresent2KHR,
                    >(val)
                }
            }
        }
    }
}

unsafe impl Send for DeviceFn {}
unsafe impl Sync for DeviceFn {}

/// VK_KHR_present_wait2 device-level functions.
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

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkWaitForPresent2KHR.html>
    ///
    /// [`Ok`] values are:
    ///
    /// [`vk::Result::SUBOPTIMAL_KHR`]
    /// [`vk::Result::SUCCESS`]
    /// [`vk::Result::TIMEOUT`]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn wait_for_present2_khr(
        &self,
        swapchain: vk::SwapchainKHR,
        p_present_wait2_info: &vk::PresentWait2InfoKHR,
    ) -> VkResult<vk::Result> {
        unsafe {
            (self.fp.wait_for_present2_khr)(
                self.handle,
                swapchain,
                p_present_wait2_info,
            )
        }.result_with_success(vk::Result::SUCCESS)
        .filter_err(|&err| matches!(
            err,
            vk::Result::SUBOPTIMAL_KHR | vk::Result::TIMEOUT,
        ).then_some(err))
    }
}
