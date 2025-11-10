use std::sync::{Arc, RwLock};

use ash::vk;

use nox_alloc::arena_alloc::*;

use crate::renderer::{
    global_resources::*,
    *
};

pub struct ComputeCommands<'a> {
    device: Arc<ash::Device>,
    command_buffer: vk::CommandBuffer,
    global_resources: Arc<RwLock<GlobalResources>>,
    current_pipeline: Option<ComputePipelineId>,
    pub(crate) wait_semaphores: GlobalVec<(TimelineSemaphoreId, u64, PipelineStage)>,
    pub(crate) signal_semaphores: GlobalVec<(TimelineSemaphoreId, u64)>,
    tmp_alloc: &'a ArenaAlloc,
    queue_index: u32,
}

impl<'a> ComputeCommands<'a> {

    #[inline(always)]
    pub(crate) fn new(
        device: Arc<ash::Device>,
        command_buffer: vk::CommandBuffer,
        global_resources: Arc<RwLock<GlobalResources>>,
        tmp_alloc: &'a ArenaAlloc,
        queue_index: u32,
    ) -> Self
    {
        Self {
            device,
            command_buffer,
            global_resources,
            current_pipeline: None,
            wait_semaphores: Default::default(),
            signal_semaphores: Default::default(),
            tmp_alloc,
            queue_index,
        }
    }

    pub fn edit_resources(
        &mut self,
        mut f: impl FnMut(&mut GlobalResources) -> Result<(), Error>,
    ) -> Result<(), Error>
    {
        let mut g = self.global_resources.write().unwrap();
        f(&mut g)
    }

    pub fn prepare_storage_image(
        &mut self,
        id: ImageId,
    ) -> Result<(), Error>
    {
        let g = self.global_resources.read().unwrap();
        let image = g.get_image(id)?;
        image.cmd_memory_barrier(
            ImageState {
                access_flags: vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE,
                layout: vk::ImageLayout::GENERAL,
                queue_family_index: self.queue_index,
                pipeline_stage: vk::PipelineStageFlags::COMPUTE_SHADER,
            },
            self.command_buffer,
            None,
            false,
        )?;
        Ok(())
    }

    pub fn bind_pipeline(&mut self, id: ComputePipelineId) -> Result<(), Error> {
        let g = self.global_resources.read().unwrap();
        let pipeline = g.get_compute_pipeline(id)?;
        unsafe {
            self.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.handle,
            );
        }
        self.current_pipeline = Some(id);
        Ok(())
    }

    #[inline(always)]
    pub fn bind_shader_resources<F>(
        &self,
        f: F,
    ) -> Result<(), Error>
        where
            F: FnMut(u32) -> ShaderResourceId,
    {
        let guard = ArenaGuard::new(self.tmp_alloc);
        let g = self.global_resources.read().unwrap();
        let pipeline = g.get_compute_pipeline(
            self.current_pipeline.expect("attempting to bind shader resources with no pipeline attached")
        )?;
        let (layout, sets) = g.pipeline_get_shader_resource(
            pipeline.layout_id,
            &guard,
            f,
        )?;
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                layout,
                0, &sets, &[]
            );
        }
        Ok(())
    }

    #[inline(always)]
    pub fn push_constants<'b, F>(
        &self,
        f: F,
    ) -> Result<(), Error>
        where
            F: FnMut(PushConstant) -> &'a [u8]
    {
        let guard = ArenaGuard::new(self.tmp_alloc);
        let g = self.global_resources.read().unwrap();
        let pipeline = g.get_compute_pipeline(
            self.current_pipeline.expect("attempting to push constants with no pipeline attached")
        )?;
        let (layout, push_constants) = g.pipeline_get_push_constants(
            pipeline.layout_id,
            &guard,
            f,
        )?;
        for (pc, bytes) in &push_constants {
            unsafe {
                self.device.cmd_push_constants(
                    self.command_buffer, 
                    layout,
                    pc.stage.into(),
                    pc.offset,
                    bytes
                );
            }
        }
        Ok(())
    }

    #[inline(always)]
    pub fn dispatch(
        &self,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    )
    {
        assert!(self.current_pipeline.is_some(), "attempting to dispatch with no pipeline attached");
        unsafe {
            self.device.cmd_dispatch(
                self.command_buffer,
                group_count_x,
                group_count_y,
                group_count_z
            );
        }
    }

    #[inline(always)]
    pub fn wait_semaphore(&mut self, id: TimelineSemaphoreId, value: u64, stage: PipelineStage) {
        self.wait_semaphores.push((id, value, stage));
    }

    #[inline(always)]
    pub fn signal_semaphore(&mut self, id: TimelineSemaphoreId, value: u64) {
        self.signal_semaphores.push((id, value));
    }
}
