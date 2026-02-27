use nox_ash::vk;

use nox_mem::{
    AsRaw,
    vec::{Vector, Vec32},
};

use crate::gpu::prelude::*;

use super::*;

#[derive(Default, Clone, Copy)]
pub struct DepthBiasInfo {
    pub constant_factor: f32,
    pub clamp: f32,
    pub slope_factor: f32,
}

impl DepthBiasInfo {

    #[inline(always)]
    pub fn new(
        constant_factor: f32,
        clamp: f32,
        slope_factor: f32,
    ) -> Self {
        Self {
            constant_factor: constant_factor.into(),
            clamp: clamp.into(),
            slope_factor: slope_factor.into(),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct SampleShadingInfo {
    pub samples: MsaaSamples,
    pub min_shading: f32,
    pub alpha_to_coverage: bool,
    pub alpha_to_one: bool,
}

impl SampleShadingInfo {

    #[inline(always)]
    pub fn new(
        samples: MsaaSamples,
        min_shading: f32,
        alpha_to_coverage: bool,
        alpha_to_one: bool,
    ) -> Self
    {
        Self {
            samples,
            min_shading: min_shading.into(),
            alpha_to_coverage,
            alpha_to_one,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct DepthBounds {
    pub min: f32,
    pub max: f32,
}

impl DepthBounds {

    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min: min.into(),
            max: max.into(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct StencilOpState {
    /// Operation performed when stencil test fails
    pub fail_op: StencilOp,
    /// Operation performed when both stencil and depth test pass
    pub pass_op: StencilOp,
    /// Operation performed when stencil test passes but depth test fails
    pub depth_fail_op: StencilOp,
    /// Compare operation for the stencil test
    pub compare_op: CompareOp,
    /// Bitmask applied to stencil and reference before comparison
    pub compare_mask: u32,
    /// Bitmask controlling which bits can be written to stencil buffer
    pub write_mask: u32,
    /// The bits which are compared against the stencil buffer
    pub reference: u32,
}

impl Default for StencilOpState {

    fn default() -> Self {
        Self {
            fail_op: StencilOp::Keep,
            pass_op: StencilOp::Keep,
            depth_fail_op: StencilOp::Keep,
            compare_op: CompareOp::Always,
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
            compare_mask: value.compare_mask,
            write_mask: value.write_mask,
            reference: value.reference,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct StencilTestInfo {
    pub front: StencilOpState,
    pub back: StencilOpState,
}

#[derive(Clone, Copy)]
pub struct DepthStencilInfo {
    pub compare_op: CompareOp,
    pub depth_bounds: Option<DepthBounds>,
    pub stencil_test_info: Option<StencilTestInfo>,
    pub write_enable: bool,
}

impl Default for DepthStencilInfo {

    fn default() -> Self {
        Self {
            compare_op: CompareOp::Never,
            depth_bounds: None,
            stencil_test_info: None,
            write_enable: false,
        }
    }
}

#[derive(Clone, Copy)]
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
        self.mask |= bit as u32;
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

#[derive(Clone, Copy)]
pub struct ColorOutputBlendState {
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op: BlendOp,
    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op: BlendOp,
}

#[derive(Clone, Copy)]
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
                    src_color_blend_factor: b.src_color_blend_factor.into(),
                    dst_color_blend_factor: b.dst_color_blend_factor.into(),
                    color_blend_op: b.color_blend_op.into(),
                    src_alpha_blend_factor: b.src_alpha_blend_factor.into(),
                    dst_alpha_blend_factor: b.dst_alpha_blend_factor.into(),
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
                src_color_blend_factor: BlendFactor::from_vk(value.src_color_blend_factor).unwrap(),
                dst_color_blend_factor: BlendFactor::from_vk(value.dst_alpha_blend_factor).unwrap(),
                color_blend_op: BlendOp::from_vk(value.color_blend_op).unwrap(),
                src_alpha_blend_factor: BlendFactor::from_vk(value.src_alpha_blend_factor).unwrap(),
                dst_alpha_blend_factor: BlendFactor::from_vk(value.dst_alpha_blend_factor).unwrap(),
                alpha_blend_op: BlendOp::from_vk(value.alpha_blend_op).unwrap(),
            });
        }
        s
    }
}

#[derive(Default, Clone, Copy)]
pub struct BlendConstants([f32; 4]);

impl From<BlendConstants> for [f32; 4] {

    fn from(value: BlendConstants) -> Self {
        value.0
    }
}

#[derive(Default, Clone)]
pub struct ColorBlendInfo {
    color_blend_attachment_states: Vec32<vk::PipelineColorBlendAttachmentState>,
    pub blend_constants: BlendConstants, // used in 'ConstColor' and 'ConstAlpha' BlendFactors
    pub logic_op: Option<vk::LogicOp>, // only for integer frame buffers, unused for now
}

impl ColorBlendInfo {
    
    pub fn add_attachment(&mut self, write_mask: WriteMask, blend_state: Option<ColorOutputBlendState>) {
        self.color_blend_attachment_states
            .push(
                ColorOutputState(write_mask, blend_state).into()
            );
    }

    pub fn attachments(&self) -> &[vk::PipelineColorBlendAttachmentState] {
        &self.color_blend_attachment_states
    }
}
