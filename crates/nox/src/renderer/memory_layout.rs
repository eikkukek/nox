use ash::vk;

#[derive(Clone, Copy)]
pub struct MemoryLayout {
    init_size: usize,
    tmp_size: usize,
    swapchain_size: usize,
    frame_graphs_size: usize,
    device_frame_size: vk::DeviceSize,
    device_staging_size: vk::DeviceSize,
    max_descriptor_sets: u32,
    uniform_sampled_images: u32,
    uniform_buffers: u32,
    uniform_storage_buffers: u32,
}

impl MemoryLayout {

    pub fn default() -> Self {
        Self {
            init_size: 1 << 18,
            tmp_size: 1 << 16,
            swapchain_size: 1 << 18,
            frame_graphs_size: 1 << 18,
            device_frame_size: 1 << 27,
            device_staging_size: 1 << 27,
            max_descriptor_sets: 1024,
            uniform_sampled_images: 4096,
            uniform_buffers: 2048,
            uniform_storage_buffers: 1024,
        }
    }

    pub fn init_size(&self) -> usize {
        self.init_size
    }

    pub fn tmp_size(&self) -> usize {
        self.tmp_size
    }

    pub fn swapchain_size(&self) -> usize {
        self.swapchain_size
    }

    pub fn frame_graphs_size(&self) -> usize {
        self.frame_graphs_size
    }

    pub fn device_frame_size(&self) -> vk::DeviceSize {
        self.device_frame_size
    }

    pub fn device_staging_size(&self) -> vk::DeviceSize {
        self.device_staging_size
    }

    pub fn max_descriptor_sets(&self) -> u32 {
        self.max_descriptor_sets
    }

    pub fn uniform_sampled_images(&self) -> u32 {
        self.uniform_sampled_images
    }

    pub fn uniform_buffers(&self) -> u32 {
        self.uniform_buffers
    }

    pub fn uniform_storage_buffers(&self) -> u32 {
        self.uniform_storage_buffers
    }
}
