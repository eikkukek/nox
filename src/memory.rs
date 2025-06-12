use super::{
    renderer,
    nox,
};

pub struct Memory {
    _nox_layout: nox::MemoryLayout,
    _nox_allocators: nox::Allocators,
    renderer_layout: renderer::MemoryLayout,
    renderer_allocators: renderer::Allocators,
}

impl Memory {

    pub fn default() -> Option<Self> {
        let nox_layout = nox::MemoryLayout::default();
        let nox_allocators = nox::Allocators::new(nox_layout)?;
        let renderer_layout = renderer::MemoryLayout::default();
        let renderer_allocators = renderer::Allocators::new(renderer_layout)?;
        Some(
            Self {
                _nox_layout: nox_layout,
                _nox_allocators: nox_allocators,
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
