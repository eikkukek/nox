use ash::vk;

use crate::dev::{
    error::{caller, Location, Tracked},
};

use crate::gpu::*;

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PassId(pub(crate) u32);

impl Default for PassId {

    fn default() -> Self {
        Self(u32::MAX)
    }
}

#[derive(Default, Clone, Copy)]
pub struct PassInfo {
    pub max_reads: u32,
    pub max_color_writes: u32,
    pub msaa_samples: MSAA,
    pub signal_semaphores: u32,
    pub wait_semaphores: u32,
}

#[derive(Clone, Copy)]
pub struct RenderArea {
    width: u32,
    height: u32,
    offset: Offset2D,
}

impl From<RenderArea> for vk::Rect2D {

    fn from(value: RenderArea) -> Self {
        Self {
            offset: value.offset.into(),
            extent: vk::Extent2D {
                width: value.width,
                height: value.height,
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ClearColorValue {
    Float([f32; 4]),
    Int([i32; 4]),
    UInt([u32; 4]),
}

impl Default for ClearColorValue {

    fn default() -> Self {
        //Self::Float([Hashable(0.0); 4])
        Self::Int([0; 4])
    }
}

impl From<[f32; 4]> for ClearColorValue {

    fn from(value: [f32; 4]) -> Self {
        Self::Float([
            value[0],
            value[1],
            value[2],
            value[3],
        ])
    }
}

impl From<ClearColorValue> for vk::ClearColorValue {

    fn from(value: ClearColorValue) -> Self {
        match value {
            ClearColorValue::Float(v) => {
                Self {
                    float32: v,
                }
            },
            ClearColorValue::Int(v) => {
                Self {
                    int32: v,
                }
            },
            ClearColorValue::UInt(v) => {
                Self {
                    uint32: v,
                }
            },
        }
    }
}

impl From<ClearColorValue> for vk::ClearValue {

    fn from(value: ClearColorValue) -> Self {
        Self { color: value.into() }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ClearValue {
    Color(ClearColorValue),
    DepthStencil { depth: f32, stencil: u32 },
}

impl Default for ClearValue {

    fn default() -> Self {
        Self::Color(Default::default())
    }
}

impl From<ClearColorValue> for ClearValue {

    fn from(value: ClearColorValue) -> Self {
        Self::Color(value)
    }
}

impl From<ClearValue> for vk::ClearValue {

    fn from(value: ClearValue) -> Self {
        match value {
            ClearValue::Color(v) => {
                Self {
                    color: v.into(),
                }
            },
            ClearValue::DepthStencil { depth, stencil } => {
                Self {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth,
                        stencil,
                    },
                }
            },
        }
    }
}

#[must_use]
#[derive(Clone, Copy)]
pub struct ReadInfo {
    pub id: ResourceId,
    pub range: Option<ImageRangeInfo>,
    loc: Option<Location>,
}

impl ReadInfo {

    #[inline(always)]
    #[track_caller]
    pub fn new(id: ResourceId, range: Option<ImageRangeInfo>) -> Self {
        Self {
            id,
            range,
            loc: caller!(),
        }
    }
}

impl Tracked for ReadInfo {

    fn location(&self) -> Option<Location> {
        self.loc
    }
}

#[derive(Clone, Copy)]
pub struct WriteResolveInfo {
    pub id: ResourceId,
    pub mode: ResolveMode,
    pub range: Option<ImageRangeInfo>,
}

impl WriteResolveInfo {

    #[inline(always)]
    pub fn new(
        id: ResourceId,
        mode: ResolveMode,
        range: impl Into<Option<ImageRangeInfo>>,
    ) -> Self
    {
        Self {
            id,
            mode,
            range: range.into(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct WriteInfo {
    pub id: ResourceId,
    pub range: Option<ImageRangeInfo>,
    pub resolve: Option<WriteResolveInfo>,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub clear_value: ClearValue,
    loc: Option<Location>,
}

impl WriteInfo {

    #[inline(always)]
    #[track_caller]
    pub fn new(id: ResourceId) -> Self
    {
        Self {
            id,
            loc: caller!(),
            range: None,
            resolve: None,
            load_op: Default::default(),
            store_op: Default::default(),
            clear_value: Default::default(),
        }
    }

    #[inline(always)]
    pub fn with_range(&mut self, range: impl Into<Option<ImageRangeInfo>>) -> &mut Self {
        self.range = range.into();
        self
    }

    #[inline(always)]
    pub fn with_resolve(&mut self, info: impl Into<Option<WriteResolveInfo>>) -> &mut Self {
        self.resolve = info.into();
        self
    }

    #[inline(always)]
    pub fn with_load_op(&mut self, load_op: AttachmentLoadOp) -> &mut Self {
        self.load_op = load_op;
        self
    }

    #[inline(always)]
    pub fn with_store_op(&mut self, store_op: AttachmentStoreOp) -> &mut Self {
        self.store_op = store_op;
        self
    }

    #[inline(always)]
    pub fn with_clear_value(&mut self, value: impl Into<ClearValue>) -> &mut Self {
        self.clear_value = value.into();
        self
    }

    #[inline(always)]
    pub(crate) fn samples(&self) -> MSAA {
        self.id.samples()
    }
}

impl Tracked for WriteInfo {

    fn location(&self) -> Option<Location> {
        self.loc
    }
}

impl<'a> Into<WriteInfo> for &'a mut WriteInfo {

    fn into(self) -> WriteInfo {
        self.clone()
    }
}
