use ash::vk;

use nox_mem::{AsRaw, Vector, GlobalVec};

use crate::{byte_hash::ByteHash, renderer::MSAA};

use super::*;

#[derive(Default, Clone, Copy)]
pub struct DepthBiasInfo {
    pub constant_factor: f32,
    pub clamp: f32,
    pub slope_factor: f32,
}

impl ByteHash for DepthBiasInfo {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.constant_factor.byte_hash(hasher);
        self.clamp.byte_hash(hasher);
        self.slope_factor.byte_hash(hasher);
    }
}

#[derive(Default, Clone, Copy)]
pub struct SampleShadingInfo {
    pub samples: MSAA,
    pub min_shading: f32,
    pub alpha_to_coverage: bool,
    pub alpha_to_one: bool,
}

impl ByteHash for SampleShadingInfo {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.samples.as_raw().byte_hash(hasher);
        self.min_shading.byte_hash(hasher);
    }
}

#[derive(Default, Clone, Copy)]
pub struct DepthBounds {
    pub min: f32,
    pub max: f32,
}

impl ByteHash for DepthBounds {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.min.byte_hash(hasher);
        self.max.byte_hash(hasher);
    }
}

#[derive(Clone, Copy)]
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

impl ByteHash for StencilOpState {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.fail_op.as_raw().byte_hash(hasher);
        self.pass_op.as_raw().byte_hash(hasher);
        self.depth_fail_op.as_raw().byte_hash(hasher);
        self.compare_op.as_raw().byte_hash(hasher);
        self.compare_mask.byte_hash(hasher);
        self.write_mask.byte_hash(hasher);
        self.reference.byte_hash(hasher);
    }
}

#[derive(Default, Clone, Copy)]
pub struct StencilTestInfo {
    pub front: StencilOpState,
    pub back: StencilOpState,
}

impl ByteHash for StencilTestInfo {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.front.byte_hash(hasher);
        self.back.byte_hash(hasher);
    }
}

#[derive(Clone, Copy)]
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

impl ByteHash for DepthStencilInfo {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.compare_op.as_raw().byte_hash(hasher);
        self.depth_bounds.byte_hash(hasher);
        self.stencil_test_info.byte_hash(hasher);
        self.write_enable.byte_hash(hasher);
        self.stencil_test_enable.byte_hash(hasher);
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

impl AsRaw for WriteMask {

    type Repr = u32;

    fn as_raw(self) -> Self::Repr {
        self.mask
    }
}

impl ByteHash for WriteMask {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.mask.byte_hash(hasher);
    }
}

#[derive(Clone, Copy)]
pub struct ColorOutputBlendState {
    pub color_blend_factor: (BlendFactor, BlendFactor),
    pub color_blend_op: BlendOp,
    pub alpha_blend_factor: (BlendFactor, BlendFactor),
    pub alpha_blend_op: BlendOp,
    pub blend_enabled: bool,
}

struct ColorOuputState(WriteMask, Option<ColorOutputBlendState>);

impl From<ColorOuputState> for vk::PipelineColorBlendAttachmentState {

    fn from(value: ColorOuputState) -> Self {
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

impl ByteHash for ColorOutputBlendState {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.color_blend_factor.0.as_raw().byte_hash(hasher);
        self.color_blend_factor.1.as_raw().byte_hash(hasher);
        self.color_blend_op.as_raw().byte_hash(hasher);
        self.alpha_blend_factor.0.as_raw().byte_hash(hasher);
        self.alpha_blend_factor.1.as_raw().byte_hash(hasher);
        self.alpha_blend_op.as_raw().byte_hash(hasher);
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
    color_blend_attachment_states: GlobalVec<vk::PipelineColorBlendAttachmentState>,
    hasher: blake3::Hasher,
    pub blend_constants: BlendConstants, // used in 'ConstColor' and 'ConstAlpha' BlendFactors
    pub logic_op: Option<vk::LogicOp>, // only for integer frame buffers, unused for now
}

impl ColorBlendInfo {
    
    pub fn add_attachment(&mut self, write_mask: WriteMask, blend_state: Option<ColorOutputBlendState>) {
        self.color_blend_attachment_states
            .push(
                ColorOuputState(write_mask, blend_state).into()
            )
            .unwrap();
        write_mask.byte_hash(&mut self.hasher);
        blend_state.byte_hash(&mut self.hasher);
    }

    pub fn attachments(&self) -> &[vk::PipelineColorBlendAttachmentState] {
        &self.color_blend_attachment_states
    }
}

impl ByteHash for vk::LogicOp {
    
    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.as_raw().byte_hash(hasher);
    }
}

impl ByteHash for ColorBlendInfo {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        let hash1 = hasher.clone().finalize();
        let hash2 = self.hasher.clone().finalize();
        *hasher = blake3::Hasher::new();
        hasher.update(hash1.as_bytes());
        hasher.update(hash2.as_bytes());
        self.blend_constants.0.as_slice().byte_hash(hasher);
        self.logic_op.byte_hash(hasher);
    }
}

pub struct CreateInfos<'a> {
    pub input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    pub tesellation_state: vk::PipelineTessellationStateCreateInfo<'a>,
    pub rasterization_state: vk::PipelineRasterizationStateCreateInfo<'a>,
    pub multisample_state: vk::PipelineMultisampleStateCreateInfo<'a>,
    pub depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo<'a>,
    pub color_blend_state: vk::PipelineColorBlendStateCreateInfo<'a>,
    pub dynamic_state: vk::PipelineDynamicStateCreateInfo<'a>,
    pub rendering_info: vk::PipelineRenderingCreateInfo<'a>,
}
