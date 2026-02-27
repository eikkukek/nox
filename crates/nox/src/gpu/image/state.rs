use nox_ash::vk;

use ahash::AHashMap;

use nox_mem::{
    vec::{Vec32, Vector},
    vec32,
};

use crate::gpu::prelude::{
    subresource_state::*,
    *
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct ImageSubresourceState {
    pub stage_mask: vk::PipelineStageFlags2,
    pub access_mask: vk::AccessFlags2,
    pub layout: vk::ImageLayout,
    pub queue_family_index: u32,
    pub command_index: u32,
    pub command_timeline_value: u64,
}

impl ImageSubresourceState {

    pub fn new(
        stage_mask: vk::PipelineStageFlags2,
        access_mask: vk::AccessFlags2,
        layout: vk::ImageLayout,
        queue_family_index: u32,
        command_index: u32,
        command_timeline_value: u64,
    ) -> Self 
    {
        Self {
            stage_mask,
            access_mask,
            layout,
            queue_family_index,
            command_index,
            command_timeline_value,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ImageMemoryBarrier {
    pub src_stage_mask: vk::PipelineStageFlags2,
    pub src_access_mask: vk::AccessFlags2,
    pub dst_stage_mask: vk::PipelineStageFlags2,
    pub dst_access_mask: vk::AccessFlags2,
    pub old_layout: vk::ImageLayout,
    pub new_layout: vk::ImageLayout,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub subresource_range: vk::ImageSubresourceRange,
    pub src_command_index: u32,
    pub src_command_timeline_value: u64,
}

impl ImageMemoryBarrier {

    pub fn into_vk(self, image: vk::Image) -> vk::ImageMemoryBarrier2<'static> {
        vk::ImageMemoryBarrier2 {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER_2,
            src_stage_mask: self.src_stage_mask,
            src_access_mask: self.src_access_mask,
            dst_stage_mask: self.dst_stage_mask,
            dst_access_mask: self.dst_access_mask,
            old_layout: self.old_layout,
            new_layout: self.new_layout,
            src_queue_family_index: self.src_queue_family_index,
            dst_queue_family_index: self.dst_queue_family_index,
            image,
            subresource_range: self.subresource_range,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ImageMipRange {
    pub state: ImageSubresourceState,
    pub base_mip_level: u32,
    pub level_count: u32,
}

impl StateRange for ImageMipRange {

    type MemoryBarrier = ImageMemoryBarrier;
    type State = ImageSubresourceState;
    type SizeType = u32;

    #[inline(always)]
    fn new(
        state: Self::State,
        offset: Self::SizeType,
        size: Self::SizeType,
    ) -> Self {
        Self {
            state,
            base_mip_level: offset,
            level_count: size,
        }
    }

    #[inline(always)]
    fn memory_barrier(
        src: Self::State,
        dst: Self::State,
        offset: Self::SizeType,
        size: Self::SizeType,
    ) -> Self::MemoryBarrier {
        ImageMemoryBarrier {
            src_stage_mask: src.stage_mask,
            src_access_mask: src.access_mask,
            dst_stage_mask: dst.stage_mask,
            dst_access_mask: dst.access_mask,
            old_layout: src.layout,
            new_layout: dst.layout,
            src_queue_family_index: src.queue_family_index,
            dst_queue_family_index: dst.queue_family_index,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::empty(),
                base_mip_level: offset, level_count: size,
                base_array_layer: 0, layer_count: 1,
            },
            src_command_index: src.command_index,
            src_command_timeline_value: src.command_timeline_value,
        }
    }

    #[inline(always)]
    fn offset(&self) -> Self::SizeType {
        self.base_mip_level
    }
    
    #[inline(always)]
    fn size(&self) -> Self::SizeType {
        self.level_count
    }

    #[inline(always)]
    fn state(&self) -> Self::State {
        self.state
    }
}

#[derive(Default)]
pub(crate) struct ImageMemoryBarrierCache {
    pub(super) cache: AHashMap<(ImageAspect, u32, u32), Vec32<ImageMemoryBarrier>>,
    pub(super) touched: Vec32<(ImageAspect, u32, u32)>,
    pub(super) barriers: Vec32<ImageMemoryBarrier>,
}

impl ImageMemoryBarrierCache {

    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub(super) fn insert(
        &mut self,
        aspect: ImageAspect,
        barrier: ImageMemoryBarrier,
    ) {
        let key = (
            aspect,
            barrier.subresource_range.base_mip_level,
            barrier.subresource_range.level_count
        );
        self.cache.entry(key)
        .and_modify(|barriers| {
            if barriers.is_empty() {
                self.touched.push(key);
            }
            barriers.push(barrier);
        }).or_insert_with(|| {
            self.touched.push(key);
            vec32![barrier]
        });
    }
}
