use ash::vk::QUEUE_FAMILY_IGNORED;

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct BufferState {
    pub access_flags: vk::AccessFlags,
    pub queue_family_index: u32,
    pub pipeline_stage: vk::PipelineStageFlags,
}

impl BufferState {

    pub fn new(
        access_flags: vk::AccessFlags,
        queue_family_index: u32,
        pipeline_stage: vk::PipelineStageFlags,
    ) -> Self
    {
        Self {
            access_flags,
            queue_family_index,
            pipeline_stage,
        }
    }

    pub fn to_memory_barrier(
        mut self,
        buffer: vk::Buffer,
        mut to: Self,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) -> vk::BufferMemoryBarrier<'static>
    {
        if self.queue_family_index == QUEUE_FAMILY_IGNORED ||
            to.queue_family_index == QUEUE_FAMILY_IGNORED 
        {
            self.queue_family_index = QUEUE_FAMILY_IGNORED;
            to.queue_family_index = QUEUE_FAMILY_IGNORED;
        }
        vk::BufferMemoryBarrier {
            s_type: vk::StructureType::BUFFER_MEMORY_BARRIER,
            src_access_mask: self.access_flags,
            dst_access_mask: to.access_flags,
            src_queue_family_index: self.queue_family_index,
            dst_queue_family_index: to.queue_family_index,
            buffer,
            offset,
            size,
            ..Default::default()
        }
    }
}
