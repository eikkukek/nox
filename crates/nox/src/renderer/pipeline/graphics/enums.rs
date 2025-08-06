use ash::vk;
use nox_mem::AsRaw;

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum DynamicState {
    CullMode = vk::DynamicState::CULL_MODE.as_raw(),
    FrontFace = vk::DynamicState::FRONT_FACE.as_raw(),
    DepthBias = vk::DynamicState::DEPTH_BIAS.as_raw(),
    DepthTestEnable = vk::DynamicState::DEPTH_TEST_ENABLE.as_raw(),
    DepthWriteEnable = vk::DynamicState::DEPTH_WRITE_ENABLE.as_raw(),
    DepthCompareOp = vk::DynamicState::DEPTH_COMPARE_OP.as_raw(),
    StencilTestEnable = vk::DynamicState::STENCIL_TEST_ENABLE.as_raw(),
    StencilOp = vk::DynamicState::STENCIL_OP.as_raw(),
    StencilWriteMask = vk::DynamicState::STENCIL_WRITE_MASK.as_raw(),
    StencilCompareMask = vk::DynamicState::STENCIL_COMPARE_MASK.as_raw(),
    StencilReference = vk::DynamicState::STENCIL_REFERENCE.as_raw(),
}

impl From<DynamicState> for vk::DynamicState {

    fn from(value: DynamicState) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum PolygonMode {
    #[default]
    Fill = vk::PolygonMode::FILL.as_raw(),
    Line = vk::PolygonMode::LINE.as_raw(),
}

impl From<PolygonMode> for vk::PolygonMode {

    fn from(value: PolygonMode) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(u32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum CullMode {
    None = vk::CullModeFlags::NONE.as_raw(),
    #[default]
    Front = vk::CullModeFlags::FRONT.as_raw(),
    Back = vk::CullModeFlags::BACK.as_raw(),
    All = vk::CullModeFlags::FRONT_AND_BACK.as_raw(),
}

impl From<CullMode> for vk::CullModeFlags {
    
    fn from(value: CullMode) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum FrontFace {
    #[default]
    CounterClockwise = vk::FrontFace::COUNTER_CLOCKWISE.as_raw(),
    ClockWise = vk::FrontFace::CLOCKWISE.as_raw(),
}

impl From<FrontFace> for vk::FrontFace {

    fn from(value: FrontFace) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum PrimitiveTopology {
    PointList = vk::PrimitiveTopology::POINT_LIST.as_raw(),
    LineList = vk::PrimitiveTopology::LINE_LIST.as_raw(),
    LineStrip = vk::PrimitiveTopology::LINE_STRIP.as_raw(),
    #[default]
    TriangleList = vk::PrimitiveTopology::TRIANGLE_LIST.as_raw(),
    TriangleStrip = vk::PrimitiveTopology::TRIANGLE_STRIP.as_raw(),
}

impl PrimitiveTopology {

    pub fn can_restart(self) -> bool {
        match self {
            Self::LineStrip | Self::TriangleStrip => true,
            _ => false
        }
    }
}

impl From<PrimitiveTopology> for vk::PrimitiveTopology {

    fn from(value: PrimitiveTopology) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum StencilOp {
    Keep = vk::StencilOp::KEEP.as_raw(),
    Zero = vk::StencilOp::ZERO.as_raw(),
    Replace = vk::StencilOp::REPLACE.as_raw(),
    IncrementClamp = vk::StencilOp::INCREMENT_AND_CLAMP.as_raw(),
    DecrementClamp = vk::StencilOp::DECREMENT_AND_CLAMP.as_raw(),
    Invert = vk::StencilOp::INVERT.as_raw(),
    IncrementWrap = vk::StencilOp::INCREMENT_AND_WRAP.as_raw(),
    DecrementWrap = vk::StencilOp::DECREMENT_AND_WRAP.as_raw(),
}

impl From<StencilOp> for vk::StencilOp {

    fn from(value: StencilOp) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum BlendFactor {
    Zero = vk::BlendFactor::ZERO.as_raw(),
    One = vk::BlendFactor::ONE.as_raw(),
    SrcColor = vk::BlendFactor::SRC_COLOR.as_raw(),
    OneMinusSrcColor = vk::BlendFactor::ONE_MINUS_SRC_COLOR.as_raw(),
    DstColor = vk::BlendFactor::DST_COLOR.as_raw(),
    OneMinusDstColor = vk::BlendFactor::ONE_MINUS_DST_COLOR.as_raw(),
    SrcAlpha = vk::BlendFactor::SRC_ALPHA.as_raw(),
    OneMinusSrcAlpha = vk::BlendFactor::ONE_MINUS_SRC_ALPHA.as_raw(),
    DstAlpha = vk::BlendFactor::DST_ALPHA.as_raw(),
    OneMinusDstAlpha = vk::BlendFactor::ONE_MINUS_DST_ALPHA.as_raw(),
    ConstColor = vk::BlendFactor::CONSTANT_COLOR.as_raw(),
    OneMinusConstColor = vk::BlendFactor::ONE_MINUS_CONSTANT_COLOR.as_raw(),
    ConstAlpha = vk::BlendFactor::CONSTANT_ALPHA.as_raw(),
    OneMinusConstAlpha = vk::BlendFactor::ONE_MINUS_CONSTANT_ALPHA.as_raw(),
}

impl BlendFactor {

    pub(crate) fn from_vk(value: vk::BlendFactor) -> Option<Self> {
        match value.as_raw() {
            x if x == Self::Zero as i32 => Some(Self::Zero),
            x if x == Self::One as i32 => Some(Self::One),
            x if x == Self::SrcColor as i32 => Some(Self::SrcColor),
            x if x == Self::OneMinusSrcColor as i32 => Some(Self::OneMinusSrcColor),
            x if x == Self::DstColor as i32 => Some(Self::DstColor),
            x if x == Self::OneMinusDstColor as i32 => Some(Self::OneMinusDstColor),
            x if x == Self::SrcAlpha as i32 => Some(Self::SrcAlpha),
            x if x == Self::OneMinusSrcAlpha as i32 => Some(Self::OneMinusSrcAlpha),
            x if x == Self::DstAlpha as i32 => Some(Self::DstAlpha),
            x if x == Self::OneMinusDstAlpha as i32 => Some(Self::OneMinusDstAlpha),
            x if x == Self::ConstColor as i32 => Some(Self::ConstColor),
            x if x == Self::OneMinusConstColor as i32 => Some(Self::OneMinusConstColor),
            x if x == Self::ConstAlpha as i32 => Some(Self::ConstAlpha),
            x if x == Self::OneMinusConstAlpha as i32 => Some(Self::OneMinusConstAlpha),
            _ => None,
        }
    }
}

impl From<BlendFactor> for vk::BlendFactor {

    fn from(value: BlendFactor) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum BlendOp {
    Add = vk::BlendOp::ADD.as_raw(),
    Sub = vk::BlendOp::SUBTRACT.as_raw(),
    SubRev = vk::BlendOp::REVERSE_SUBTRACT.as_raw(),
    Min = vk::BlendOp::MIN.as_raw(),
    Max = vk::BlendOp::MAX.as_raw(),
}

impl BlendOp {

    pub(crate) fn from_vk(value: vk::BlendOp) -> Option<Self> {
        match value.as_raw() {
            x if x == Self::Add as i32 => Some(Self::Add),
            x if x == Self::Sub as i32 => Some(Self::Sub),
            x if x == Self::SubRev as i32 => Some(Self::SubRev),
            x if x == Self::Min as i32 => Some(Self::Min),
            x if x == Self::Max as i32 => Some(Self::Max),
            _ => None,
        }
    }
}

impl From<BlendOp> for vk::BlendOp {

    fn from(value: BlendOp) -> Self {
        Self::from_raw(value.as_raw())
    }
}
