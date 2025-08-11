use ash::vk;

use nox_mem::{slice, vec_types::{Vector, GlobalVec, FixedVec}, Allocator};

use super::*;

use crate::renderer::{image::Format, *};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GraphicsPipelineInfo {
    pub(crate) dynamic_states: GlobalVec<vk::DynamicState>,
    pub(crate) color_output_formats: GlobalVec<vk::Format>,
    vertex_input_bindings: GlobalVec<VertexInputBinding>,
    polygon_mode: PolygonMode,
    cull_mode: CullMode,
    front_face: FrontFace,
    depth_bias_info: Option<DepthBiasInfo>,
    primitive_topology: (PrimitiveTopology, bool),
    sample_shading_info: Option<SampleShadingInfo>,
    depth_stencil_info: Option<DepthStencilInfo>,
    color_blend_info: ColorBlendInfo,
    pub(crate) depth_output_format: vk::Format,
    pub(crate) stencil_output_format: vk::Format,
    vertex_input_attribute_count: u32,
    pub(crate) layout_id: PipelineLayoutID,
    depth_clamp: bool,
    rasterizer_discard: bool,
}

impl GraphicsPipelineInfo {

    pub fn new(layout_id: PipelineLayoutID) -> Self {
        Self {
            dynamic_states: GlobalVec::from(slice![
                vk::DynamicState::VIEWPORT,
                vk::DynamicState::SCISSOR,
            ]),
            color_output_formats: Default::default(),
            vertex_input_bindings: Default::default(),
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
            vertex_input_attribute_count: 0,
            layout_id,
            depth_clamp: false,
            rasterizer_discard: false,
        }
    }

    /// Adds vertex input binding to the pipeline (see [`VertexInputBinding`])
    /// The binding must be unique and its attribute locations must not intersect
    /// with other bindings.
    pub fn with_vertex_input_binding(&mut self, binding: VertexInputBinding) -> &mut Self {
        let first_location = binding.first_location();
        let last_location = binding.last_location();
        for b in &self.vertex_input_bindings {
            if b.binding == binding.binding {
                panic!("vertex binding {} already exists", binding.binding)
            }
            if first_location < b.last_location()  &&
                b.first_location() < last_location
            {
                panic!("vertex input binding {} intersects with binding {}", binding.binding, b.binding)
            }
        }
        self.vertex_input_attribute_count += binding.attributes.len() as u32;
        self.vertex_input_bindings.push(binding).unwrap();
        self
    }

    pub fn with_depth_clamp(&mut self, enabled: bool) -> &mut Self {
        self.depth_clamp = enabled;
        self
    }

    pub fn with_depth_stencil(&mut self, info: DepthStencilInfo) -> &mut Self {
        self.depth_stencil_info = Some(info);
        self
    }

    pub fn with_rasterizer_discard(&mut self, enabled: bool) -> &mut Self {
        self.rasterizer_discard = enabled;
        self
    }

    pub fn with_polygon_mode(&mut self, polygon_mode: PolygonMode) -> &mut Self {
        self.polygon_mode = polygon_mode;
        self
    }

    pub fn with_cull_mode(&mut self, cull_mode: CullMode) -> &mut Self {
        self.cull_mode = cull_mode;
        self
    }

    pub fn with_front_face(&mut self, front_face: FrontFace) -> &mut Self {
        self.front_face = front_face;
        self
    }

    pub fn with_depth_bias(&mut self, depth_bias_info: DepthBiasInfo) -> &mut Self {
        self.depth_bias_info = Some(depth_bias_info);
        self
    }

    pub fn with_primitive_topology(&mut self, topology: PrimitiveTopology, restart_enable: bool) -> &mut Self {
        self.primitive_topology = (topology, restart_enable);
        self
    }

    pub fn with_sample_shading(&mut self, sample_shading_info: Option<SampleShadingInfo>) -> &mut Self {
        self.sample_shading_info = sample_shading_info;
        self
    }

    /// Blend constants are used with color attachments that use 'ConstColor' or 'ConstAlpha' BlendFactors.
    /// The default constants are [0.0, 0.0, 0.0, 0.0]
    pub fn with_blend_constants(&mut self, blend_constants: BlendConstants) -> &mut Self {
        self.color_blend_info.blend_constants = blend_constants;
        self
    }

    /// Appends a color output to the pipeline.
    /// The number of color outputs of a pipeline must match exactly with the number of outputs in the fragment shader.
    pub fn with_color_output(
        &mut self,
        format: impl Format,
        write_mask: WriteMask,
        blend_state: Option<ColorOutputBlendState>
    ) -> &mut Self
    {
        self.color_output_formats.push(format.as_vk_format()).unwrap();
        self.color_blend_info.add_attachment(write_mask, blend_state);
        self
    }

    pub(crate) fn with_color_output_vk(
        &mut self,
        format: vk::Format,
        write_mask: WriteMask,
        blend_state: Option<ColorOutputBlendState>
    ) -> &mut Self
    {
        self.color_output_formats.push(format).unwrap();
        self.color_blend_info.add_attachment(write_mask, blend_state);
        self
    }

    pub fn with_depth_output(&mut self, format: impl Format) -> &mut Self {
        self.depth_output_format = format.as_vk_format();
        self
    }

    pub fn with_stencil_output(&mut self, format: impl Format) -> &mut Self {
        self.stencil_output_format = format.as_vk_format();
        self
    }

    pub fn with_dynamic_states(&mut self, dynamic_state: &[DynamicState]) -> &mut Self {
        self.dynamic_states.append_map(dynamic_state, |v| (*v).into()).unwrap();
        self
    }

    pub(crate) fn as_create_info<'a, Alloc: Allocator>(
        &self,
        global_resources: &GlobalResources,
        alloc: &'a Alloc,
    ) -> Result<CreateInfos<'a, Alloc>, Error>
    {

        let layout = global_resources.get_pipeline_layout(self.layout_id)?;
        let shader_ids = layout.shader_ids();

        let mut shader_stage_infos = FixedVec::with_capacity(shader_ids.len(), alloc)?;

        let mut vertex_shader_included = false;
        let mut fragment_shader_included = false;

        for id in shader_ids {
            let shader = global_resources.get_shader(*id)?;
            const NAME: &core::ffi::CStr = unsafe {
                core::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0")
            };
            if shader.stage() == ShaderStage::Vertex {
                if vertex_shader_included {
                    return Err(Error::ShaderError(String::from("Vertex shader included twice in pipeline")))
                }
                vertex_shader_included = true;
            }
            if shader.stage() == ShaderStage::Fragment {
                if fragment_shader_included {
                    return Err(Error::ShaderError(String::from("Fragment shader included twice in pipeline")))
                }
                fragment_shader_included = true;
            }
            shader_stage_infos.push(vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                stage: shader.stage().into(),
                module: *shader.shader_module(),
                p_name: NAME.as_ptr(),
                ..Default::default()
            }).unwrap();
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

        let depth_bias_info = self.depth_bias_info.unwrap_or(Default::default());

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

        let sample_shading_info = self.sample_shading_info.unwrap_or(Default::default());

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

        let depth_stencil_info = self.depth_stencil_info.unwrap_or(Default::default());

        let stencil_test_info = depth_stencil_info.stencil_test_info.unwrap_or(Default::default());

        let depth_bounds = depth_stencil_info.depth_bounds.unwrap_or(Default::default());

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
            logic_op: self.color_blend_info.logic_op.unwrap_or(Default::default()),
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

        let mut vertex_input_bindings = FixedVec 
            ::with_capacity(self.vertex_input_bindings.len(), alloc)?;

        let mut vertex_input_attributes = FixedVec
            ::with_capacity(self.vertex_input_attribute_count as usize, alloc)?;

        for binding in self.vertex_input_bindings.iter().map(|v| *v) {
            vertex_input_bindings.push(binding.into()).unwrap();
            let b = binding.binding;
            for attr in binding.attributes {
                vertex_input_attributes.push(attr.into_vk(b)).unwrap();
            }
        }

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            vertex_binding_description_count: vertex_input_bindings.len() as u32,
            p_vertex_binding_descriptions: vertex_input_bindings.as_ptr(),
            vertex_attribute_description_count: vertex_input_attributes.len() as u32,
            p_vertex_attribute_descriptions: vertex_input_attributes.as_ptr(),
            ..Default::default()
        };

        Ok(CreateInfos {
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
            layout: layout.handle(),
        })
    }
}
