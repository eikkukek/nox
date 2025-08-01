use core::hash::Hash;

use ash::vk;

use nox_mem::{Vector, vec_types::GlobalVec};

use crate::renderer::Error;

use super::*;

#[derive(Clone)]
pub struct PipelineLayoutInfo {
    set_layouts: GlobalVec<vk::DescriptorSetLayout>,
    push_constant_ranges: GlobalVec<vk::PushConstantRange>,
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
        }
    }

    pub fn with_set_layout(mut self, handle: vk::DescriptorSetLayout) -> Self {
        self.set_layouts.push(handle).unwrap();
        self
    }

    pub fn _with_push_constant_range(mut self, range: PushConstantRange) -> Self {
        self.push_constant_ranges.push(range.into()).unwrap();
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

impl PartialEq for PipelineLayoutInfo {

    fn eq(&self, other: &Self) -> bool {
        self.set_layouts == other.set_layouts &&
        self.push_constant_ranges.map_eq(
            &other.push_constant_ranges,
            |v1, v2| {
                Into::<PushConstantRange>::into(*v1) ==
                Into::<PushConstantRange>::into(*v2)
        })
    }
}

impl Eq for PipelineLayoutInfo {}

impl Hash for PipelineLayoutInfo {

    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.set_layouts.hash(state);
        self.push_constant_ranges.map_hash(
            state,
            |v, s| {
                Into::<PushConstantRange>::into(*v).hash(s);
            }
        );
    }
}
