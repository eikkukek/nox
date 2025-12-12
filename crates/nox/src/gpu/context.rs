use super::*;

use core::ops::{Deref, DerefMut};

use std::sync::RwLockWriteGuard;

pub struct GpuContext<'a> {
    pub(super) global_resources: RwLockWriteGuard<'a, GlobalResources>,
    pub(super) transfer_requests: &'a mut TransferRequests,
    pub(super) frame_buffer_size: Dimensions,
}

impl<'a> GpuContext<'a> {

    #[inline(always)]
    pub(super) fn new(
        global_resources: RwLockWriteGuard<'a, GlobalResources>,
        transfer_requests: &'a mut TransferRequests,
        frame_buffer_size: Dimensions,
    ) -> Self {
        Self {
            global_resources,
            transfer_requests,
            frame_buffer_size,
        }
    }

    #[inline(always)]
    pub fn add_async_transfer_request(
        &mut self,
        staging_alloc: LinearDeviceAllocId,
        signal_semaphores: &[(TimelineSemaphoreId, u64)]
    ) -> CommandRequestId
    {
        self.transfer_requests.add_async_request(staging_alloc, signal_semaphores)
    }

    #[inline(always)]
    pub fn frame_buffer_size(&self) -> image::Dimensions {
        self.frame_buffer_size
    }
}

impl<'a> Deref for GpuContext<'a> {

    type Target = GlobalResources;

    fn deref(&self) -> &Self::Target {
        &self.global_resources
    }
}

impl<'a> DerefMut for GpuContext<'a> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.global_resources
    }
}
