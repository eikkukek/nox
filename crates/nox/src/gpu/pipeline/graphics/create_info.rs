use core::{
    ops::{Deref, DerefMut},
    ffi::c_void,
    ptr,
};

use nox_mem::vec::NonNullVec32;

use crate::gpu::ext;

use super::*;

#[derive(Clone)]
pub struct GraphicsPipelineCreateTemplate {
    pub(crate) dynamic_states: Vec32<DynamicState>,
    pub(crate) color_output_formats: Vec32<vk::Format>,
    pub(super) vertex_input_bindings: Vec32<VertexInputBinding>,
    pub(super) vertex_input_attributes: Vec32<VertexInputAttributeInternal>,
    pub(super) polygon_mode: PolygonMode,
    pub(super) cull_mode: CullModeFlags,
    pub(super) front_face: FrontFace,
    pub(super) depth_bias_info: Option<DepthBiasInfo>,
    pub(super) primitive_topology: (PrimitiveTopology, bool),
    pub(crate) sample_shading_info: Option<SampleShadingInfo>,
    pub(super) depth_stencil_info: Option<DepthStencilInfo>,
    pub(super) color_blend_info: ColorBlendInfo,
    pub(crate) depth_output_format: vk::Format,
    pub(crate) stencil_output_format: vk::Format,
    pub(crate) shader_set_id: ShaderSetId,
    pub(super) line_width: f32,
    pub(super) depth_clamp: bool,
    pub(super) rasterizer_discard: bool,
    pub(super) robustness_info: PipelineRobustnessInfo,
}

pub struct GraphicsPipelineCreateInfo<'a> {
    out: &'a mut GraphicsPipelineId,
    inner: GraphicsPipelineCreateInfo,
}

impl Deref for GraphicsPipelineCreateInfo<'a> {
}

impl GraphicsPipelineCreateInfo {

    pub fn new(shader_set_id: ShaderSetId) -> Self {
        Self {
            dynamic_states: vec32![],
            color_output_formats: Default::default(),
            vertex_input_bindings: Default::default(),
            vertex_input_attributes: Default::default(),
            polygon_mode: Default::default(),
            cull_mode: Default::default(),
            front_face: Default::default(),
            depth_bias_info: None,
            primitive_topology: Default::default(),
            sample_shading_info: None,
            depth_stencil_info: None,
            color_blend_info: Default::default(),
            depth_output_format: Default::default(),
            stencil_output_format: Default::default(),
            shader_set_id,
            line_width: 1.0,
            depth_clamp: false,
            rasterizer_discard: false,
            robustness_info: Default::default(),
        }
    }

    /// Adds vertex input to the pipeline.
    ///
    /// The binding must be unique and input locations from [`I::get_attributes()`] with
    /// `first_location` must not contain locations already added to the pipeline.
    #[inline(always)]
    pub fn with_vertex_input<const N_ATTRIBUTES: usize, I>(
        mut self, 
        binding: u32,
        first_location: u32,
        input_rate: VertexInputRate,
    ) -> Result<Self>
        where I: VertexInput<N_ATTRIBUTES>
    {
        if self.vertex_input_bindings
            .iter()
            .any(|b| b.binding == binding)
        {
            return Err(Error::just_context(format_compact!(
                "binding {binding} already exists in pipeline"
            )))
        }
        let attributes = I::get_attributes(first_location);
        let last_location = attributes.last()
            .map(|attr| attr.location)
            .unwrap_or(first_location);
        for attr in self.vertex_input_attributes.iter().copied() {
            if attr.location >= first_location &&
                attr.location <= last_location
            {
                return Err(Error::just_context(format_compact!(
                    "location {} already exists in pipeline", attr.location
                )))
            }
        }
        self.vertex_input_attributes
            .append_map(&attributes, |attr| attr.into_internal(binding));
        self.vertex_input_bindings.push(VertexInputBinding::new(
            binding, input_rate, size_of::<I>() as u32
        ));
        Ok(self)
    }

    #[inline(always)]
    pub fn with_dynamic_states(mut self, dynamic_state: &[DynamicState]) -> Self {
        self.dynamic_states.fast_append(dynamic_state);
        self
    }

    /// Sets the width of rasterized line segments.
    ///
    /// This *must* be left as the default value `1.0`, if [`BaseDeviceFeatures`] `wide_lines` is
    /// set to `false`.
    ///
    /// If dynamic states contain [`DynamicState::LINE_WIDTH`], this value is ignored and *must* be
    /// set with [`DrawCommands`]
    #[inline(always)]
    pub fn with_line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    #[inline(always)]
    pub fn with_depth_clamp(mut self, enabled: bool) -> Self {
        self.depth_clamp = enabled;
        self
    }

    #[inline(always)]
    pub fn with_depth_stencil(mut self, info: DepthStencilInfo) -> Self {
        self.depth_stencil_info = Some(info);
        self
    }

    #[inline(always)]
    pub fn with_rasterizer_discard(mut self, enabled: bool) -> Self {
        self.rasterizer_discard = enabled;
        self
    }

    #[inline(always)]
    pub fn with_polygon_mode(mut self, polygon_mode: PolygonMode) -> Self {
        self.polygon_mode = polygon_mode;
        self
    }

    #[inline(always)]
    pub fn with_cull_mode(mut self, cull_mode: CullModeFlags) -> Self {
        self.cull_mode = cull_mode;
        self
    }

    #[inline(always)]
    pub fn with_front_face(mut self, front_face: FrontFace) -> Self {
        self.front_face = front_face;
        self
    }

    #[inline(always)]
    pub fn with_depth_bias(mut self, depth_bias_info: DepthBiasInfo) -> Self {
        self.depth_bias_info = Some(depth_bias_info);
        self
    }

    #[inline(always)]
    pub fn with_primitive_topology(mut self, topology: PrimitiveTopology, restart_enable: bool) -> Self {
        self.primitive_topology = (topology, restart_enable);
        self
    }

    #[inline(always)]
    pub fn with_sample_shading(mut self, sample_shading_info: SampleShadingInfo) -> Self {
        self.sample_shading_info = Some(sample_shading_info);
        self
    }

    /// Blend constants are used with color attachments that use 'ConstColor' or 'ConstAlpha' BlendFactors.
    /// The default constants are [0.0, 0.0, 0.0, 0.0]
    #[inline(always)]
    pub fn with_blend_constants(mut self, blend_constants: BlendConstants) -> Self {
        self.color_blend_info.blend_constants = blend_constants;
        self
    }

    /// Appends a color output to the pipeline.
    /// The number of color outputs of a pipeline must match exactly with the number of outputs in the fragment shader.
    #[inline(always)]
    pub fn with_color_output(
        mut self,
        format: impl Format,
        write_mask: WriteMask,
        blend_state: Option<ColorOutputBlendState>,
    ) -> Self
    {
        self.color_output_formats.push(format.as_vk_format());
        self.color_blend_info.add_attachment(write_mask, blend_state);
        self
    } 

    #[inline(always)]
    pub fn with_depth_output(mut self, format: impl Format) -> Self {
        self.depth_output_format = format.as_vk_format();
        self
    }

    #[inline(always)]
    pub fn with_stencil_output(mut self, format: impl Format) -> Self {
        self.stencil_output_format = format.as_vk_format();
        self
    }

    /// Sets robustness info of the pipeline.
    ///
    /// # Valid usage
    /// - If [`ext::pipeline_robustness`] is not enabled, the robustness info *must* be
    /// [`PipelineRobustnessInfo::default`].
    /// - If [`ext::robust_image_access`] is not supported, image behavior *must* not be
    /// [`PipelineRobustnessImageBehavior::ROBUST_IMAGE_ACCESS`].
    /// - If [`ext::robustness2`] 'robust_buffer_access2` is not supported, buffer behaviors *must*
    /// not be [`PipelineRobustnessBufferBehavior::ROBUST_BUFFER_ACCESS_2`].
    /// - If [`ext::robustness2`] `robust_image_access2` is not supported, image behavior *must* not
    /// [`PipelineRobustnessImageBehavior::ROBUST_IMAGE_ACCESS_2`].
    #[inline(always)]
    pub fn with_robustness_info(mut self, info: PipelineRobustnessInfo) -> Self {
        self.robustness_info = info;
        self
    }

    pub(crate) fn prepare<'a, Alloc>(
        &self,
        gpu: &Gpu,
        alloc: &'a Alloc,
    ) -> Result<(PreparedCreateInfos<'a, Alloc>, Arc<ShaderSetInner>)>
        where 
            Alloc: LocalAlloc<Error = Error>,
    {

        let shader_set = gpu
            .shader_cache()
            .get_shader_set(self.shader_set_id)
            .context_with(|| format_compact!(
                "failed to get shader set {:?}",
                self.shader_set_id,
            ))?.clone();

        let shaders = shader_set.shaders();
        let mut shader_stage_infos = NonNullVec32
            ::with_capacity(shaders.len() as u32, alloc)?;

        let mut vertex_shader_included = false;
        let mut fragment_shader_included = false;

        for (shader, entry, module) in shaders {
            match shader.stage() {
                ShaderStage::Vertex => {
                    if vertex_shader_included {
                        return Err(Error::just_context("vertex shader included twice in pipeline"))
                    }
                    vertex_shader_included = true;
                }
                ShaderStage::Fragment => {
                    if fragment_shader_included {
                        return Err(Error::just_context("fragment shader included twice in pipeline"))
                    }
                    fragment_shader_included = true;
                },
                _ => {
                    return Err(Error::just_context(format_compact!(
                        "unsupported shader stage {}, only vertex and fragment shaders are supported for graphics pipelines",
                        shader.stage(),
                    )))
                }
            }
            shader_stage_infos.push(vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                stage: shader.stage().into(),
                p_name: entry.as_ptr(),
                module: *module,
                ..Default::default()
            });
        }

        if !vertex_shader_included {
            return Err(Error::just_context("no vertex shader included in graphics pipeline"))
        }

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            topology: self.primitive_topology.0.into(),
            primitive_restart_enable: self.primitive_topology.1.into(),
            ..Default::default()
        };

        let tesellation_state = vk::PipelineTessellationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_TESSELLATION_STATE_CREATE_INFO,
            ..Default::default()
        };

        let depth_bias_info = self.depth_bias_info.unwrap_or_default();

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            depth_clamp_enable: self.depth_clamp.into(),
            rasterizer_discard_enable: self.rasterizer_discard.into(),
            polygon_mode: self.polygon_mode.into(),
            cull_mode: self.cull_mode.into(),
            front_face: self.front_face.into(),
            depth_bias_enable: self.depth_bias_info.is_some().into(),
            depth_bias_constant_factor: depth_bias_info.constant_factor,
            depth_bias_clamp: depth_bias_info.clamp,
            depth_bias_slope_factor: depth_bias_info.slope_factor,
            line_width: 1.0,
            ..Default::default()
        };

        let sample_shading_info = self.sample_shading_info.unwrap_or_default();

        let multisample_state = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            rasterization_samples: sample_shading_info.samples.into(),
            sample_shading_enable: self.sample_shading_info.is_some().into(),
            min_sample_shading: sample_shading_info.min_shading,
            p_sample_mask: core::ptr::null(),
            alpha_to_coverage_enable: sample_shading_info.alpha_to_coverage.into(),
            alpha_to_one_enable: sample_shading_info.alpha_to_one.into(),
            ..Default::default()
        };

        let depth_stencil_info = self.depth_stencil_info.unwrap_or_default();

        let stencil_test_info = depth_stencil_info.stencil_test_info.unwrap_or_default();

        let depth_bounds = depth_stencil_info.depth_bounds.unwrap_or_default();

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            depth_test_enable: self.depth_stencil_info.is_some().into(),
            depth_write_enable: depth_stencil_info.write_enable.into(),
            depth_compare_op: depth_stencil_info.compare_op.into(),
            depth_bounds_test_enable: depth_stencil_info.depth_bounds.is_some().into(),
            stencil_test_enable: depth_stencil_info.stencil_test_info.is_some().into(),
            front: stencil_test_info.front.into(),
            back: stencil_test_info.back.into(),
            min_depth_bounds: depth_bounds.min,
            max_depth_bounds: depth_bounds.max,
            ..Default::default()
        };

        let color_blend_attachment_states = self.color_blend_info.attachments();

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            logic_op_enable: self.color_blend_info.logic_op.is_some().into(),
            logic_op: self.color_blend_info.logic_op.unwrap_or_default(),
            attachment_count: color_blend_attachment_states.len() as u32,
            p_attachments: color_blend_attachment_states.as_ptr(),
            blend_constants: self.color_blend_info.blend_constants.into(),
            ..Default::default()
        };

        let mut dynamic_states = NonNullVec32::with_capacity(
            4 + self.dynamic_states.len(),
            alloc
        )?;

        dynamic_states.append(&[
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR,
            vk::DynamicState::VIEWPORT_WITH_COUNT,
            vk::DynamicState::SCISSOR_WITH_COUNT,
        ]);

        dynamic_states.append_map(
            &self.dynamic_states,
            |state| state.into(),
        );

        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let rendering_info = vk::PipelineRenderingCreateInfo {
            s_type: vk::StructureType::PIPELINE_RENDERING_CREATE_INFO,
            view_mask: 0,
            color_attachment_count: self.color_output_formats.len() as u32,
            p_color_attachment_formats: self.color_output_formats.as_ptr(),
            depth_attachment_format: self.depth_output_format,
            stencil_attachment_format: self.stencil_output_format,
            ..Default::default()
        };

        let mut vertex_input_bindings = NonNullVec32 
            ::with_capacity(self.vertex_input_bindings.len(), alloc)?;

        vertex_input_bindings.extend(
            self.vertex_input_bindings.iter().copied().map(Into::into)
        );

        let mut vertex_input_attributes = NonNullVec32
            ::with_capacity(self.vertex_input_attributes.len(), alloc)?;

        vertex_input_attributes.extend(
            self.vertex_input_attributes.iter().copied().map(Into::into)
        );

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            vertex_binding_description_count: vertex_input_bindings.len() as u32,
            p_vertex_binding_descriptions: vertex_input_bindings.as_ptr(),
            vertex_attribute_description_count: vertex_input_attributes.len() as u32,
            p_vertex_attribute_descriptions: vertex_input_attributes.as_ptr(),
            ..Default::default()
        };

        if !gpu
            .get_device_attribute(ext::pipeline_robustness::IS_ENABLED_ATTRIBUTE_NAME)
            .bool().unwrap_or_default() &&
            self.robustness_info != PipelineRobustnessInfo::default()
        {
            return Err(Error::just_context(
                "pipeline robustness info must be the default value if pipeline robustness extension is not enabled"
            ))
        }

        match self.robustness_info.image_behavior {
            PipelineRobustnessImageBehavior::RobustImageAccess => {
                if !gpu
                    .get_device_attribute(ext::robust_image_access::IS_SUPPORTED_ATTRIBUTE_NAME)
                    .bool().unwrap_or_default()
                {
                    return Err(Error::just_context(
                    "pipeline robustness image behavior must not robust image access if robust image accesss extension is not enabled"
                    ))
                }
            },
            PipelineRobustnessImageBehavior::RobustImageAccess2 => {
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
        };

        for behavior in [
                self.robustness_info.storage_buffer_behavior,
                self.robustness_info.uniform_buffer_behavior,
                self.robustness_info.vertex_input_behavior,
            ]
        {
            match behavior {
                PipelineRobustnessBufferBehavior::RobustBufferAccess2 => {
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

        let mut s = (PreparedCreateInfos {
            p_next: ptr::null(),
            shader_stage_infos,
            vertex_input_state,
            input_assembly_state,
            tesellation_state,
            rasterization_state,
            multisample_state,
            depth_stencil_state,
            color_blend_state,
            dynamic_state,
            rendering_info,
            layout: shader_set.pipeline_layout(),
            robustness_info: self.robustness_info.into(),
            _vertex_input_bindings: vertex_input_bindings,
            _vertex_input_attributes: vertex_input_attributes,
            _dynamic_states: dynamic_states,
            alloc,
        }, shader_set);
        s.0.rendering_info.p_next = &s.0.robustness_info as *const _ as *const c_void;
        s.0.p_next = &s.0.rendering_info as *const _ as *const c_void;
        Ok(s)
    }
}

pub(crate) struct PreparedCreateInfos<'a, Alloc>
    where Alloc: LocalAlloc,
{
    pub p_next: *const c_void,
    pub shader_stage_infos: NonNullVec32<'a, vk::PipelineShaderStageCreateInfo<'static>>,
    pub vertex_input_state: vk::PipelineVertexInputStateCreateInfo<'static>,
    pub input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    pub tesellation_state: vk::PipelineTessellationStateCreateInfo<'a>,
    pub rasterization_state: vk::PipelineRasterizationStateCreateInfo<'a>,
    pub multisample_state: vk::PipelineMultisampleStateCreateInfo<'a>,
    pub depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo<'a>,
    pub color_blend_state: vk::PipelineColorBlendStateCreateInfo<'a>,
    pub dynamic_state: vk::PipelineDynamicStateCreateInfo<'a>,
    pub rendering_info: vk::PipelineRenderingCreateInfo<'a>,
    pub layout: vk::PipelineLayout,
    pub robustness_info: vk::PipelineRobustnessCreateInfo<'static>,
    pub _vertex_input_bindings: NonNullVec32<'a, vk::VertexInputBindingDescription>,
    pub _vertex_input_attributes: NonNullVec32<'a, vk::VertexInputAttributeDescription>,
    pub _dynamic_states: NonNullVec32<'a, vk::DynamicState>,
    pub alloc: &'a Alloc,
}

impl<'a, Alloc> PreparedCreateInfos<'a, Alloc>
    where Alloc: LocalAlloc,
{

    #[inline(always)]
    pub fn as_create_info(&self) -> vk::GraphicsPipelineCreateInfo<'_> {
        vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: self.p_next,
            stage_count: self.shader_stage_infos.len() as u32,
            p_stages: self.shader_stage_infos.as_ptr(),
            p_vertex_input_state: &self.vertex_input_state,
            p_input_assembly_state: &self.input_assembly_state,
            p_tessellation_state: &self.tesellation_state,
            p_viewport_state: ptr::null(),
            p_rasterization_state: &self.rasterization_state,
            p_multisample_state: &self.multisample_state,
            p_depth_stencil_state: &self.depth_stencil_state,
            p_color_blend_state: &self.color_blend_state,
            p_dynamic_state: &self.dynamic_state,
            layout: self.layout,
            ..Default::default()
        }
    }
}

impl<Alloc> Drop for PreparedCreateInfos<'_, Alloc>
    where Alloc: LocalAlloc,
{

    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            self.shader_stage_infos.drop_and_free(self.alloc);
            self._vertex_input_bindings.drop_and_free(self.alloc);
            self._vertex_input_attributes.drop_and_free(self.alloc);
            self._dynamic_states.drop_and_free(self.alloc);
        }
    }
}
