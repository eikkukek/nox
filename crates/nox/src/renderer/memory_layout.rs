use ash::vk;

#[derive(Clone, Copy)]
pub struct MemoryLayout {
    tmp_arena_size: usize,
    swapchain_arena_size: usize,
    frame_graph_arena_size: usize,
    frame_graph_device_block_size: vk::DeviceSize,
    max_descriptor_sets: u32,
    max_uniform_sampled_images: u32,
    max_uniform_buffers: u32,
    max_uniform_storage_buffers: u32,
    max_uniform_storage_images: u32,
}

impl MemoryLayout {

    pub fn default() -> Self {
        Self {
            tmp_arena_size: 1 << 16,
            swapchain_arena_size: 1 << 18,
            frame_graph_arena_size: 1 << 18,
            frame_graph_device_block_size: 1 << 29,
            max_descriptor_sets: 1024,
            max_uniform_sampled_images: 4096,
            max_uniform_buffers: 2048,
            max_uniform_storage_buffers: 1024,
            max_uniform_storage_images: 1024,
        }
    }

    pub fn tmp_arena_size(&self) -> usize {
        self.tmp_arena_size
    }

    pub fn swapchain_size(&self) -> usize {
        self.swapchain_arena_size
    }

    pub fn frame_graph_arena_size(&self) -> usize {
        self.frame_graph_arena_size
    }

    pub fn frame_graph_device_block_size(&self) -> vk::DeviceSize {
        self.frame_graph_device_block_size
    }

    pub fn max_descriptor_sets(&self) -> u32 {
        self.max_descriptor_sets
    }

    pub fn max_uniform_sampled_images(&self) -> u32 {
        self.max_uniform_sampled_images
    }

    pub fn max_uniform_buffers(&self) -> u32 {
        self.max_uniform_buffers
    }

    pub fn max_uniform_storage_buffers(&self) -> u32 {
        self.max_uniform_storage_buffers
    }

    pub fn max_uniform_storage_images(&self) -> u32 {
        self.max_uniform_storage_images
    }
}
