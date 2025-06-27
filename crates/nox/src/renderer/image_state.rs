use ash::vk;

#[derive(Clone, Copy)]
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
        &self,
        image: vk::Image,
        to: &Self,
        subresource_range: vk::ImageSubresourceRange,
    ) -> vk::ImageMemoryBarrier<'_>
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
            subresource_range,
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
