use std::sync::Arc;

use nox_ash::vk;

use compact_str::format_compact;

use crate::gpu::prelude::*;
use crate::dev::error::{Error, Result, Context};

#[derive(Clone)]
pub(crate) struct ComputePipeline {
    handle: PipelineHandle,
}

impl ComputePipeline { 

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

#[derive(Clone)]
pub struct ComputePipelineCreateInfo {
    pub(super) shader_set_id: ShaderSetId,
    pub(super) robustness_info: vk::PipelineRobustnessCreateInfo<'static>,
}

impl ComputePipelineCreateInfo {

    /// Creates new compute pipeline create info.
    ///
    /// # Valid usage
    /// - `shader_set_id` *must* be a valid [`ShaderSetId`] and the shader set *must* contain
    /// [`ShaderStage::Compute`].
    #[inline(always)]
    pub fn new(shader_set_id: ShaderSetId) -> Self {
        ComputePipelineCreateInfo {
            shader_set_id,
            robustness_info: PipelineRobustnessInfo::default().into(),
        }
    }

    #[inline(always)]
    pub fn with_robustness_info(
        mut self,
        robustness_info: PipelineRobustnessInfo,
    ) -> Self {
        self.robustness_info = robustness_info.into();
        self
    }

    pub(crate) fn prepare(
        &mut self,
        gpu: &Gpu,
    ) -> Result<(vk::ComputePipelineCreateInfo<'_>, Arc<ShaderSetInner>)>
    {
        let shader_set = gpu
            .shader_cache()
            .get_shader_set(self.shader_set_id)
            .context_with(|| format_compact!(
                "failed to get shader set {:?}",
                self.shader_set_id,
            ))?.clone();
        let (_, entry, module) = shader_set.shaders()
            .iter()
            .find(|(s, _, _)| s.stage() == ShaderStage::Compute)
            .ok_or_else(|| Error::just_context(format_compact!(
                "couldn't find compute shader from shader set {:?}",
                self.shader_set_id,
            )))?;
        match self.robustness_info.images {
            vk::PipelineRobustnessImageBehavior::ROBUST_IMAGE_ACCESS => {
                if !gpu
                    .get_device_attribute(ext::robust_image_access::IS_SUPPORTED_ATTRIBUTE_NAME)
                    .bool().unwrap_or_default()
                {
                    return Err(Error::just_context(
                    "pipeline robustness image behavior must not robust image access if robust image accesss extension is not enabled"
                    ))
                }
            },
            vk::PipelineRobustnessImageBehavior::ROBUST_IMAGE_ACCESS_2 => {
                if !gpu
                    .get_device_attribute(ext::robustness2::IS_ROBUST_IMAGE_ACCESS_2_SUPPORTED_ATTRIBUTE_NAME)
                    .bool().unwrap_or_default()
                {
                    return Err(Error::just_context(
                        "pipeline robustness image behavior must not be robust image access 2 if it is not supported"
                    ))
                }
            },
            _ => {}
        }
        for behavior in [
                self.robustness_info.storage_buffers,
                self.robustness_info.uniform_buffers,
                self.robustness_info.vertex_input,
            ]
        {
            match behavior {
                vk::PipelineRobustnessBufferBehavior::ROBUST_BUFFER_ACCESS_2 => {
                    if !gpu
                        .get_device_attribute(ext::robustness2::IS_ROBUST_BUFFER_ACCESS_2_SUPPORTED_ATTRIBUTE_NAME)
                        .bool().unwrap_or_default()
                    {
                        return Err(Error::just_context(
                            "pipeline robustness buffer behavior must not be robust buffer access 2 if its not supported"
                        ))
                    }
                },
                _ => {},
            }
        }
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
        }.push_next(&mut self.robustness_info), shader_set))
    }
}
