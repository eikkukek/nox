use std::sync::{RwLock, Arc};

use ash::vk;

use nox_mem::vec_types::{Vector, ArrayVec};

use crate::{renderer::*, stack_alloc::StackGuard};

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

pub struct RenderCommands{
    device: Arc<ash::Device>,
    command_buffer: vk::CommandBuffer,
    global_resources: Arc<RwLock<GlobalResources>>,
    current_pipeline: Option<GraphicsPipelineID>,
    tmp_alloc: Rc<StackAlloc>,
}

impl RenderCommands {

    #[inline(always)]
    pub(crate) fn new(
        device: Arc<ash::Device>,
        command_buffer: vk::CommandBuffer,
        global_resources: Arc<RwLock<GlobalResources>>,
        tmp_alloc: Rc<StackAlloc>,
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
    pub fn bind_pipeline(&mut self, id: GraphicsPipelineID) {
        let handle = self.global_resources
            .read()
            .unwrap()
            .get_pipeline_handle(id);
        unsafe {
            self.device.cmd_bind_pipeline(self.command_buffer, vk::PipelineBindPoint::GRAPHICS, handle);
        }
        self.current_pipeline = Some(id);
    }

    #[inline(always)]
    pub fn bind_shader_resources<F>(
        &self,
        f: F,
    ) -> Result<(), Error>
        where
            F: FnMut(u32) -> ShaderResourceID,
    {
        let guard = StackGuard::new(&*self.tmp_alloc);
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
    pub fn push_constants<'a, F>(
        &'a self,
        f: F,
    ) -> Result<(), Error>
        where
            F: FnMut(PushConstant) -> &'a [u8]
    {
        let guard = StackGuard::new(&*self.tmp_alloc);
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
        vertex_buffers: ArrayVec<(BufferID, u64), VERTEX_BUFFER_COUNT>,
        index_buffer: (BufferID, u64),
    )
    {
        assert!(self.current_pipeline.is_some(), "attempting to draw with no pipeline attached");
        unsafe {
            let command_buffer = self.command_buffer;
            let resources = self.global_resources.read().unwrap();
            let index_buf = resources.get_buffer(index_buffer.0);
            assert!(index_buf.properties().size >= index_buffer.1 + info.index_count as u64 * info.index_type.index_size(),
                "index buffer offset + index count goes out of index buffer range");
            let mut vert = ArrayVec::<vk::Buffer, VERTEX_BUFFER_COUNT>::new();
            let mut vert_off = ArrayVec::<vk::DeviceSize, VERTEX_BUFFER_COUNT>::new();
            for (id, offset) in &vertex_buffers {
                let buf = resources.get_buffer(*id);
                assert!(buf.properties().size >= *offset, "vertex buffer offset goes out of vertex buffer range");
                vert.push(buf.handle()).unwrap();
                vert_off.push(*offset).unwrap();
            }
            self.device.cmd_bind_vertex_buffers(command_buffer, info.first_binding, &vert, &vert_off);
            self.device.cmd_bind_index_buffer(command_buffer, index_buf.handle(), index_buffer.1, info.index_type.into());
            self.device.cmd_draw_indexed(command_buffer, info.index_count, info.instance_count, info.first_index, info.vertex_offset, info.first_instance);
        }
    }
}
