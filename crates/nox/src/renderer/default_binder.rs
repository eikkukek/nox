use std::sync::Arc;

use ash::vk;

use super::memory_binder::{DeviceMemory, MemoryBinder}

pub struct Memory {
    device: Arc<ash::Device>,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
}

impl DeviceMemory for Memory {

    fn device_memory(&self) -> vk::DeviceMemory {
        self.memory
    }

    fn offset(&self) -> vk::DeviceSize {
        0
    }

    fn size(&self) -> vk::DeviceSize {
        self.size
    }

    unsafe fn free_memory(&mut self) {
        unsafe {
            self.device.free_memory(self.memory, None);
        }
    }
}

pub struct DefaultBinder {}

impl MemoryBinder for DefaultBinder {

    type Memory = Memory;

    fn bind_image_memory(&mut self, image: vk::Image) -> Result<Self::Memory, super::Error> {

    }
}
