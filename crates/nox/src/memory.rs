use super::renderer;

pub struct Memory {
    renderer_layout: renderer::MemoryLayout,
    renderer_allocators: renderer::Allocators,
}

impl Memory {

    pub fn renderer_layout(&self) -> &renderer::MemoryLayout {
        &self.renderer_layout
    }

    pub fn renderer_allocators(&self) -> &renderer::Allocators {
        &self.renderer_allocators
    }
}

impl Default for Memory {

    fn default() -> Self {
        let renderer_layout = renderer::MemoryLayout::default();
        let renderer_allocators = renderer::Allocators::new(renderer_layout).unwrap();
        Self {
            renderer_layout,
            renderer_allocators,
        }
    }
}
