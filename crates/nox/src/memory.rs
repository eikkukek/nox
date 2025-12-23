use nox_alloc::arena_alloc::ArenaAlloc;

use super::gpu;

pub struct GpuMemory {
    layout: gpu::MemoryLayout,
    host_allocators: gpu::HostAllocators,
}

impl GpuMemory {

    pub fn layout(&self) -> &gpu::MemoryLayout {
        &self.layout
    }

    pub fn host_allocators(&self) -> &gpu::HostAllocators {
        &self.host_allocators
    }
}

impl Default for GpuMemory {

    fn default() -> Self {
        let layout = gpu::MemoryLayout::default();
        let host_allocators = gpu::HostAllocators::new(layout).unwrap();
        Self {
            layout,
            host_allocators,
        }
    }
}

pub struct Memory {
    gpu: GpuMemory,
    tmp_alloc: ArenaAlloc,
}

impl Memory {

    pub fn gpu(&self) -> &GpuMemory {
        &self.gpu
    }

    pub fn tmp_alloc(&self) -> &ArenaAlloc {
        &self.tmp_alloc
    }
}

impl Default for Memory {

    fn default() -> Self {
        Self {
            gpu: Default::default(),
            tmp_alloc: ArenaAlloc
                ::new(1 << 16)
                .unwrap(),
        }
    }
}
