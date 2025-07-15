use blake3;

use nox_mem::{GlobalVec, Vector};

use crate::byte_hash::ByteHash;

use super::*;

#[derive(Clone)]
pub struct PipelineLayoutInfo {
    set_layouts: GlobalVec<DescriptorSetLayoutInfo>,
    push_constant_ranges: GlobalVec<PushConstantRange>,
}

impl PipelineLayoutInfo {

    pub fn with_set_layout(&mut self, info: DescriptorSetLayoutInfo) -> &mut Self {
        self.set_layouts.push(info).unwrap();
        self
    }

    pub fn with_push_constant_range(&mut self, range: PushConstantRange) -> &mut Self {
        self.push_constant_ranges.push(range).unwrap();
        self
    }
}

impl ByteHash for PipelineLayoutInfo {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.set_layouts.byte_hash(hasher);
    }
}
