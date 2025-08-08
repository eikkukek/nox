use core::hash::{Hash, Hasher};

use ash::vk;

use nox_mem::{Hashable, AsRaw, vec_types::{Vector, GlobalVec, FixedVec}, Allocator};

use crate::renderer::*;

use super::*;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepthBiasInfo {
    pub constant_factor: Hashable<f32>,
    pub clamp: Hashable<f32>,
    pub slope_factor: Hashable<f32>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampleShadingInfo {
    pub samples: MSAA,
    pub min_shading: Hashable<f32>,
    pub alpha_to_coverage: bool,
    pub alpha_to_one: bool,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepthBounds {
    pub min: Hashable<f32>,
    pub max: Hashable<f32>,
}

impl DepthBounds {

    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min: min.into(),
            max: max.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StencilOpState {
    pub fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub compare_op: CompareOp,
    pub compare_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

impl Default for StencilOpState {

    fn default() -> Self {
        Self {
            fail_op: StencilOp::Zero,
            pass_op: StencilOp::Zero,
            depth_fail_op: StencilOp::Zero,
            compare_op: CompareOp::Never,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        }
    }
}

impl From<StencilOpState> for vk::StencilOpState {

    fn from(value: StencilOpState) -> Self {
        Self {
            fail_op: value.fail_op.into(),
            pass_op: value.pass_op.into(),
            depth_fail_op: value.depth_fail_op.into(),
            compare_op: value.compare_op.into(),
            compare_mask: value.compare_mask.into(),
            write_mask: value.write_mask.into(),
            reference: value.reference.into(),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StencilTestInfo {
    pub front: StencilOpState,
    pub back: StencilOpState,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepthStencilInfo {
    pub compare_op: CompareOp,
    pub depth_bounds: Option<DepthBounds>,
    pub stencil_test_info: Option<StencilTestInfo>,
    pub write_enable: bool,
    pub stencil_test_enable: bool,
}

impl Default for DepthStencilInfo {

    fn default() -> Self {
        Self {
            compare_op: CompareOp::Never,
            depth_bounds: None,
            stencil_test_info: None,
            write_enable: false,
            stencil_test_enable: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WriteMask {
    mask: u32,
}

impl WriteMask {

    pub fn all() -> Self {
        Self {
            mask: vk::ColorComponentFlags::RGBA.as_raw(),
        }
    }

    pub fn none() -> Self {
        Self {
            mask: 0,
        }
    }

    pub fn with_r_bit(&mut self, bit: bool) -> &mut Self {
        self.mask |= (bit as u32) << 0;
        self
    }

    pub fn with_g_bit(&mut self, bit: bool) -> &mut Self {
        self.mask |= (bit as u32) << 1;
        self
    }

    pub fn with_b_bit(&mut self, bit: bool) -> &mut Self {
        self.mask |= (bit as u32) << 2;
        self
    }

    pub fn with_a_bit(&mut self, bit: bool) -> &mut Self {
        self.mask |= (bit as u32) << 3;
        self
    }
}

impl Default for WriteMask {

    fn default() -> Self {
        Self::all()
    }
}

impl From<WriteMask> for vk::ColorComponentFlags {

    fn from(value: WriteMask) -> Self {
        Self::from_raw(value.mask)
    }
}

impl From<vk::ColorComponentFlags> for WriteMask {

    fn from(value: vk::ColorComponentFlags) -> Self {
        Self {
            mask: value.as_raw(),
        }
    }
}

impl AsRaw for WriteMask {

    type Repr = u32;

    fn as_raw(self) -> Self::Repr {
        self.mask
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorOutputBlendState {
    pub color_blend_factor: (BlendFactor, BlendFactor),
    pub color_blend_op: BlendOp,
    pub alpha_blend_factor: (BlendFactor, BlendFactor),
    pub alpha_blend_op: BlendOp,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ColorOutputState(WriteMask, Option<ColorOutputBlendState>);

impl From<ColorOutputState> for vk::PipelineColorBlendAttachmentState {

    fn from(value: ColorOutputState) -> Self {
        match value.1 {
            None => {
                Self {
                    blend_enable: 0,
                    color_write_mask: value.0.into(),
                    ..Default::default()
                }
            },
            Some(b) => {
                Self {
                    blend_enable: 1,
                    src_color_blend_factor: b.color_blend_factor.0.into(),
                    dst_color_blend_factor: b.color_blend_factor.1.into(),
                    color_blend_op: b.color_blend_op.into(),
                    src_alpha_blend_factor: b.alpha_blend_factor.0.into(),
                    dst_alpha_blend_factor: b.alpha_blend_factor.1.into(),
                    alpha_blend_op: b.alpha_blend_op.into(),
                    color_write_mask: value.0.into(),
                }
            }
        }
    }
}

impl From<vk::PipelineColorBlendAttachmentState> for ColorOutputState {

    fn from(value: vk::PipelineColorBlendAttachmentState) -> Self {
        let mut s = Self(value.color_write_mask.into(), None);
        if value.blend_enable == 1 {
            s.1 = Some(ColorOutputBlendState {
                color_blend_factor: (
                    BlendFactor::from_vk(value.src_color_blend_factor).unwrap(),
                    BlendFactor::from_vk(value.dst_color_blend_factor).unwrap(),
                ),
                color_blend_op: BlendOp::from_vk(value.color_blend_op).unwrap(),
                alpha_blend_factor:  (
                    BlendFactor::from_vk(value.src_alpha_blend_factor).unwrap(),
                    BlendFactor::from_vk(value.dst_alpha_blend_factor).unwrap(),
                ),
                alpha_blend_op: BlendOp::from_vk(value.alpha_blend_op).unwrap(),
            });
        }
        s
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlendConstants([Hashable<f32>; 4]);

impl From<BlendConstants> for [f32; 4] {

    fn from(value: BlendConstants) -> Self {
        [
            value.0[0].to_inner(),
            value.0[1].to_inner(),
            value.0[2].to_inner(),
            value.0[3].to_inner(),
        ]
    }
}

#[derive(Default, Clone)]
pub struct ColorBlendInfo {
    color_blend_attachment_states: GlobalVec<vk::PipelineColorBlendAttachmentState>,
    pub blend_constants: BlendConstants, // used in 'ConstColor' and 'ConstAlpha' BlendFactors
    pub logic_op: Option<vk::LogicOp>, // only for integer frame buffers, unused for now
}

impl ColorBlendInfo {
    
    pub fn add_attachment(&mut self, write_mask: WriteMask, blend_state: Option<ColorOutputBlendState>) {
        self.color_blend_attachment_states
            .push(
                ColorOutputState(write_mask, blend_state).into()
            )
            .unwrap();
    }

    pub fn attachments(&self) -> &[vk::PipelineColorBlendAttachmentState] {
        &self.color_blend_attachment_states
    }
}

impl Hash for ColorBlendInfo {

    fn hash<H: Hasher>(&self, state: &mut H) {
        self.color_blend_attachment_states.len().hash(state);
        for output in self.color_blend_attachment_states.iter()
            .map(|v| ColorOutputState::from(*v))
        {
            output.hash(state);
        }
        self.blend_constants.hash(state);
        self.logic_op.hash(state);
    }
}

impl PartialEq for ColorBlendInfo {

    fn eq(&self, other: &Self) -> bool {
        if self.color_blend_attachment_states.len() !=
            other.color_blend_attachment_states.len()
        {
            return false
        }
        for (i, output) in self.color_blend_attachment_states.iter()
            .map(|v| ColorOutputState::from(*v)).enumerate()
        {
            if output != other.color_blend_attachment_states[i].into() {
                return false
            }
        }
        self.blend_constants == other.blend_constants &&
        self.logic_op == other.logic_op
    }
}

impl Eq for ColorBlendInfo {}

pub(crate) struct CreateInfos<'a, Alloc: Allocator> {
    pub shader_stage_infos: FixedVec<'a, vk::PipelineShaderStageCreateInfo<'static>, Alloc>,
    pub _vertex_input_bindings: FixedVec<'a, vk::VertexInputBindingDescription, Alloc>,
    pub _vertex_input_attributes: FixedVec<'a, vk::VertexInputAttributeDescription, Alloc>,
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
}
