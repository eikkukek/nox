use ash::vk;

use nox_mem::{AsRaw, GlobalVec, Vector};

use crate::byte_hash::ByteHash;

use super::*;

#[derive(Clone, Copy)]
pub struct DescriptorBindingInfo {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub shader_stage: ShaderStage,
}

impl DescriptorBindingInfo {

    pub fn new(
        binding: u32,
        descriptor_type: DescriptorType,
        descriptor_count: u32,
        shader_stage: ShaderStage
    ) -> Self
    {
        Self {
            binding,
            descriptor_type,
            descriptor_count,
            shader_stage,
        }
    }
}

impl<'a> From<DescriptorBindingInfo> for vk::DescriptorSetLayoutBinding<'a> {

    fn from(value: DescriptorBindingInfo) -> Self {
        Self {
            binding: value.binding,
            descriptor_type: value.descriptor_type.into(),
            descriptor_count: value.descriptor_count,
            stage_flags: value.shader_stage.into(),
            ..Default::default()
        }
    }
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
    bindings: GlobalVec<vk::DescriptorSetLayoutBinding<'static>>,
    hasher: blake3::Hasher,
}

impl DescriptorSetLayoutInfo {

    pub fn new(binding_capacity: u32) -> Self {
        Self {
            bindings: GlobalVec::with_capacity(binding_capacity as usize).unwrap(),
            hasher: blake3::Hasher::new(),
        }
    }

    pub fn with_binding(&mut self, binding: DescriptorBindingInfo) -> &mut Self {
        self.bindings.push(binding.into()).unwrap();
        binding.byte_hash(&mut self.hasher);
        self
    }

    pub fn build(&self, device: &ash::Device) -> Result<vk::DescriptorSetLayout, vk::Result> {
        let create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            binding_count: self.bindings.len() as u32,
            p_bindings: self.bindings.as_ptr(),
            ..Default::default()
        };
        unsafe {
            device.create_descriptor_set_layout(&create_info, None)
        }
    }
}

impl ByteHash for DescriptorSetLayoutInfo {

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

#[derive(Clone, Copy)]
pub struct PushConstantRange {
    pub shader_stage: ShaderStage,
    pub offset: u32,
    pub size: u32,
}

impl PushConstantRange {

    pub fn new(
        shader_stage: ShaderStage,
        offset: u32,
        size: u32,
    ) -> Self
    {
        Self {
            shader_stage,
            offset,
            size,
        }
    }
}

impl From<PushConstantRange> for vk::PushConstantRange {

    fn from(value: PushConstantRange) -> Self {
        Self {
            stage_flags: value.shader_stage.into(),
            offset: value.offset,
            size: value.size,
        }
    }
}

impl From<vk::PushConstantRange> for PushConstantRange {

    fn from(value: vk::PushConstantRange) -> Self {
        Self {
            shader_stage: value.stage_flags.into(),
            offset: value.offset,
            size: value.size,
        }
    }
}

impl ByteHash for PushConstantRange {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.shader_stage.as_raw().byte_hash(hasher);
        self.offset.byte_hash(hasher);
        self.size.byte_hash(hasher);
    }
}
