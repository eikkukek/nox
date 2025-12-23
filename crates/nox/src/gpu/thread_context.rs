use std::sync::Arc;

use super::{
    Vulkan,
    helpers::{self},
    RaiiHandle,
};

use ash::vk;

pub struct ThreadContext {
    vk: Arc<Vulkan>,
    graphics_pool: vk::CommandPool,
    transfer_pool: vk::CommandPool,
    compute_pool: vk::CommandPool,
}

impl ThreadContext {

    pub fn new(
        vk: Arc<Vulkan>,
    ) -> Result<Self, vk::Result> {
        let flags = vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        let device = vk.device();
        let queue_family_indices = vk.queue_family_indices();
        let graphics_pool = RaiiHandle::new(
            helpers::create_command_pool(device,
                flags, queue_family_indices.graphics_index()
            )?,
            |v| unsafe { vk.device().destroy_command_pool(v, None); }
        );
        let transfer_pool = RaiiHandle::new(
            helpers::create_command_pool(device,
                flags, queue_family_indices.transfer_index()
            )?,
            |v| unsafe { device.destroy_command_pool(v, None); }
        );
        let compute_pool =
            helpers::create_command_pool(&device, flags, vk.queue_family_indices().compute_index())?;
        Ok(
            Self {
                vk: vk.clone(),
                graphics_pool: graphics_pool.into_inner(),
                transfer_pool: transfer_pool.into_inner(),
                compute_pool,
            }
        )
    }

    pub fn graphics_pool(&self) -> vk::CommandPool {
        self.graphics_pool
    }

    pub fn _transfer_pool(&self) -> vk::CommandPool {
        self.transfer_pool
    }

    pub fn compute_pool(&self) -> vk::CommandPool {
        self.compute_pool
    }
}

impl Drop for ThreadContext {

    fn drop(&mut self) {
        unsafe {
            let device = self.vk.device();
            device.destroy_command_pool(self.graphics_pool, None);
            device.destroy_command_pool(self.transfer_pool, None);
            device.destroy_command_pool(self.compute_pool, None);
        }
    }
}
