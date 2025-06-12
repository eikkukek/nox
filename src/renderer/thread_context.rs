use crate::string_types::{array_format, SmallError};

use super::{
    physical_device,
    helpers::{self},
};

use ash::vk;

pub struct ThreadContext {
    device: ash::Device,
    graphics_pool: vk::CommandPool,
    transfer_pool: vk::CommandPool,
    compute_pool: vk::CommandPool,
}

impl ThreadContext {

    pub fn new(
        device: &ash::Device,
        queue_families: &physical_device::QueueFamilyIndices,
    ) -> Result<Self, SmallError> {
        let flags = vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        let graphics_pool =
            helpers::create_command_pool(&device, flags, queue_families.get_graphics_index())
            .map_err(|e|
                array_format!("failed to create command pool {:?}", e)
            )?;
        let transfer_pool =
            helpers::create_command_pool(&device, flags, queue_families.get_graphics_index())
            .map_err(|e| {
                unsafe { device.destroy_command_pool(graphics_pool, None); }
                array_format!("failed to create command pool {:?}", e)
            })?;
        let compute_pool =
            helpers::create_command_pool(&device, flags, queue_families.get_graphics_index())
            .map_err(|e| {
                unsafe { device.destroy_command_pool(graphics_pool, None); }
                unsafe { device.destroy_command_pool(transfer_pool, None); }
                array_format!("failed to create command pool {:?}", e)
            })?;
        Ok(
            Self {
                device: device.clone(),
                graphics_pool,
                transfer_pool,
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

    pub fn _compute_pool(&self) -> vk::CommandPool {
        self.compute_pool
    }
}

impl Drop for ThreadContext {

    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.graphics_pool, None);
            self.device.destroy_command_pool(self.transfer_pool, None);
            self.device.destroy_command_pool(self.compute_pool, None);
        }
    }
}
