use nox_ash::vk;

use nox_alloc::arena::ArenaGuard;

use crate::gpu::prelude::*;

use crate::dev::error::{Result, Context};

use nox_mem::vec::{Vec32, Vector};

pub struct ComputeCommands<'a, 'b> {
    command_buffer: vk::CommandBuffer,
    context: GpuContext<'a>,
    current_pipeline: Option<ComputePipelineId>,
    wait_semaphores: Vec32<(TimelineSemaphoreId, u64, PipelineStage)>,
    signal_semaphores: Vec32<(TimelineSemaphoreId, u64)>,
    tmp_alloc: &'a ArenaGuard<'b>,
    queue_index: u32,
}

pub(crate) struct ComputeCommandsStorage {
    pub wait_semaphores: Vec32<(TimelineSemaphoreId, u64, PipelineStage)>,
    pub signal_semaphores: Vec32<(TimelineSemaphoreId, u64)>,
}

impl<'a, 'b> ComputeCommands<'a, 'b> {

    #[inline(always)]
    pub(crate) fn new(
        command_buffer: vk::CommandBuffer,
        context: GpuContext<'a>,
        tmp_alloc: &'a ArenaGuard<'b>,
        queue_index: u32,
    ) -> Self
    {
        Self {
            command_buffer,
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
        let image = self.context.get_image_mut(id)?;
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
        ).context_with(|| format_compact!(
            "image {:?} memory barrier failed",
            id,
        ))?;
        Ok(())
    }

    pub fn bind_pipeline(&mut self, id: ComputePipelineId) -> Result<()> {
        let pipeline = self.context.get_compute_pipeline(id)?;
        unsafe {
            self.context.vk().device().cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.handle(),
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
        let Some(pipeline) = self.current_pipeline else {
            return Err(Error::just_context(
                "attempting to bind shader resources with no pipeline binded",
            ))
        };
        let pipeline = self.context.get_compute_pipeline(pipeline)?;
        let sets = self.context.get_shader_set_resources(
            pipeline.shader_set(),
            self.tmp_alloc,
            f,
        )?;
        unsafe {
            self.context.vk().device().cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.shader_set().pipeline_layout(),
                0, &sets, &[]
            );
        }
        unsafe {
            self.tmp_alloc.clear();
        }
        Ok(())
    }

    #[inline(always)]
    pub fn push_constants<'c, F>(
        &self,
        f: F,
    ) -> Result<()>
        where
            F: FnMut(PushConstantRange) -> &'c [u8]
    {
        let Some(pipeline) = self.current_pipeline else {
            return Err(Error::just_context(
                "attempting to push constants with no pipeline binded",
            ))
        };
        let pipeline = self.context.get_compute_pipeline(pipeline)?;
        let push_constants = self.context.get_shader_set_push_constant_ranges(
            pipeline.shader_set(),
            self.tmp_alloc,
            f,
        )?;
        for (pc, bytes) in &push_constants {
            unsafe {
                self.context.vk().device().cmd_push_constants(
                    self.command_buffer, 
                    pipeline.shader_set().pipeline_layout(),
                    pc.stage.into(),
                    pc.offset,
                    bytes
                );
            }
        }
        unsafe {
            self.tmp_alloc.clear();
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
            self.context.vk().device().cmd_dispatch(
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

    pub(crate) fn finish(self) -> ComputeCommandsStorage {
        ComputeCommandsStorage { wait_semaphores: self.wait_semaphores, signal_semaphores: self.signal_semaphores }
    }
}
