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
    pub resource_id: ResourceId,
    pub range_info: Option<ImageRangeInfo>,
    loc: Option<Location>,
}

impl ReadInfo {

    #[inline(always)]
    #[track_caller]
    pub fn new(resource_id: ResourceId, range_info: Option<ImageRangeInfo>) -> Self {
        Self {
            resource_id,
            range_info,
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
pub struct WriteInfo {
    pub main_id: ResourceId,
    pub range_info: Option<ImageRangeInfo>,
    pub resolve: Option<(ResourceId, ResolveMode)>,
    pub resolve_range_info: Option<ImageRangeInfo>,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub clear_value: ClearValue,
    loc: Option<Location>,
}

impl WriteInfo {

    #[inline(always)]
    #[track_caller]
    pub fn new(
        main_id: ResourceId,
        range_info: Option<ImageRangeInfo>,
        resolve: Option<(ResourceId, ResolveMode)>,
        resolve_range_info: Option<ImageRangeInfo>,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
        clear_value: impl Into<ClearValue>,
    ) -> Self
    {
        Self {
            main_id,
            range_info,
            resolve,
            resolve_range_info,
            load_op,
            store_op,
            clear_value: clear_value.into(),
            loc: caller!(),
        }
    }

    #[inline(always)]
    pub(crate) fn samples(&self) -> MSAA {
        self.main_id.samples()
    }
}

impl Tracked for WriteInfo {

    fn location(&self) -> Option<Location> {
        self.loc
    }
}
