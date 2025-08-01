use core::hash::Hash;

use ash::vk;

use nox_mem::{Vector, vec_types::GlobalVec};

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

#[derive(Clone)]
pub struct DescriptorSetLayoutInfo {
    bindings: GlobalVec<vk::DescriptorSetLayoutBinding<'static>>,
}

impl DescriptorSetLayoutInfo {

    pub fn new(binding_capacity: u32) -> Self {
        Self {
            bindings: GlobalVec::with_capacity(binding_capacity as usize).unwrap(),
        }
    }

    pub fn with_binding(&mut self, binding: DescriptorBindingInfo) -> &mut Self {
        self.bindings.push(binding.into()).unwrap();
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

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
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
