use ash::vk;

use crate::renderer::{global_resources::*, Error, ShaderStage};

pub struct ComputePipelineInfo {
    pub(crate) layout_id: PipelineLayoutID,
}

impl ComputePipelineInfo {

    pub fn new(layout_id: PipelineLayoutID) -> Self {
        Self {
            layout_id,
        }
    }

    pub(crate) fn as_create_info(
        &self,
        global_resources: &GlobalResources,
    ) -> Result<vk::ComputePipelineCreateInfo<'_>, Error>
    {
        let layout = global_resources.get_pipeline_layout(self.layout_id)?;
        let shader = layout
            .shader_ids()
            .iter()
            .map(|v| global_resources.get_shader(*v).unwrap())
            .find(|v| v.stage() == ShaderStage::Compute)
            .ok_or(Error::ShaderError(format!("failed to find compute shader")))?;
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
