use std::sync::{Arc, RwLock};

use ash::vk;

use crate::renderer::{
    global_resources::{GlobalResources, ImageSourceID, ImageSubresourceID},
    ImageState,
    MSAA,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ResourceID {
    pub(crate) id: ImageSourceID,
    pub(crate) format: vk::Format,
    pub(crate) samples: MSAA,
    pub(crate) is_transient: bool,
}

impl ResourceID {
    
    #[inline(always)]
    pub(crate) fn vk_format(&self) -> vk::Format {
        self.format
    }

    #[inline(always)]
    pub(crate) fn samples(&self) -> MSAA {
        self.samples
    }
}

pub(crate) struct SubresourceResetGuard {
    pub(crate) resources: Arc<RwLock<GlobalResources>>,
    pub(crate) command_buffer: vk::CommandBuffer,
    pub(crate) id: ImageSubresourceID,
    pub(crate) dst_state: ImageState,
}

impl Drop for SubresourceResetGuard {

    fn drop(&mut self) {
        if let Ok(v) = self.resources
            .write()
            .unwrap()
            .get_mut_image_subresource(self.id)
        {
            v.cmd_memory_barrier(self.dst_state, self.command_buffer);
        }
    }
}
