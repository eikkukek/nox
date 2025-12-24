use ash::vk;

use crate::gpu::{resources::*, ShaderStage};
use crate::dev::error::{Error, Result};

pub struct ComputePipelineInfo {
    pub(crate) layout_id: PipelineLayoutId,
}

impl ComputePipelineInfo {

    pub fn new(layout_id: PipelineLayoutId) -> Self {
        Self {
            layout_id,
        }
    }

    pub(crate) fn as_create_info(
        &self,
        resources: &Resources,
    ) -> Result<vk::ComputePipelineCreateInfo<'_>>
    {
        let layout = resources.get_pipeline_layout(self.layout_id)?;
        let shader = layout
            .shader_ids()
            .iter()
            .map(|v| resources.get_shader(*v).unwrap())
            .find(|v| v.stage() == ShaderStage::Compute)
            .ok_or(Error::just_context("couldn't find compute shader"))?;
        const NAME: &core::ffi::CStr = unsafe {
            core::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0")
        };
        Ok(vk::ComputePipelineCreateInfo {
            s_type: vk::StructureType::COMPUTE_PIPELINE_CREATE_INFO,
            stage: vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                stage: vk::ShaderStageFlags::COMPUTE,
                module: *shader.shader_module(),
                p_name: NAME.as_ptr(),
                ..Default::default()
            },
            layout: layout.handle(),
            ..Default::default()
        })
    }
}
