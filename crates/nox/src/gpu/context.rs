use super::*;

use core::ops::{Deref, DerefMut};

use std::sync::RwLockWriteGuard;

pub struct GpuContext<'a>
{
    vk: &'a Arc<Vulkan>,
    resources: RwLockWriteGuard<'a, Resources>,
    transfer_requests: &'a mut TransferRequests,
    buffered_frames: u32,
}

impl<'a> GpuContext<'a> 
{

    #[inline(always)]
    pub(super) fn new(
        vk: &'a Arc<Vulkan>,
        resources: RwLockWriteGuard<'a, Resources>,
        transfer_requests: &'a mut TransferRequests,
        buffered_frames: u32,
    ) -> Self
    {
        Self {
            resources,
            vk,
            transfer_requests,
            buffered_frames,
        }
    }

    #[inline(always)]
    pub fn is_layer_enabled(&self, layer: Layer) -> bool {
        self.vk.is_layer_enabled(layer)
    }

    #[inline(always)]
    pub fn is_extension_enabled(&self, extension: Extension) -> bool {
        self.vk.is_extension_enabled(extension)
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
    pub fn buffered_frames(&self) -> u32 {
        self.buffered_frames
    }

    #[inline(always)]
    pub(crate) fn create_surface<'mem>(
        &self,
        window: &win::WinitWindow,
        host_allocators: &'mem HostAllocators,
    ) -> Result<Surface<'mem>>
    {
        Surface::new(
            window,
            self.vk.clone(),
            self.buffered_frames,
            host_allocators,
        )
    }
}

impl<'a> Deref for GpuContext<'a> {

    type Target = Resources;

    fn deref(&self) -> &Self::Target {
        &self.resources
    }
}

impl<'a> DerefMut for GpuContext<'a> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resources
    }
}
