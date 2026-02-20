use super::*;

use core::ops::{Deref, DerefMut};

pub struct GpuContext
{
    vk: Arc<Vulkan>,
    resources: Arc<Resources>,
    transfer_requests: &'a mut TransferRequests,
    memory_layout: &'a MemoryLayout,
    buffered_frames: u32,
}

impl<'a> GpuContext<'a> 
{

    #[inline(always)]
    pub(super) fn new(
        vk: &'a Arc<Vulkan>,
        resources: RwLockWriteGuard<'a, Resources>,
        transfer_requests: &'a mut TransferRequests,
        memory_layout: &'a MemoryLayout,
        buffered_frames: u32,
    ) -> Self
    {
        Self {
            resources,
            vk,
            transfer_requests,
            memory_layout,
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
        staging_binder: LinearBinderId,
        signal_semaphores: &[(TimelineSemaphoreId, u64)]
    ) -> CommandRequestId
    {
        self.transfer_requests.add_async_request(staging_binder, signal_semaphores)
    } 

    #[inline(always)]
    pub fn buffered_frames(&self) -> u32 {
        self.buffered_frames
    }

    #[inline(always)]
    pub(crate) fn create_surface(
        &self,
        window: &win::WinitWindow,
    ) -> Result<Surface>
    {
        Surface::new(
            window,
            self.vk.clone(),
            self.buffered_frames,
            *self.memory_layout,
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
