use super::renderer;

pub struct Memory {
    renderer_layout: renderer::MemoryLayout,
    renderer_allocators: renderer::Allocators,
}

impl Memory {

    pub fn default() -> Option<Self> {
        let renderer_layout = renderer::MemoryLayout::default();
        let renderer_allocators = renderer::Allocators::new(renderer_layout)?;
        Some(
            Self {
                renderer_layout,
                renderer_allocators,
            }
        )
    }

    pub fn renderer_layout(&self) -> &renderer::MemoryLayout {
        &self.renderer_layout
    }

    pub fn renderer_allocators(&self) -> &renderer::Allocators {
        &self.renderer_allocators
    }
}
