use ash::vk;

pub use vk::Format as Format;

use nox_mem::{slice, AsRaw, GlobalVec, Vector};

use crate::byte_hash::ByteHash;

pub use super::*;

#[derive(Clone)]
pub struct GraphicsPipelineInfo {
    dynamic_states: GlobalVec<vk::DynamicState>,
    color_output_formats: GlobalVec<vk::Format>,
    polygon_mode: PolygonMode,
    cull_mode: CullMode,
    front_face: FrontFace,
    depth_bias_info: Option<DepthBiasInfo>,
    primitive_topology: (PrimitiveTopology, bool),
    sample_shading_info: Option<SampleShadingInfo>,
    depth_stencil_info: Option<DepthStencilInfo>,
    color_blend_info: ColorBlendInfo,
    depth_output_format: Format,
    stencil_output_format: Format,
    depth_clamp: bool,
    rasterizer_discard: bool,
}

impl GraphicsPipelineInfo {

    pub fn new() -> Self {
        Self {
            dynamic_states: GlobalVec::from(slice![
                vk::DynamicState::VIEWPORT,
                vk::DynamicState::SCISSOR,
            ]),
            color_output_formats: Default::default(),
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
            depth_clamp: false,
            rasterizer_discard: false,
        }
    }

    pub fn with_depth_clamp(mut self, enabled: bool) -> Self {
        self.depth_clamp = enabled;
        self
    }

    pub fn with_rasterizer_discard(mut self, enabled: bool) -> Self {
        self.rasterizer_discard = enabled;
        self
    }

    pub fn with_polygon_mode(mut self, polygon_mode: PolygonMode) -> Self {
        self.polygon_mode = polygon_mode;
        self
    }

    pub fn with_cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.cull_mode = cull_mode;
        self
    }

    pub fn with_front_face(mut self, front_face: FrontFace) -> Self {
        self.front_face = front_face;
        self
    }

    pub fn with_depth_bias(mut self, depth_bias_info: Option<DepthBiasInfo>) -> Self {
        self.depth_bias_info = depth_bias_info;
        self
    }

    pub fn with_primitive_topology(mut self, topology: PrimitiveTopology, restart_enable: bool) -> Self {
        self.primitive_topology = (topology, restart_enable);
        self
    }

    pub fn with_sample_shading(mut self, sample_shading_info: Option<SampleShadingInfo>) -> Self {
        self.sample_shading_info = sample_shading_info;
        self
    }

    /// Blend constants are used with color attachments that use 'ConstColor' or 'ConstAlpha' BlendFactors.
    /// The default constants are [0.0, 0.0, 0.0, 0.0]
    pub fn with_blend_constants(mut self, blend_constants: BlendConstants) -> Self {
        self.color_blend_info.blend_constants = blend_constants;
        self
    }

    /// Appends a color output to the pipeline.
    /// The number of color outputs of a pipeline must match exactly with the number of outputs in the fragment shader.
    pub fn with_color_output(mut self, format: Format, write_mask: WriteMask, blend_state: Option<ColorOutputBlendState>) -> Self {
        self.color_output_formats.push(format).unwrap();
        self.color_blend_info.add_attachment(write_mask, blend_state);
        self
    }

    pub fn with_depth_output(mut self, format: Format) -> Self {
        self.depth_output_format = format;
        self
    }

    pub fn with_stencil_output(mut self, format: Format) -> Self {
        self.stencil_output_format = format;
        self
    }

    pub fn with_dynamic_states(mut self, dynamic_state: &[DynamicState]) -> Self {
        self.dynamic_states.append_map(dynamic_state, |v| (*v).into()).unwrap();
        self
    }

    pub fn as_create_info(&self) -> CreateInfos<'_> {

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
            depth_bias_constant_factor: depth_bias_info.constant_factor,
            depth_bias_clamp: depth_bias_info.clamp,
            depth_bias_slope_factor: depth_bias_info.constant_factor,
            ..Default::default()
        };

        let sample_shading_info = self.sample_shading_info.unwrap_or(Default::default());

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

        let depth_stencil_info = self.depth_stencil_info.unwrap_or(Default::default());

        let stencil_test_info = depth_stencil_info.stencil_test_info.unwrap_or(Default::default());

        let depth_bounds = depth_stencil_info.depth_bounds.unwrap_or(Default::default());

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            depth_test_enable: self.depth_stencil_info.is_some().into(),
            depth_write_enable: depth_stencil_info.write_enable.into(),
            depth_compare_op: depth_stencil_info.compare_op.into(),
            depth_bounds_test_enable: depth_stencil_info.depth_bounds.is_some().into(),
            stencil_test_enable: depth_stencil_info.stencil_test_enable.into(),
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

        CreateInfos {
            input_assembly_state,
            tesellation_state,
            rasterization_state,
            multisample_state,
            depth_stencil_state,
            color_blend_state,
            dynamic_state,
            rendering_info,
        }
    }
}

impl ByteHash for GraphicsPipelineInfo {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.dynamic_states.byte_hash(hasher);
        self.color_output_formats.byte_hash(hasher);
        self.polygon_mode.as_raw().byte_hash(hasher);
        self.cull_mode.as_raw().byte_hash(hasher);
        self.front_face.as_raw().byte_hash(hasher);
        self.depth_bias_info.byte_hash(hasher);
        self.primitive_topology.0.as_raw().byte_hash(hasher);
        self.primitive_topology.1.byte_hash(hasher);
        self.sample_shading_info.byte_hash(hasher);
        self.depth_stencil_info.byte_hash(hasher);
        self.color_blend_info.byte_hash(hasher);
        self.depth_output_format.byte_hash(hasher);
        self.stencil_output_format.byte_hash(hasher);
        self.depth_clamp.byte_hash(hasher);
        self.rasterizer_discard.byte_hash(hasher);
    }
}
