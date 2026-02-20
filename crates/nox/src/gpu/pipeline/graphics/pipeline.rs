use std::sync::Arc;

use nox_ash::vk;

use compact_str::format_compact;

use nox_mem::{
    alloc::LocalAlloc,
    slice::{self, AllocSlice},
    vec::{FixedVec32, Vec32, Vector},
    dynamic::Dyn,
    vec32,
};

use crate::gpu::prelude::*;
use crate::dev::error::{Error, Result, Context};

#[derive(Clone)]
pub struct GraphicsPipeline {
    handle: PipelineHandle,
    samples: MSAA,
    vertex_input_bindings: Vec32<VertexInputBinding>,
    color_outputs_and_dynamic_states: Arc<[i32]>,
    n_color_output_formats: u32,
    depth_output_format: vk::Format,
    stencil_output_format: vk::Format,
}

impl GraphicsPipeline {

    #[inline(always)]
    pub(crate) unsafe fn new(
        vk: Arc<Vulkan>,
        handle: vk::Pipeline,
        shader_set: Arc<ShaderSetInner>,
        attr: &GraphicsPipelineAttributes,
    ) -> Self {
        let n_color_output_formats = attr.color_output_formats.len() as usize;
        let color_outputs_and_dynamic_states =
        unsafe {
            let data: Arc<[i32]> = Arc::uninit_slice(
                n_color_output_formats + attr.dynamic_states.len() as usize
            );
            let data = Arc::into_raw(data);
            let ptr = Dyn::raw_parts(data).data.cast_mut();
            slice::cast::<_, i32>(&attr.color_output_formats).unwrap().as_ptr()
                .copy_to_nonoverlapping(
                    ptr,
                    n_color_output_formats,
                );
            slice::cast::<_, i32>(&attr.dynamic_states).unwrap().as_ptr()
                .copy_to_nonoverlapping(
                    ptr.add(n_color_output_formats),
                    attr.dynamic_states.len() as usize
                );
            Arc::from_raw(data)
        };
        Self {
            handle: unsafe {
                PipelineHandle::new(vk, handle, shader_set)
            },
            samples: attr.sample_shading_info
                .map(|info| info.samples)
                .unwrap_or(MSAA::X1),
            vertex_input_bindings: attr.vertex_input_bindings.clone(),
            color_outputs_and_dynamic_states,
            n_color_output_formats: n_color_output_formats as u32,
            depth_output_format: attr.depth_output_format,
            stencil_output_format: attr.stencil_output_format,
        }
    }

    #[inline(always)]
    pub(crate) fn handle(&self) -> &PipelineHandle {
        &self.handle
    }

    #[inline(always)]
    pub fn samples(&self) -> MSAA {
        self.samples
    }

    #[inline(always)]
    pub fn vertex_input_bindings(&self) -> &[VertexInputBinding] {
        &self.vertex_input_bindings
    }

    #[inline(always)]
    pub fn color_output_formats(&self) -> &[VkFormat] {
        unsafe {
            slice::cast(
                &self.color_outputs_and_dynamic_states[0..self.n_color_output_formats as usize]
            ).unwrap()
        }
    }

    #[inline(always)]
    pub fn depth_output_format(&self) -> VkFormat {
        self.depth_output_format
    }

    #[inline(always)]
    pub fn stencil_output_format(&self) -> VkFormat {
        self.stencil_output_format
    }

    #[inline(always)]
    pub fn dynamic_states(&self) -> &[VkDynamicState] {
        unsafe {
            slice::cast(
                &self.color_outputs_and_dynamic_states[self.n_color_output_formats as usize..]
            ).unwrap()
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GraphicsPipelineAttributes {
    pub(crate) dynamic_states: Vec32<vk::DynamicState>,
    pub(crate) color_output_formats: Vec32<vk::Format>,
    vertex_input_bindings: Vec32<VertexInputBinding>,
    vertex_input_attributes: Vec32<VertexInputAttributeInternal>,
    polygon_mode: PolygonMode,
    cull_mode: CullMode,
    front_face: FrontFace,
    depth_bias_info: Option<DepthBiasInfo>,
    primitive_topology: (PrimitiveTopology, bool),
    pub(crate) sample_shading_info: Option<SampleShadingInfo>,
    depth_stencil_info: Option<DepthStencilInfo>,
    color_blend_info: ColorBlendInfo,
    pub(crate) depth_output_format: vk::Format,
    pub(crate) stencil_output_format: vk::Format,
    pub(crate) shader_set_id: ShaderSetId,
    depth_clamp: bool,
    rasterizer_discard: bool,
}

impl GraphicsPipelineAttributes {

    pub(crate) fn new(shader_set_id: ShaderSetId) -> Self {
        Self {
            dynamic_states: vec32![
                vk::DynamicState::VIEWPORT,
                vk::DynamicState::SCISSOR
            ],
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
            depth_clamp: false,
            rasterizer_discard: false,
        }
    }

    /// Adds vertex input to the pipeline.
    ///
    /// The binding must be unique and input locations from [`I::get_attributes()`] with
    /// `first_location` must not contain locations already added to the pipeline.
    #[inline(always)]
    pub fn with_vertex_input<const N_ATTRIBUTES: usize, I>(
        &mut self, 
        binding: u32,
        first_location: u32,
        input_rate: VertexInputRate,
    ) -> Result<&mut Self>
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
    pub fn with_depth_clamp(&mut self, enabled: bool) -> &mut Self {
        self.depth_clamp = enabled;
        self
    }

    #[inline(always)]
    pub fn with_depth_stencil(&mut self, info: DepthStencilInfo) -> &mut Self {
        self.depth_stencil_info = Some(info);
        self
    }

    #[inline(always)]
    pub fn with_rasterizer_discard(&mut self, enabled: bool) -> &mut Self {
        self.rasterizer_discard = enabled;
        self
    }

    #[inline(always)]
    pub fn with_polygon_mode(&mut self, polygon_mode: PolygonMode) -> &mut Self {
        self.polygon_mode = polygon_mode;
        self
    }

    #[inline(always)]
    pub fn with_cull_mode(&mut self, cull_mode: CullMode) -> &mut Self {
        self.cull_mode = cull_mode;
        self
    }

    #[inline(always)]
    pub fn with_front_face(&mut self, front_face: FrontFace) -> &mut Self {
        self.front_face = front_face;
        self
    }

    #[inline(always)]
    pub fn with_depth_bias(&mut self, depth_bias_info: DepthBiasInfo) -> &mut Self {
        self.depth_bias_info = Some(depth_bias_info);
        self
    }

    #[inline(always)]
    pub fn with_primitive_topology(&mut self, topology: PrimitiveTopology, restart_enable: bool) -> &mut Self {
        self.primitive_topology = (topology, restart_enable);
        self
    }

    #[inline(always)]
    pub fn with_sample_shading(&mut self, sample_shading_info: SampleShadingInfo) -> &mut Self {
        self.sample_shading_info = Some(sample_shading_info);
        self
    }

    /// Blend constants are used with color attachments that use 'ConstColor' or 'ConstAlpha' BlendFactors.
    /// The default constants are [0.0, 0.0, 0.0, 0.0]
    #[inline(always)]
    pub fn with_blend_constants(&mut self, blend_constants: BlendConstants) -> &mut Self {
        self.color_blend_info.blend_constants = blend_constants;
        self
    }

    /// Appends a color output to the pipeline.
    /// The number of color outputs of a pipeline must match exactly with the number of outputs in the fragment shader.
    #[inline(always)]
    pub fn with_color_output(
        &mut self,
        format: impl Format,
        write_mask: WriteMask,
        blend_state: Option<ColorOutputBlendState>,
    ) -> &mut Self
    {
        self.color_output_formats.push(format.as_vk_format());
        self.color_blend_info.add_attachment(write_mask, blend_state);
        self
    } 

    #[inline(always)]
    pub fn with_depth_output(&mut self, format: impl Format) -> &mut Self {
        self.depth_output_format = format.as_vk_format();
        self
    }

    #[inline(always)]
    pub fn with_stencil_output(&mut self, format: impl Format) -> &mut Self {
        self.stencil_output_format = format.as_vk_format();
        self
    }

    #[inline(always)]
    pub fn with_dynamic_states(&mut self, dynamic_state: &[DynamicState]) -> &mut Self {
        self.dynamic_states.append_map(dynamic_state, |v| (*v).into());
        self
    }

    pub(crate) fn as_create_info<'a, Alloc>(
        &self,
        resources: &Resources,
        alloc: &'a Alloc,
    ) -> Result<(CreateInfos<'a, Alloc>, Arc<ShaderSetInner>)>
        where 
            Alloc: LocalAlloc<Error = Error>,
            &'a Alloc: AsRef<Alloc>,
    {

        let shader_set = resources
            .shader_cache()
            .get_shader_set(self.shader_set_id)
            .context_with(|| format_compact!(
                "failed to get shader set {:?}",
                self.shader_set_id,
            ))?;

        let shaders = shader_set.shaders();
        let mut shader_stage_infos = FixedVec32
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
            depth_bias_constant_factor: depth_bias_info.constant_factor.to_inner(),
            depth_bias_clamp: depth_bias_info.clamp.to_inner(),
            depth_bias_slope_factor: depth_bias_info.slope_factor.to_inner(),
            line_width: 1.0,
            ..Default::default()
        };

        let sample_shading_info = self.sample_shading_info.unwrap_or_default();

        let multisample_state = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            rasterization_samples: sample_shading_info.samples.into(),
            sample_shading_enable: self.sample_shading_info.is_some().into(),
            min_sample_shading: sample_shading_info.min_shading.to_inner(),
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
            min_depth_bounds: depth_bounds.min.to_inner(),
            max_depth_bounds: depth_bounds.max.to_inner(),
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

        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            dynamic_state_count: self.dynamic_states.len() as u32,
            p_dynamic_states: self.dynamic_states.as_ptr(),
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

        let mut vertex_input_bindings = FixedVec32
            ::with_capacity(self.vertex_input_bindings.len(), alloc)?;

        vertex_input_bindings.extend(
            self.vertex_input_bindings.iter().copied().map(Into::into)
        );

        let mut vertex_input_attributes = FixedVec32
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

        Ok((CreateInfos {
            _vertex_input_bindings: vertex_input_bindings,
            _vertex_input_attributes: vertex_input_attributes,
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
        }, shader_set))
    }
}
