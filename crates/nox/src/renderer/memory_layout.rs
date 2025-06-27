use ash::vk;

#[derive(Clone, Copy)]
pub struct MemoryLayout {
    init_size: usize,
    swapchain_size: usize,
    device_local_size: vk::DeviceSize,
    device_staging_size: vk::DeviceSize,
    device_uniform_size: vk::DeviceSize,
    device_frame_size: vk::DeviceSize,
}

impl MemoryLayout {

    pub fn default() -> Self {
        Self {
            init_size: 1 << 18,
            swapchain_size: 1 << 18,
            device_local_size: 256_000_000,
            device_staging_size: 64_000_000,
            device_uniform_size: 64_000_000,
            device_frame_size: 1 << 27,
        }
    }

    pub fn init_size(&self) -> usize {
        self.init_size
    }

    pub fn swapchain_size(&self) -> usize {
        self.swapchain_size
    }

    pub fn device_local_size(&self) -> vk::DeviceSize {
        self.device_local_size
    }

    pub fn device_staging_size(&self) -> vk::DeviceSize {
        self.device_staging_size
    }

    pub fn device_uniform_size(&self) -> vk::DeviceSize {
        self.device_uniform_size
    }

    pub fn device_frame_size(&self) -> vk::DeviceSize {
        self.device_frame_size
    }
}
