use std::sync::Arc;

use ash::vk;

use nox_alloc::arena_alloc::*;

use crate::gpu::*;

use crate::dev::error::{Result, Context};

pub struct ComputeCommands<'a> {
    command_buffer: vk::CommandBuffer,
    context: GpuContext<'a>,
    device: Arc<ash::Device>,
    current_pipeline: Option<ComputePipelineId>,
    pub(crate) wait_semaphores: GlobalVec<(TimelineSemaphoreId, u64, PipelineStage)>,
    pub(crate) signal_semaphores: GlobalVec<(TimelineSemaphoreId, u64)>,
    tmp_alloc: &'a ArenaAlloc,
    queue_index: u32,
}

impl<'a> ComputeCommands<'a> {

    #[inline(always)]
    pub(crate) fn new(
        command_buffer: vk::CommandBuffer,
        context: GpuContext<'a>,
        tmp_alloc: &'a ArenaAlloc,
        queue_index: u32,
    ) -> Self
    {
        Self {
            command_buffer,
            device: context.device(),
            context,
            current_pipeline: None,
            wait_semaphores: Default::default(),
            signal_semaphores: Default::default(),
            tmp_alloc,
            queue_index,
        }
    }

    #[inline(always)]
    pub fn gpu(&mut self) -> &mut GpuContext<'a> {
        &mut self.context
    }

    pub fn prepare_storage_image(
        &mut self,
        id: ImageId,
    ) -> Result<()>
    {
        let image = self.context.get_image(id)?;
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
        ).context("image memory barrier failed")?;
        Ok(())
    }

    pub fn bind_pipeline(&mut self, id: ComputePipelineId) -> Result<()> {
        let pipeline = self.context.get_compute_pipeline(id)?;
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
    ) -> Result<()>
        where
            F: FnMut(u32) -> ShaderResourceId,
    {
        let guard = ArenaGuard::new(self.tmp_alloc);
        let Some(pipeline) = self.current_pipeline else {
            return Err(Error::just_context(
                "attempting to bind shader resources with no pipeline binded",
            ))
        };
        let pipeline = self.context.get_compute_pipeline(pipeline)?;
        let (layout, sets) = self.context.pipeline_get_shader_resource(
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
    ) -> Result<()>
        where
            F: FnMut(PushConstant) -> &'a [u8]
    {
        let guard = ArenaGuard::new(self.tmp_alloc);
        let Some(pipeline) = self.current_pipeline else {
            return Err(Error::just_context(
                "attempting to push constants with no pipeline binded",
            ))
        };
        let pipeline = self.context.get_compute_pipeline(pipeline)?;
        let (layout, push_constants) = self.context.pipeline_get_push_constants(
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
    ) -> Result<()>
    {
        if self.current_pipeline.is_none() {
            return Err(Error::just_context("attempting to dispatch with no pipeline binded"))
        }
        unsafe {
            self.device.cmd_dispatch(
                self.command_buffer,
                group_count_x,
                group_count_y,
                group_count_z
            );
        }
        Ok(())
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
