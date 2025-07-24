use ash::vk::{self, Handle};

use nox_mem::{GlobalVec, Vector};

use crate::{
    renderer::Error,
    byte_hash::ByteHash,
};

use super::*;

#[derive(Clone)]
pub struct PipelineLayoutInfo {
    set_layouts: GlobalVec<vk::DescriptorSetLayout>,
    push_constant_ranges: GlobalVec<vk::PushConstantRange>,
    hasher: blake3::Hasher,
}

impl PipelineLayoutInfo {

    pub fn new(
        set_capacity: u32,
        push_constant_capacity: u32,
    ) -> Self
    {
        Self {
            set_layouts: GlobalVec::with_capacity(set_capacity as usize).unwrap(),
            push_constant_ranges: GlobalVec::with_capacity(push_constant_capacity as usize).unwrap(),
            hasher: blake3::Hasher::new(),
        }
    }

    pub fn with_set_layout(mut self, handle: vk::DescriptorSetLayout) -> Self {
        self.set_layouts.push(handle).unwrap();
        handle.as_raw().byte_hash(&mut self.hasher);
        self
    }

    pub fn _with_push_constant_range(mut self, range: PushConstantRange) -> Self {
        self.push_constant_ranges.push(range.into()).unwrap();
        range.byte_hash(&mut self.hasher);
        self
    }

    pub fn build(&self, device: &ash::Device) -> Result<vk::PipelineLayout, Error> {
        let create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            set_layout_count: self.set_layouts.len() as u32,
            p_set_layouts: self.set_layouts.as_ptr(),
            push_constant_range_count: self.push_constant_ranges.len() as u32,
            p_push_constant_ranges: self.push_constant_ranges.as_ptr(),
            ..Default::default()
        };
        unsafe {
            Ok(device.create_pipeline_layout(&create_info, None)?)
        }
    }
}

impl ByteHash for PipelineLayoutInfo {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        let hash1 = hasher.clone().finalize();
        let hash2 = self.hasher.clone().finalize();
        *hasher = blake3::Hasher::new();
        hasher.update(b"combine_v1{");
        hasher.update(hash1.as_bytes());
        hasher.update(b"}{");
        hasher.update(hash2.as_bytes());
        hasher.update(b"}");
    }
}
