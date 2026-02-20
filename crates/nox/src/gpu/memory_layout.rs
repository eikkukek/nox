#[derive(Clone, Copy)]
pub struct MemoryLayout {
    tmp_arena_size: usize,
    swapchain_arena_size: usize,
    frame_graph_arena_size: usize,
}

impl Default for MemoryLayout {

    fn default() -> Self {
        Self {
            tmp_arena_size: 1 << 16,
            swapchain_arena_size: 1 << 18,
            frame_graph_arena_size: 1 << 18,
        }
    }
}

impl MemoryLayout {

    #[inline(always)]
    pub fn tmp_arena_size(&self) -> usize {
        self.tmp_arena_size
    }

    #[inline(always)]
    pub fn swapchain_size(&self) -> usize {
        self.swapchain_arena_size
    }

    #[inline(always)]
    pub fn frame_graph_arena_size(&self) -> usize {
        self.frame_graph_arena_size
    }
}
