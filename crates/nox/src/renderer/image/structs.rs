use core::{
    num::NonZeroU32,
    hash::Hash,
};

use ash::vk;

use crate::byte_hash::ByteHash;

use super::ComponentSwizzle;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

impl ByteHash for Dimensions {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.width.byte_hash(hasher);
        self.height.byte_hash(hasher);
        self.depth.byte_hash(hasher);
    }
}

#[derive(Default, Clone, Copy)]
pub struct ComponentMapping {
    r: ComponentSwizzle,
    g: ComponentSwizzle,
    b: ComponentSwizzle,
    a: ComponentSwizzle,
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

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy, PartialEq)]
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
        self,
        image: vk::Image,
        to: Self,
        subresource_range: ImageSubresourceRangeInfo,
    ) -> vk::ImageMemoryBarrier<'static>
    {
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
