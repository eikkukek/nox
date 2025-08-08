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
    pub id: BufferID,
    pub offset: u64,
}

impl DrawBufferInfo {

    pub fn new(id: BufferID, offset: u64) -> Self {
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
    current_pipeline: Option<GraphicsPipelineID>,
    tmp_alloc: &'a ArenaAlloc,
}

impl<'a> RenderCommands<'a> {

    #[inline(always)]
    pub(crate) fn new(
        device: Arc<ash::Device>,
        command_buffer: vk::CommandBuffer,
        global_resources: Arc<RwLock<GlobalResources>>,
        tmp_alloc: &'a ArenaAlloc,
    ) -> Self
    {
        Self {
            device,
            command_buffer,
            global_resources,
            current_pipeline: None,
            tmp_alloc,
        }
    }

    #[inline(always)]
    pub fn bind_pipeline(&mut self, id: GraphicsPipelineID) -> Result<(), Error> {
        let handle = self.global_resources
            .read()
            .unwrap()
            .get_pipeline_handle(id)?;
        unsafe {
            self.device.cmd_bind_pipeline(self.command_buffer, vk::PipelineBindPoint::GRAPHICS, handle);
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
            F: FnMut(u32) -> ShaderResourceID,
    {
        let guard = ArenaGuard::new(&*self.tmp_alloc);
        let g = self.global_resources.read().unwrap();
        let (layout, sets) = g.pipeline_get_shader_resource(
            self.current_pipeline.expect("attempting to bind shader resources with no pipeline attached"),
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
    pub fn push_constants<'b, F>(
        &'b self,
        f: F,
    ) -> Result<(), Error>
        where
            F: FnMut(PushConstant) -> &'b [u8]
    {
        let guard = ArenaGuard::new(&*self.tmp_alloc);
        let g = self.global_resources.read().unwrap();
        let (layout, pcs) = g.pipeline_get_push_constants(
            self.current_pipeline.expect("attempting to push constants with not pipeline attached"),
            f,
            &guard,
        )?;
        unsafe {
            for pc in &pcs {
                self.device.cmd_push_constants(
                    self.command_buffer,
                    layout,
                    pc.0.stage_flags,
                    pc.0.offset,
                    pc.1
                );
            }
        }
        Ok(())
    }

    #[inline(always)]
    pub fn draw_indexed<const VERTEX_BUFFER_COUNT: usize>(
        &self,
        info: DrawInfo,
        vertex_buffers: ArrayVec<DrawBufferInfo, VERTEX_BUFFER_COUNT>,
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
            for (id, offset) in vertex_buffers.iter().map(|v| (v.id, v.offset)) {
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
}
