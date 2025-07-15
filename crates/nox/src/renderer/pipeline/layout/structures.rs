use nox_mem::{AsRaw, GlobalVec};

use crate::byte_hash::ByteHash;

use super::*;

#[derive(Clone, Copy)]
pub struct DescriptorBindingInfo {
    binding: u32,
    descriptor_type: DescriptorType,
    descriptor_count: u32,
    shader_stage: ShaderStage,
}

impl ByteHash for DescriptorBindingInfo {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.binding.byte_hash(hasher);
        self.descriptor_type.as_raw().byte_hash(hasher);
        self.descriptor_count.byte_hash(hasher);
        self.shader_stage.as_raw().byte_hash(hasher);
    }
}

#[derive(Clone)]
pub struct DescriptorSetLayoutInfo {
    bindings: GlobalVec<DescriptorBindingInfo>,
}

impl ByteHash for DescriptorSetLayoutInfo {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.bindings.byte_hash(hasher);
    }
}

#[derive(Clone, Copy)]
pub struct PushConstantRange {
    shader_stage: ShaderStage,
    offset: u32,
    size: u32,
}

impl ByteHash for PushConstantRange {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.shader_stage.as_raw().byte_hash(hasher);
        self.offset.byte_hash(hasher);
        self.size.byte_hash(hasher);
    }
}
