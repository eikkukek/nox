use ash::vk;

use nox_mem::{slot_map::{SlotIndex, GlobalSlotMap}, vec_types::{GlobalVec}};

pub struct PipelineTypeInfo {
    msaa_samples: vk::SampleCountFlags,
    color_formats: GlobalVec<vk::Format>,
    depth_format: vk::Format,
    stencil_format: vk::Format,
}

impl PipelineTypeInfo {

    pub fn msaa_samples(&self) -> vk::SampleCountFlags {
        self.msaa_samples
    }

    pub fn color_formats(&self) -> &[vk::Format] {
        &self.color_formats
    }

    pub fn depth_format(&self) -> vk::Format {
        self.depth_format
    }

    pub fn stencil_format(&self) -> vk::Format {
        self.stencil_format
    }
}

#[derive(Clone, Copy)]
pub struct PipelineID {
    type_index: SlotIndex<PipelineTypeInfo>,
}

impl PipelineID {
    
    pub fn type_index(&self) -> SlotIndex<PipelineTypeInfo> {
        self.type_index
    }
}

pub struct PipelineCache {
    pipeline_types: GlobalSlotMap<PipelineTypeInfo>,
}

impl PipelineCache {

    pub fn new() -> Self {
        Self {
            pipeline_types: GlobalSlotMap::new(),
        }
    }

    pub fn get_type_info(&self, id: PipelineID) -> &PipelineTypeInfo {
        &self.pipeline_types[id.type_index]
    }
}
