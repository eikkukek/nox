use std::sync::Arc;

use ash::vk;

use nox_mem::vec_types::{Vector, ArrayVec};

use nox_alloc::arena_alloc::*;

use crate::gpu::*;

use super::*;

#[derive(Clone, Copy, Debug)]
pub struct DrawInfo {
    pub first_index: u32,
    pub index_count: u32,
    pub index_type: IndexType,
    pub vertex_offset: i32,
    pub first_instance: u32,
    pub instance_count: u32,
    pub first_binding: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct DrawBufferInfo {
    pub id: BufferId,
    pub offset: u64,
}

impl DrawBufferInfo {

    pub fn new(id: BufferId, offset: u64) -> Self {
        Self {
            id,
            offset,
        }
    }
}

impl Default for DrawInfo {

    fn default() -> Self {
        Self {
            first_index: 0,
            index_count: 0,
            index_type: Default::default(),
            vertex_offset: 0,
            first_instance: 0,
            instance_count: 1,
            first_binding: 0,
        }
    }
}

pub(crate) struct RenderCommandsStorage {
    transfer_command_pool: Arc<TransientCommandPool>,
    graphics_command_pool: Arc<TransientCommandPool>,
    pub(crate) transfer_commands: GlobalVec<TransferCommandsStorage>,
}

impl RenderCommandsStorage {

    pub fn new(
        device: Arc<ash::Device>,
        queue_family_indices: QueueFamilyIndices,
    ) -> Result<Self>
    {
        Ok(Self {
            transfer_command_pool: Arc::new(
                TransientCommandPool
                    ::new(device.clone(), queue_family_indices.transfer_index())
                    .context("failed to create transient transfer command pool")?
            ),
            graphics_command_pool: Arc::new(
                TransientCommandPool    
                    ::new(device.clone(), queue_family_indices.graphics_index())
                    .context("failed to create transient graphics command pool")?
            ),
            transfer_commands: Default::default(),
        })
    }
}

pub struct RenderCommands<'a, 'b>{
    command_buffer: vk::CommandBuffer,
    storage: RenderCommandsStorage,
    pub(crate) frame_graph: &'a mut FrameGraph<'b>,
    current_pipeline: Option<GraphicsPipelineId>,
    current_sample_count: MSAA,
    tmp_alloc: &'a ArenaAlloc,
    frame_semaphore: vk::Semaphore,
    frame_semaphore_value: u64,
    buffered_frames: u32,
}

impl<'a, 'b> RenderCommands<'a, 'b> {

    #[inline(always)]
    pub(crate) fn new(
        command_buffer: vk::CommandBuffer,
        frame_graph: &'a mut FrameGraph<'b>,
        frame_semaphore: vk::Semaphore,
        frame_semaphore_value: u64,
        tmp_alloc: &'a ArenaAlloc,
        queue_family_indices: QueueFamilyIndices,
        buffered_frames: u32,
    ) -> Result<Self>
    {
        Ok(Self {
            command_buffer,
            storage: RenderCommandsStorage::new(frame_graph.gpu().device(), queue_family_indices)?,
            frame_graph,
            current_pipeline: None,
            current_sample_count: MSAA::X1,
            frame_semaphore,
            frame_semaphore_value,
            tmp_alloc,
            buffered_frames,
        })
    }

    #[inline(always)]
    pub fn gpu(&self) -> &GpuContext<'b> {
        self.frame_graph.gpu()
    }

    #[inline(always)]
    pub fn gpu_mut(&mut self) -> &mut GpuContext<'b> {
        self.frame_graph.gpu_mut()
    }

    #[inline(always)]
    pub fn frame_graph(&self) -> &FrameGraph<'b> {
        &self.frame_graph
    }

    fn device(&self) -> &ash::Device {
        self.storage.transfer_command_pool.device()
    }

    #[inline(always)]
    pub unsafe fn reset_linear_device_alloc(&mut self, id: LinearDeviceAllocId) -> Result<()> {
        unsafe {
            self.gpu_mut()
                .lock_linear_device_alloc(id, &[])?
                .reset();
        }
        Ok(())
    }

    #[inline(always)]
    pub fn synced_transfer_commands(
        &mut self,
        alloc: LinearDeviceAllocId,
        mut f: impl FnMut(&mut TransferCommands) -> Result<()>
    ) -> Result<()>
    {
        let alloc = self.gpu_mut()
            .lock_linear_device_alloc(alloc, &[])?;
        let mut alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: self.storage.transfer_command_pool.handle(),
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            ..Default::default()
        };
        let mut transfer_command_buffer = Default::default();
        helpers::allocate_command_buffers(
            self.device(),
            &alloc_info,
            core::slice::from_mut(&mut transfer_command_buffer),
        ).context("failed to allocate transfer command buffer")?;
        helpers
            ::begin_command_buffer(self.device(), transfer_command_buffer)
            .context("failed to begin transfer command buffer")?;
        alloc_info.command_pool = self.storage.graphics_command_pool.handle();
        let mut graphics_command_buffer = Default::default();
        helpers::allocate_command_buffers(
            self.device(),
            &alloc_info,
            core::slice::from_mut(&mut graphics_command_buffer),
        ).context("failed to allocate graphics command buffer")?;
        helpers
            ::begin_command_buffer(self.device(), graphics_command_buffer)
            .context("failed to begin graphics command buffer")?;
        let storage = self.storage.transfer_commands.push(TransferCommandsStorage::new(
            self.storage.transfer_command_pool.clone(),
            transfer_command_buffer,
            self.storage.graphics_command_pool.clone(),
            graphics_command_buffer,
            alloc,
            &[],
            Default::default()
        )?);
        f(&mut TransferCommands::new(storage, self.frame_graph.gpu_mut()))
    }

    #[inline(always)]
    pub fn buffered_frames(&self) -> u32 {
        self.buffered_frames
    }

    #[inline(always)]
    pub fn frame_buffer_size(&self) -> Dimensions {
        self.gpu().frame_buffer_size
    }

    /// Waits for previous frame until `timeout` where `timeout` is in nanoseconds.
    /// Returns Ok(true) on success, Ok(false) on timeout and Err(err) if there's another error.
    #[inline(always)]
    pub fn wait_for_previous_frame(&self, timeout: u64) -> Result<bool> {
        let semaphore = self.frame_semaphore;
        let value = self.frame_semaphore_value;
        let wait_info = vk::SemaphoreWaitInfo {
            s_type: vk::StructureType::SEMAPHORE_WAIT_INFO,
            semaphore_count: 1,
            p_semaphores: &semaphore,
            p_values: &value,
            ..Default::default()
        };
        if let Err(err) = unsafe { self.device().wait_semaphores(&wait_info, timeout) } {
            if err == vk::Result::TIMEOUT {
                return Ok(false)
            } else {
                return Err(Error::new("unexpected vulkan error", err))
            }
        }
        Ok(true)
    }

    #[inline(always)]
    pub fn wait_idle(&self) -> Result<()> {
        unsafe {
            self.device()
                .device_wait_idle()
                .context("unexpected vulkan error")?;
        }
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn set_current_sample_count(&mut self, samples: MSAA) {
        self.current_sample_count = samples;
        self.current_pipeline = None;
    }

    #[inline(always)]
    pub fn bind_pipeline(&mut self, id: GraphicsPipelineId) -> Result<()> {
        if self.current_pipeline.unwrap_or_default() == id {
            return Ok(());
        }
        let pipeline = self.gpu().get_graphics_pipeline(id)?;
        if pipeline.samples != self.current_sample_count {
            return Err(Error::just_context(format_compact!(
                "pipeline sample count {} must match pass sample count {}",
                self.current_sample_count, pipeline.samples,
            )))
        }
        unsafe {
            self.device().cmd_bind_pipeline(
                self.command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.handle
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
        let guard = ArenaGuard::new(&self.tmp_alloc);
        let Some(pipeline) = self.current_pipeline else {
            return Err(Error::just_context("attempting to bind shader resources with no pipeline binded"))
        };
        let pipeline = self.gpu()
            .get_graphics_pipeline(pipeline)
            .context("failed to get graphics pipeline")?;
        let (layout, sets) = self.gpu().pipeline_get_shader_resource(
            pipeline.layout_id,
            &guard,
            f,
        )?;
        if sets.is_empty() {
            return Ok(())
        }
        unsafe {
            self.device().cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                layout,
                0, &sets, &[]
            );
        }
        Ok(())
    }

    #[inline(always)]
    pub fn push_constants<F>(
        &self,
        f: F,
    ) -> Result<()>
        where
            F: FnMut(PushConstant) -> &'a [u8],
    {
        let guard = ArenaGuard::new(&self.tmp_alloc);
        let Some(pipeline) = self.current_pipeline else {
            return Err(Error::just_context("attempting to push constants with no pipeline binded"))
        };
        let pipeline = self.gpu().get_graphics_pipeline(pipeline)?;
        let (layout, push_constants) = self.gpu().pipeline_get_push_constants(
            pipeline.layout_id,
            &guard,
            f
        )?;
        for (pc, bytes) in &push_constants {
            unsafe {
                self.device().cmd_push_constants(
                    self.command_buffer,
                    layout,
                    pc.stage.into(),
                    pc.offset,
                    bytes,
                );
            }
        }
        Ok(())
    }

    #[inline(always)]
    pub fn draw_indexed<const VERTEX_BUFFER_COUNT: usize>(
        &self,
        info: DrawInfo,
        bindings: [DrawBufferInfo; VERTEX_BUFFER_COUNT],
        index_buffer: DrawBufferInfo,
    ) -> Result<()>
    {
        if self.current_pipeline.is_none() {
            return Err(Error::just_context("attempting to draw with no pipeline binded"))
        };
        unsafe {
            let command_buffer = self.command_buffer;
            let index_buf = self.gpu().get_buffer(index_buffer.id)?;
            let index_buf_properties = index_buf.properties();
            if index_buf_properties.size <
                index_buffer.offset + info.index_count as u64 * info.index_type.index_size()
            {
                return Err(Error::new(
                    "given buffer size and offset are out of range of index buffer",
                    BufferError::OutOfRange {
                        buffer_size: index_buf.properties().size,
                        requested_offset: index_buffer.offset,
                        requested_size: info.index_count as u64 * info.index_type.index_size(),
                    },
                ))
            }
            if let Some(err) = index_buf.validate_usage(vk::BufferUsageFlags::INDEX_BUFFER) {
                return Err(Error::new("index buffer has incompatible usage", err))
            }
            let mut vert = ArrayVec::<vk::Buffer, VERTEX_BUFFER_COUNT>::new();
            let mut vert_off = ArrayVec::<vk::DeviceSize, VERTEX_BUFFER_COUNT>::new();
            for (id, offset) in bindings.iter().map(|v| (v.id, v.offset)) {
                let buf = self.gpu().get_buffer(id)?;
                let properties = buf.properties();
                let size = properties.size;
                if size <= offset {
                    return Err(Error::new(
                        "given buffer size  and offset are out of range of vertex buffer",
                        BufferError::OutOfRange {
                            buffer_size: size,
                            requested_offset: offset, requested_size: 1,
                        }
                    ))
                }
                if let Some(err) = buf.validate_usage(vk::BufferUsageFlags::VERTEX_BUFFER) {
                    return Err(Error::new(format_compact!("vertex buffer has incompatible usage"), err))
                }
                vert.push(buf.handle()).unwrap();
                vert_off.push(offset).unwrap();
            }
            let device = self.device();
            device.cmd_bind_vertex_buffers(command_buffer, info.first_binding, &vert, &vert_off);
            device.cmd_bind_index_buffer(
                command_buffer, index_buf.handle(),
                index_buffer.offset, info.index_type.into()
            );
            device.cmd_draw_indexed(
                command_buffer,
                info.index_count, info.instance_count,
                info.first_index, info.vertex_offset,
                info.first_instance
            );
        }
        Ok(())
    }

    #[inline(always)]
    pub fn draw<const VERTEX_BUFFER_COUNT: usize>(
        &self,
        first_vertex: u32,
        vertex_count: u32,
        first_instance: u32,
        instance_count: u32,
        first_binding: u32,
        bindings: ArrayVec<DrawBufferInfo, VERTEX_BUFFER_COUNT>,
    ) -> Result<()>
    {
        if self.current_pipeline.is_none() {
            return Err(Error::just_context("attempting to draw with no pipeline binded"))
        };
        unsafe {
            let command_buffer = self.command_buffer;
            let mut vert = ArrayVec::<vk::Buffer, VERTEX_BUFFER_COUNT>::new();
            let mut vert_off = ArrayVec::<vk::DeviceSize, VERTEX_BUFFER_COUNT>::new();
            for (id, offset) in bindings.iter().map(|v| (v.id, v.offset)) {
                let buf = self.gpu().get_buffer(id)?;
                let properties = buf.properties();
                let size = properties.size;
                if size <= offset {
                    return Err(Error::new(
                        format_compact!("given buffer size and offset are out of range of vertex buffer"),
                        BufferError::OutOfRange {
                            buffer_size: size,
                            requested_offset: offset, requested_size: 1,
                        }
                    ))
                }
                if let Some(err) = buf.validate_usage(vk::BufferUsageFlags::VERTEX_BUFFER) {
                    return Err(Error::new(format_compact!("vertex buffer has incompatible usage"), err))
                }
                vert.push(buf.handle()).unwrap();
                vert_off.push(offset).unwrap();
            }
            let device = self.device();
            device.cmd_bind_vertex_buffers(command_buffer, first_binding, &vert, &vert_off);
            device.cmd_draw(
                command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance
            );
        }
        Ok(())
    }

    #[inline(always)]
    pub fn draw_bufferless(&self, vertex_count: u32, instance_count: u32) -> Result<()> {
        if self.current_pipeline.is_none() {
            return Err(Error::just_context("attempting to draw with no pipeline binded"))
        };
        unsafe {
            self.device().cmd_draw(self.command_buffer, vertex_count, instance_count, 0, 0);
        }
        Ok(())
    }

    pub(crate) fn finish(self) -> RenderCommandsStorage {
        self.storage
    }
}
