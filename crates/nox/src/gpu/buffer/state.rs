use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct BufferState {
    pub access_mask: vk::AccessFlags2,
    pub stage_mask: vk::PipelineStageFlags2,
    pub queue_family_index: u32,
    pub command_index: u32,
    pub command_timeline_value: u64,
}

impl BufferState {

    pub fn new(
        access_mask: vk::AccessFlags2,
        stage_mask: vk::PipelineStageFlags2,
        queue_family_index: u32,
        command_index: u32,
        command_timeline_value: u64,
    ) -> Self
    {
        Self {
            access_mask,
            stage_mask,
            queue_family_index,
            command_index,
            command_timeline_value,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BufferMemoryBarrier {
    pub src_stage_mask: vk::PipelineStageFlags2,
    pub src_access_mask: vk::AccessFlags2,
    pub dst_stage_mask: vk::PipelineStageFlags2,
    pub dst_access_mask: vk::AccessFlags2,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub offset: vk::DeviceSize,
    pub size: vk::DeviceSize,
    pub src_command_index: u32,
    pub src_command_timeline_value: u64,
}

impl BufferMemoryBarrier {

    #[inline(always)]
    pub fn into_vk(&self, handle: vk::Buffer) -> vk::BufferMemoryBarrier2 {
        vk::BufferMemoryBarrier2 {
            s_type: vk::StructureType::BUFFER_MEMORY_BARRIER_2,
            src_stage_mask: self.src_stage_mask,
            src_access_mask: self.src_access_mask,
            dst_stage_mask: self.dst_stage_mask,
            dst_access_mask: self.dst_access_mask,
            src_queue_family_index: self.src_queue_family_index,
            dst_queue_family_index: self.dst_queue_family_index,
            buffer: handle,
            offset: self.offset,
            size: self.size,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct BufferRange {
    pub state: BufferState,
    pub offset: vk::DeviceSize,
    pub size: vk::DeviceSize,
}

impl StateRange for BufferRange {

    type MemoryBarrier = BufferMemoryBarrier;
    type State = BufferState;
    type SizeType = vk::DeviceSize;

    #[inline(always)]
    fn new(
        state: Self::State,
        offset: Self::SizeType,
        size: Self::SizeType,
    ) -> Self {
        Self {
            state,
            offset,
            size,
        }
    }

    #[inline(always)]
    fn memory_barrier(
        src: Self::State,
        dst: Self::State,
        offset: Self::SizeType,
        size: Self::SizeType,
    ) -> Self::MemoryBarrier {
        BufferMemoryBarrier {
            src_stage_mask: src.stage_mask,
            src_access_mask: src.access_mask,
            dst_stage_mask: dst.stage_mask,
            dst_access_mask: dst.access_mask,
            src_queue_family_index: src.queue_family_index,
            dst_queue_family_index: dst.queue_family_index,
            offset,
            size,
            src_command_index: src.command_index,
            src_command_timeline_value: src.command_timeline_value,
        }
    }

    #[inline(always)]
    fn offset(&self) -> Self::SizeType {
        self.offset
    }

    #[inline(always)]
    fn size(&self) -> Self::SizeType {
        self.size
    }

    #[inline(always)]
    fn state(&self) -> Self::State {
        self.state
    }
}

#[derive(Default)]
pub(crate) struct BufferMemoryBarrierCache {
    pub(super) cache: Vec32<BufferMemoryBarrier>,
}

impl BufferMemoryBarrierCache {

    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }
}
