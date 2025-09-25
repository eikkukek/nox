use std::sync::{RwLock, Arc};

use ash::vk;

use nox_mem::vec_types::{Vector, ArrayVec};

use nox_alloc::arena_alloc::*;

use crate::{renderer::*, has_not_bits};

#[derive(Clone, Copy)]
pub struct DrawInfo {
    pub first_index: u32,
    pub index_count: u32,
    pub index_type: IndexType,
    pub vertex_offset: i32,
    pub first_instance: u32,
    pub instance_count: u32,
    pub first_binding: u32,
}

#[derive(Clone, Copy)]
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

pub struct RenderCommands<'a>{
    device: Arc<ash::Device>,
    command_buffer: vk::CommandBuffer,
    global_resources: Arc<RwLock<GlobalResources>>,
    current_pipeline: Option<GraphicsPipelineId>,
    current_sample_count: MSAA,
    tmp_alloc: &'a ArenaAlloc,
    semaphore: vk::Semaphore,
    semaphore_value: u64,
    buffered_frames: u32,
}

impl<'a> RenderCommands<'a> {

    #[inline(always)]
    pub(crate) fn new(
        device: Arc<ash::Device>,
        command_buffer: vk::CommandBuffer,
        global_resources: Arc<RwLock<GlobalResources>>,
        semaphore: vk::Semaphore,
        semaphore_value: u64,
        tmp_alloc: &'a ArenaAlloc,
        buffered_frames: u32,
    ) -> Self
    {
        Self {
            device,
            command_buffer,
            global_resources,
            current_pipeline: None,
            current_sample_count: MSAA::X1,
            semaphore,
            semaphore_value,
            tmp_alloc,
            buffered_frames,
        }
    }

    #[inline(always)]
    pub fn edit_resources(
        &mut self,
        mut f: impl FnMut(&mut GlobalResources) -> Result<(), Error>
    ) -> Result<(), Error> {
        f(&mut *self.global_resources.write().unwrap())
    }

    #[inline(always)]
    pub fn buffered_frames(&self) -> u32 {
        self.buffered_frames
    }

    #[inline(always)]
    pub fn wait_for_previous_frame(&self) -> Result<(), Error> {
        let semaphore = self.semaphore;
        let value = self.semaphore_value;
        let wait_info = vk::SemaphoreWaitInfo {
            s_type: vk::StructureType::SEMAPHORE_WAIT_INFO,
            semaphore_count: 1,
            p_semaphores: &semaphore,
            p_values: &value,
            ..Default::default()
        };
        unsafe {
            self.device.wait_semaphores(&wait_info, u64::MAX)?;
        }
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn set_current_sample_count(&mut self, samples: MSAA) {
        self.current_sample_count = samples;
        self.current_pipeline = None;
    }

    #[inline(always)]
    pub fn bind_pipeline(&mut self, id: GraphicsPipelineId) -> Result<(), Error> {
        let g = self.global_resources.read().unwrap();
        let pipeline = g.get_graphics_pipeline(id)?;
        assert!(pipeline.samples == self.current_sample_count,
            "pipeline sample count must match pass sample count, pass sample count {:?}, pipeline sample count {:?}",
            self.current_sample_count, pipeline.samples,
        );
        unsafe {
            self.device.cmd_bind_pipeline(self.command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.handle);
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
        let guard = ArenaGuard::new(&*self.tmp_alloc);
        let g = self.global_resources.read().unwrap();
        let pipeline = g.get_graphics_pipeline(
            self.current_pipeline.expect("attempting to bind shader resources with no pipeline attached")
        )?;
        let (layout, sets) = g.pipeline_get_shader_resource(
            pipeline.layout_id,
            f,
            &guard,
        )?;
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                layout,
                0, &sets, &[]
            );
        }
        Ok(())
    }

    #[inline(always)]
    pub fn push_constants(
        &self,
        bytes: &[u8],
    ) -> Result<(), Error>
    {
        let g = self.global_resources.read().unwrap();
        let pipeline = g.get_graphics_pipeline(
            self.current_pipeline.expect("attempting to push constants with no pipeline attached")
        )?;
        let (layout, stages) = g.pipeline_get_push_constant_stages(pipeline.layout_id)?;
        unsafe {
            self.device.cmd_push_constants(
                self.command_buffer,
                layout,
                stages,
                0,
                bytes,
            );
        }
        Ok(())
    }

    #[inline(always)]
    pub fn draw_indexed<const VERTEX_BUFFER_COUNT: usize>(
        &self,
        info: DrawInfo,
        bindings: [DrawBufferInfo; VERTEX_BUFFER_COUNT],
        index_buffer: DrawBufferInfo,
    ) -> Result<(), Error>
    {
        assert!(self.current_pipeline.is_some(), "attempting to draw with no pipeline attached");
        unsafe {
            let command_buffer = self.command_buffer;
            let resources = self.global_resources.read().unwrap();
            let index_buf = resources.get_buffer(index_buffer.id)?;
            let index_buf_properties = index_buf.properties();
            if index_buf_properties.size < index_buffer.offset + info.index_count as u64 * info.index_type.index_size() {
                return Err(BufferError::OutOfRange {
                    buffer_size: index_buf.properties().size,
                    requested_offset: index_buffer.offset,
                    requested_size: info.index_count as u64 * info.index_type.index_size(),
                }.into())
            }
            if has_not_bits!(index_buf_properties.usage, vk::BufferUsageFlags::INDEX_BUFFER) {
                return Err(BufferError::UsageMismatch {
                    missing_usage: vk::BufferUsageFlags::INDEX_BUFFER
                }.into())
            }
            let mut vert = ArrayVec::<vk::Buffer, VERTEX_BUFFER_COUNT>::new();
            let mut vert_off = ArrayVec::<vk::DeviceSize, VERTEX_BUFFER_COUNT>::new();
            for (id, offset) in bindings.iter().map(|v| (v.id, v.offset)) {
                let buf = resources.get_buffer(id)?;
                let properties = buf.properties();
                let size = properties.size;
                if size <= offset {
                    return Err(BufferError::OutOfRange {
                        buffer_size: size,
                        requested_offset: offset, requested_size: 1,
                    }.into())
                }
                if has_not_bits!(properties.usage, vk::BufferUsageFlags::VERTEX_BUFFER) {
                    return Err(BufferError::UsageMismatch {
                        missing_usage: vk::BufferUsageFlags::VERTEX_BUFFER
                    }.into())
                }
                vert.push(buf.handle()).unwrap();
                vert_off.push(offset).unwrap();
            }
            self.device.cmd_bind_vertex_buffers(command_buffer, info.first_binding, &vert, &vert_off);
            self.device.cmd_bind_index_buffer(command_buffer, index_buf.handle(), index_buffer.offset, info.index_type.into());
            self.device.cmd_draw_indexed(
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
    ) -> Result<(), Error>
    {
        assert!(self.current_pipeline.is_some(), "attempting to draw with no pipeline attached");
        unsafe {
            let command_buffer = self.command_buffer;
            let resources = self.global_resources.read().unwrap();
            let mut vert = ArrayVec::<vk::Buffer, VERTEX_BUFFER_COUNT>::new();
            let mut vert_off = ArrayVec::<vk::DeviceSize, VERTEX_BUFFER_COUNT>::new();
            for (id, offset) in bindings.iter().map(|v| (v.id, v.offset)) {
                let buf = resources.get_buffer(id)?;
                let properties = buf.properties();
                let size = properties.size;
                if size <= offset {
                    return Err(BufferError::OutOfRange {
                        buffer_size: size,
                        requested_offset: offset, requested_size: 1,
                    }.into())
                }
                if has_not_bits!(properties.usage, vk::BufferUsageFlags::VERTEX_BUFFER) {
                    return Err(BufferError::UsageMismatch {
                        missing_usage: vk::BufferUsageFlags::VERTEX_BUFFER
                    }.into())
                }
                vert.push(buf.handle()).unwrap();
                vert_off.push(offset).unwrap();
            }
            self.device.cmd_bind_vertex_buffers(command_buffer, first_binding, &vert, &vert_off);
            self.device.cmd_draw(
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
    pub fn draw_bufferless(&self, vertex_count: u32, instance_count: u32) {
        assert!(self.current_pipeline.is_some(), "attempting to draw with no pipeline attached");
        unsafe {
            self.device.cmd_draw(self.command_buffer, vertex_count, instance_count, 0, 0);
        }
    }
}
