use core::{
    num::NonZeroU32,
    hash::Hash,
};

use ash::vk;

use super::*;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Dimensions {

    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    pub fn zero(&self) -> bool {
        self.width == 0 ||
        self.height == 0 ||
        self.depth == 0
    }

    pub fn texel_count(&self) -> vk::DeviceSize {
        self.width as vk::DeviceSize *
        self.height as vk::DeviceSize *
        self.depth as vk::DeviceSize
    }
}

impl From<Dimensions> for vk::Extent3D {

    fn from(value: Dimensions) -> Self {
        vk::Extent3D {
            width: value.width,
            height: value.height,
            depth: value.depth,
        }
    }
}

impl From<vk::Extent2D> for Dimensions {

    fn from(value: vk::Extent2D) -> Self {
        Dimensions::new(
            value.width,
            value.height,
            1
        )
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Offset {

    pub fn new(x: i32, y: i32, z: i32) -> Self
    {
        Self {x, y, z}
    }
}

impl From<Offset> for vk::Offset3D {

    fn from(value: Offset) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentMapping {
    pub r: ComponentSwizzle,
    pub g: ComponentSwizzle,
    pub b: ComponentSwizzle,
    pub a: ComponentSwizzle,
}

impl From<ComponentMapping> for vk::ComponentMapping {

    fn from(value: ComponentMapping) -> Self {
        Self {
            r: value.r.into(),
            g: value.g.into(),
            b: value.b.into(),
            a: value.a.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSubresourceRangeInfo {
    pub aspect_mask: u32,
    pub base_mip_level: u32,
    pub level_count: NonZeroU32,
    pub base_array_layer: u32,
    pub layer_count: NonZeroU32,
}

impl ImageSubresourceRangeInfo {

    pub fn new(
        aspect_mask: u32,
        base_mip_level: u32,
        level_count: u32,
        base_array_layer: u32,
        layer_count: u32,
    ) -> Option<Self>
    {
        Some(Self {
            aspect_mask,
            base_mip_level,
            level_count: NonZeroU32::new(level_count)?,
            base_array_layer,
            layer_count: NonZeroU32::new(layer_count)?,
        })
    }

    pub fn overlaps(self, other: Self) -> bool {
        if self.base_mip_level < other.base_mip_level + other.level_count.get() &&
            other.base_mip_level < self.base_mip_level + self.level_count.get() &&
            self.base_array_layer < other.base_array_layer + other.layer_count.get() &&
            other.base_array_layer < self.base_array_layer + self.layer_count.get()
        {
            return self.aspect_mask & other.aspect_mask != 0
        }
        false
    }
}

impl Default for ImageSubresourceRangeInfo {

    fn default() -> Self {
        Self {
            aspect_mask: 0,
            base_mip_level: 0,
            level_count: NonZeroU32::new(1).unwrap(),
            base_array_layer: 0,
            layer_count: NonZeroU32::new(1).unwrap(),
        }
    }
}

impl From<ImageSubresourceRangeInfo> for vk::ImageSubresourceRange {

    fn from(value: ImageSubresourceRangeInfo) -> Self {
        Self {
            aspect_mask: vk::ImageAspectFlags::from_raw(value.aspect_mask),
            base_mip_level: value.base_mip_level,
            level_count: value.level_count.get(),
            base_array_layer: value.base_array_layer,
            layer_count: value.layer_count.get(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSubresourceLayers {
    pub aspect_mask: u32,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: NonZeroU32,
}

impl ImageSubresourceLayers {

    pub fn new<A: Into<u32>>(
        aspect_mask: A,
        mip_level: u32,
        base_array_layer: u32,
        layer_count: u32,
    ) -> Option<Self> {
        Some(Self {
            aspect_mask: aspect_mask.into(),
            mip_level,
            base_array_layer,
            layer_count: NonZeroU32::new(layer_count)?,
        })
    }
}

impl From<ImageSubresourceLayers> for vk::ImageSubresourceLayers {

    fn from(value: ImageSubresourceLayers) -> Self {
        Self {
            aspect_mask: vk::ImageAspectFlags::from_raw(value.aspect_mask),
            mip_level: value.mip_level,
            base_array_layer: value.base_array_layer,
            layer_count: value.layer_count.get(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentInfo {
    pub component_mapping: ComponentMapping,
    pub(crate) format: vk::Format,
}

impl ComponentInfo {

    pub fn new<F: Format>(
        component_mapping: ComponentMapping,
        format: F,
    ) -> Self
    {
        Self {
            component_mapping,
            format: format.as_vk_format(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageRangeInfo {
    pub subresource_info: ImageSubresourceRangeInfo,
    pub component_info: Option<ComponentInfo>,
}

impl ImageRangeInfo {

    pub fn new(
        subresource_info: ImageSubresourceRangeInfo,
        component_info: Option<ComponentInfo>,
    ) -> Self
    {
        Self {
            subresource_info,
            component_info,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ImageState {
    pub access_flags: vk::AccessFlags,
    pub layout: vk::ImageLayout,
    pub queue_family_index: u32,
    pub pipeline_stage: vk::PipelineStageFlags,
}

impl ImageState {

    pub fn new(
        access_flags: vk::AccessFlags,
        layout: vk::ImageLayout,
        queue_family_index: u32,
        pipeline_stage: vk::PipelineStageFlags,
    ) -> Self 
    {
        Self {
            access_flags,
            layout,
            queue_family_index,
            pipeline_stage,
        }
    }

    pub fn access_flags(&mut self, access_flags: vk::AccessFlags) {
        self.access_flags = access_flags
    }

    pub fn layout(&mut self, layout: vk::ImageLayout) {
        self.layout = layout
    }

    pub fn queue_family_index(&mut self, queue_index: u32) {
        self.queue_family_index = queue_index
    }

    pub fn pipeline_stage(&mut self, pipeline_stage: vk::PipelineStageFlags) {
        self.pipeline_stage = pipeline_stage
    }

    pub fn to_memory_barrier(
        mut self,
        image: vk::Image,
        mut to: Self,
        subresource_range: ImageSubresourceRangeInfo,
    ) -> vk::ImageMemoryBarrier<'static>
    {
        if self.queue_family_index == vk::QUEUE_FAMILY_IGNORED ||
            to.queue_family_index == vk::QUEUE_FAMILY_IGNORED
        {
            self.queue_family_index = vk::QUEUE_FAMILY_IGNORED;
            to.queue_family_index = vk::QUEUE_FAMILY_IGNORED;
        }
        vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            src_access_mask: self.access_flags,
            dst_access_mask: to.access_flags,
            old_layout: self.layout,
            new_layout: to.layout,
            src_queue_family_index: self.queue_family_index,
            dst_queue_family_index: to.queue_family_index,
            image: image,
            subresource_range: subresource_range.into(),
            ..Default::default()
        }
    }
}

impl Default for ImageState {

    fn default() -> Self {
        Self {
            access_flags: vk::AccessFlags::NONE,
            layout: vk::ImageLayout::UNDEFINED,
            queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            pipeline_stage: vk::PipelineStageFlags::TOP_OF_PIPE,
        }
    }
}
