use std::sync::Arc;

use nox_ash::vk;

use compact_str::format_compact;

use crate::gpu::prelude::*;
use crate::dev::error::{Error, Result, Context};

#[derive(Clone)]
pub struct ComputePipeline {
    handle: PipelineHandle,
}

impl ComputePipeline { 

    #[inline(always)]
    pub fn default_attributes(shader_set_id: ShaderSetId) -> ComputePipelineAttributes {
        ComputePipelineAttributes { shader_set_id }
    }

    #[inline(always)]
    pub(crate) unsafe fn new(
        vk: Arc<Vulkan>,
        handle: vk::Pipeline,
        shader_set: Arc<ShaderSetInner>,
    ) -> Self {
        unsafe {
            Self {
                handle: PipelineHandle::new(vk, handle, shader_set),
            }
        }
    }

    #[inline(always)]
    pub(crate) fn handle(&self) -> &PipelineHandle {
        &self.handle
    }

    #[inline(always)]
    pub(crate) fn shader_set(&self) -> &Arc<ShaderSetInner> {
        &self.handle.shader_set
    }
}

pub struct ComputePipelineAttributes {
    pub(crate) shader_set_id: ShaderSetId,
}

impl ComputePipelineAttributes {

    pub(crate) fn as_create_info(
        &self,
        resources: &Resources,
    ) -> Result<(vk::ComputePipelineCreateInfo<'_>, Arc<ShaderSetInner>)>
    {
        let shader_set = resources
            .shader_cache()
            .get_shader_set(self.shader_set_id)
            .context_with(|| format_compact!(
                "failed to get shader set {:?}",
                self.shader_set_id,
            ))?;
        let (_, entry, module) = shader_set.shaders()
            .iter()
            .find(|(s, _, _)| s.stage() == ShaderStage::Compute)
            .ok_or_else(|| Error::just_context(format_compact!(
                "couldn't find compute shader from shader set {:?}",
                self.shader_set_id,
            )))?;
        Ok((vk::ComputePipelineCreateInfo {
            s_type: vk::StructureType::COMPUTE_PIPELINE_CREATE_INFO,
            stage: vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                stage: vk::ShaderStageFlags::COMPUTE,
                module: *module,
                p_name: entry.as_ptr(),
                ..Default::default()
            },
            layout: shader_set.pipeline_layout(), 
            ..Default::default()
        }, shader_set))
    }
}
