use crate::string::{String, SmallError};

use super::{
    physical_device,
    helpers,
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
        device: ash::Device,
        queue_families: &physical_device::QueueFamilyIndices,
    ) -> Result<Self, SmallError> {
        let flags = vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        let graphics_pool =
            helpers::create_command_pool(&device, flags, queue_families.get_graphics_index())
            .map_err(|e| {
                String::format(format_args!(
                    "failed to create command pool {:?}", e
                ))
            })?;
        let transfer_pool =
            helpers::create_command_pool(&device, flags, queue_families.get_graphics_index())
            .map_err(|e| {
                unsafe { device.destroy_command_pool(graphics_pool, None); }
                String::format(format_args!(
                    "failed to create command pool {:?}", e
                ))
            })?;
        let compute_pool =
            helpers::create_command_pool(&device, flags, queue_families.get_graphics_index())
            .map_err(|e| {
                unsafe { device.destroy_command_pool(graphics_pool, None); }
                unsafe { device.destroy_command_pool(transfer_pool, None); }
                String::format(format_args!(
                    "failed to create command pool {:?}", e
                ))
            })?;
        Ok(
            Self {
                device,
                graphics_pool,
                transfer_pool,
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

    pub fn destroy(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.graphics_pool, None);
            self.device.destroy_command_pool(self.transfer_pool, None);
            self.device.destroy_command_pool(self.compute_pool, None);
        }
    }
}
