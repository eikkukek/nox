use super::{
    renderer,
    stack_allocator::StackMemory,
};

pub struct Memory<'mem> {
    _pool: StackMemory,
    renderer_memory: renderer::Memory<'mem>,
}

impl<'mem> Memory<'mem> {

    pub fn default() -> Option<Self> {
        let renderer_layout = renderer::MemoryLayout::default();
        let mut pool = StackMemory::new(renderer_layout.alloc_size())?;
        let renderer_memory = renderer::Memory::new(renderer_layout, &mut pool)?;
        Some(
            Self {
                _pool: pool,
                renderer_memory,
            }
        )
    }
}

pub struct Backend<'mem> {
    memory: Memory<'mem>,
}

impl<'mem> Backend<'mem> {

    pub fn new(
        memory: Memory<'mem>,
    ) -> Option<Self>
    {
        Some(
            Backend {
                memory,
            }
        )
    }

    pub fn renderer_memory(&mut self) -> &mut renderer::Memory<'mem> {
        &mut self.memory.renderer_memory
    }
}

impl<'mem> Drop for Backend<'mem> {

    fn drop(&mut self) {
        println!("Nox backend message: terminating backend");
    }
}
