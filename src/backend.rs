use super::{
    renderer,
    nox,
};

pub struct Memory {
    nox_layout: nox::MemoryLayout,
    nox_allocators: nox::Allocators,
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
                nox_layout,
                nox_allocators,
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

pub struct Backend<'mem> {
    pub memory: &'mem mut Memory,
}

impl<'mem> Backend<'mem> {

    pub fn new(
        memory: &'mem mut Memory,
    ) -> Option<Self>
    {
        Some(
            Backend {
                memory,
            }
        )
    }
}

impl<'i> Drop for Backend<'i> {

    fn drop(&mut self) {
        println!("Nox backend message: terminating backend");
    }
}
