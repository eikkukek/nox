use std::sync::Arc;

use super::{
    physical_device,
    helpers::{self},
    RaiiHandle,
};

use ash::vk;

pub struct ThreadContext {
    device: Arc<ash::Device>,
    graphics_pool: vk::CommandPool,
    transfer_pool: vk::CommandPool,
    compute_pool: vk::CommandPool,
}

impl ThreadContext {

    pub fn new(
        device: Arc<ash::Device>,
        queue_families: physical_device::QueueFamilyIndices,
    ) -> Result<Self, vk::Result> {
        let flags = vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        let graphics_pool = RaiiHandle::new(
            helpers::create_command_pool(&device, flags, queue_families.graphics_index())?,
            |v| unsafe { device.destroy_command_pool(v, None); }
        );
        let transfer_pool = RaiiHandle::new(
            helpers::create_command_pool(&device, flags, queue_families.transfer_index())?,
            |v| unsafe { device.destroy_command_pool(v, None); }
        );
        let compute_pool =
            helpers::create_command_pool(&device, flags, queue_families.compute_index())?;
        Ok(
            Self {
                device: device.clone(),
                graphics_pool: graphics_pool.into_inner(),
                transfer_pool: transfer_pool.into_inner(),
                compute_pool,
            }
        )
    }

    pub fn graphics_pool(&self) -> vk::CommandPool {
        self.graphics_pool
    }

    pub fn transfer_pool(&self) -> vk::CommandPool {
        self.transfer_pool
    }

    pub fn compute_pool(&self) -> vk::CommandPool {
        self.compute_pool
    }
}

impl Drop for ThreadContext {

    fn drop(&mut self) {
        unsafe {
            let device = &*self.device;
            device.destroy_command_pool(self.graphics_pool, None);
            device.destroy_command_pool(self.transfer_pool, None);
            device.destroy_command_pool(self.compute_pool, None);
        }
    }
}
