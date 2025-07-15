use ash::vk;

use nox_mem::{Allocator, GlobalVec, FixedVec};

pub struct PipelineTypeInfo<'alloc, Alloc: Allocator> {
    msaa_samples: vk::SampleCountFlags,
    color_formats: FixedVec<'alloc, vk::Format, Alloc>,
    depth_format: vk::Format,
    stencil_format: vk::Format,
}

impl<'alloc, Alloc: Allocator> PipelineTypeInfo<'alloc, Alloc> {

    pub fn msaa_samples(&self) -> vk::SampleCountFlags {
        self.msaa_samples
    }

    pub fn color_formats(&self) -> &FixedVec<'alloc, vk::Format, Alloc> {
        &self.color_formats
    }

    pub fn depth_format(&self) -> vk::Format {
        self.depth_format
    }

    pub fn stencil_format(&self) -> vk::Format {
        self.stencil_format
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PipelineID {
    type_index: u32,
    pipeline_index: u32,
}

impl PipelineID {

    pub fn type_index(&self) -> u32 {
        self.type_index
    }

    pub fn pipeline_index(&self) -> u32 {
        self.pipeline_index
    }
}

pub struct PipelineCache<'alloc, Alloc: Allocator> {
    pipeline_types: GlobalVec<PipelineTypeInfo<'alloc, Alloc>>,
}

impl<'alloc, Alloc: Allocator> PipelineCache<'alloc, Alloc> {

    pub fn new() -> Self {
        Self {
            pipeline_types: GlobalVec::new(),
        }
    }

    pub fn get_type_info(&self, id: PipelineID) -> &PipelineTypeInfo<'alloc, Alloc> {
        assert!(id.type_index < self.pipeline_types.len() as u32, "invalid pipeline id {:?}", id);
        &self.pipeline_types[id.type_index as usize]
    }
}
