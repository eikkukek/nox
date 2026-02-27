use super::*;

pub struct DeviceLimits<'a> {
    pub(super) limits: &'a vk::PhysicalDeviceLimits,
}

impl DeviceLimits<'_> {

    #[inline(always)]
    pub fn max_viewports(&self) -> u32 {
        self.limits.max_viewports
    }
}
