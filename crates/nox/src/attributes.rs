use crate::gpu;

pub struct Attributes {
    pub(crate) buffered_frames: u32,
    pub(crate) gpu_memory_layout: gpu::MemoryLayout,
    pub(crate) gpu_cache_attributes: gpu::CacheAttributes,
}

impl Attributes {

    pub(crate) fn new() -> Self
    {
        Attributes {
            buffered_frames: 3,
            gpu_memory_layout: Default::default(),
            gpu_cache_attributes: Default::default(),
        }
    }

    #[inline]
    pub fn with_gpu_memory_layout(mut self, layout: gpu::MemoryLayout) -> Self {
        self.gpu_memory_layout = layout;
        self
    }

    #[inline]
    pub fn with_buffered_frames(mut self, frames: u32) -> Self {
        self.buffered_frames = frames;
        self
    }

    #[inline]
    pub fn with_gpu_cache_attributes(mut self, attributes: gpu::CacheAttributes) -> Self {
        self.gpu_cache_attributes = attributes;
        self
    }
}
