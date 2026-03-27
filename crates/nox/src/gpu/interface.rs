use super::*;

pub struct DeviceLimits<'a> {
    pub(super) limits: &'a vk::PhysicalDeviceLimits,
}

impl DeviceLimits<'_> {

    #[inline(always)]
    pub fn max_viewports(&self) -> u32 {
        self.limits.max_viewports
    }

    #[inline(always)]
    pub fn max_push_constant_size(&self) -> u32 {
        self.limits.max_push_constants_size
    }

    #[inline(always)]
    pub fn min_uniform_buffer_offset_alignment(&self) -> DeviceSize {
        self.limits.min_uniform_buffer_offset_alignment
    }
}
