use ash::vk;

use crate::renderer::{
    image::ImageRangeInfo,
    MSAA,
    frame_state::ResourceID,
};

use super::{AttachmentLoadOp, AttachmentStoreOp};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PassID(pub(crate) u32);

#[derive(Default, Clone, Copy)]
pub struct PassInfo {
    pub max_reads: u32,
    pub max_color_writes: u32,
    pub max_dependencies: u32,
    pub msaa_samples: MSAA,
}

#[derive(Default, Clone, Copy)]
pub struct Offset {
    x: i32,
    y: i32
}

impl From<Offset> for vk::Offset2D {

    fn from(value: Offset) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RenderArea {
    width: u32,
    height: u32,
    offset: Offset,
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

#[derive(Default, Clone, Copy, PartialEq)]
pub struct ClearDepthStencilValue {
    pub depth: f32,
    pub stencil: u32,
}

impl From<ClearDepthStencilValue> for vk::ClearDepthStencilValue {

    fn from(value: ClearDepthStencilValue) -> Self {
        Self {
            depth: value.depth,
            stencil: value.stencil,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ClearValue {
    Color(ClearColorValue),
    DepthStencil(ClearDepthStencilValue),
}

impl Default for ClearValue {

    fn default() -> Self {
        Self::Color(Default::default())
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
            ClearValue::DepthStencil(v) => {
                Self {
                    depth_stencil: v.into(),
                }
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct ReadInfo {
    pub resource_id: ResourceID,
    pub range_info: Option<ImageRangeInfo>,
}

impl ReadInfo {

    #[inline(always)]
    pub fn new(resource_id: ResourceID, range_info: Option<ImageRangeInfo>) -> Self {
        Self {
            resource_id,
            range_info,
        }
    }

    #[inline(always)]
    pub(crate) fn _vk_format(&self) -> vk::Format {
        self.range_info
            .map(|v| v.component_info
                .map(|v| v.format)
                .unwrap_or(self.resource_id.vk_format()))
            .unwrap_or(self.resource_id.vk_format())
    }
}

#[derive(Clone, Copy)]
pub struct WriteInfo {
    pub resource_id: ResourceID,
    pub range_info: Option<ImageRangeInfo>,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub clear_value: ClearValue,
}

impl WriteInfo {

    #[inline(always)]
    pub fn new(
        resource_id: ResourceID,
        range_info: Option<ImageRangeInfo>,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
        clear_value: ClearValue,
    ) -> Self
    {
        Self {
            resource_id,
            range_info,
            load_op,
            store_op,
            clear_value,
        }
    }

    #[inline(always)]
    pub(crate) fn _vk_format(&self) -> vk::Format {
        self.range_info
            .map(|v| v.component_info
                .map(|v| v.format)
                .unwrap_or(self.resource_id.vk_format()))
            .unwrap_or(self.resource_id.vk_format())
    }

    #[inline(always)]
    pub(crate) fn samples(&self) -> MSAA {
        self.resource_id.samples()
    }
}
