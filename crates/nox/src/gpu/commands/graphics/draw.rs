use core::ops::{Deref, DerefMut};

use nox_ash::vk;

use compact_str::format_compact;

use nox_mem::{
    vec::{NonNullVec32, Vec32, ArrayVec, Vector},
    alloc::LocalAlloc,
    borrow::SizedCowMut,
};

use nox_alloc::arena::*;

use crate::{
    gpu::{
        prelude::*,
        commands::prelude::ShaderResourceBindCall,
    },
    dev::error::*,
};

#[derive(Clone, Copy, Debug)]
pub struct IndexedDrawInfo {
    first_index: u32,
    index_count: u32,
    index_type: IndexType,
    first_instance: u32,
    instance_count: u32,
    vertex_offset: i32,
}

impl IndexedDrawInfo {
    
    #[inline(always)]
    pub fn new(
        first_index: u32,
        index_count: u32,
        index_type: IndexType,
    ) -> Self {
        Self {
            first_index,
            index_count,
            index_type,
            first_instance: 0,
            instance_count: 1,
            vertex_offset: 0,
        }
    }

    #[inline(always)]
    pub fn with_instances(mut self, first_instance: u32, instance_count: u32) -> Self {
        self.first_instance = first_instance;
        self.instance_count = instance_count;
        self
    }

    #[inline(always)]
    pub fn with_vertex_offset(mut self, offset: i32) -> Self {
        self.vertex_offset = offset;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DrawBufferRange {
    pub id: BufferId,
    pub offset: u64,
    pub size: u64
}

impl DrawBufferRange {

    #[inline(always)]
    pub fn new(id: BufferId, offset: u64, size: u64) -> Self {
        Self {
            id,
            offset,
            size,
        }
    }
}

pub type VertexBufferInfo = DrawBufferRange;

#[derive(Clone, Copy, Debug)]
pub struct IndexBufferInfo {
    pub id: BufferId,
    pub offset: u64,
}

impl IndexBufferInfo {

    #[inline(always)]
    pub fn new(id: BufferId, offset: u64) -> Self {
        Self {
            id,
            offset,
        }
    }
}

pub(super) struct DrawCall {
    pub index_buffer: Option<DrawBufferRange>,
    pub vertex_buffers: NonNullVec32<'static, DrawBufferRange>,
}

#[derive(Default)]
pub(crate) struct DrawCommandCache {
    pub(super) pipelines: Vec32<PipelineHandle>,
    pub(super) shader_resource_binds: Vec32<ShaderResourceBindCall>,
    pub(super) draw_calls: Vec32<DrawCall>,
}

unsafe impl Send for DrawCommandCache {}
unsafe impl Sync for DrawCommandCache {}

impl DrawCommandCache {

    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.pipelines.clear();
        self.shader_resource_binds.clear();
        self.draw_calls.clear();
    }
}

pub(crate) enum DrawCommandAlloc<'a> {
    Borrowed(&'a (dyn LocalAlloc<Error = Error> + 'a)),
    Owned(Box<dyn LocalAlloc<Error = Error> + 'a>),
}

impl<'a> Deref for DrawCommandAlloc<'a> {

    type Target = dyn LocalAlloc<Error = Error> + 'a;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(o) => o.deref(),
        }
    }
}

pub(crate) struct DrawCommandStorage<'a> {
    pub(super) command_buffer: vk::CommandBuffer,
    color_formats: NonNullVec32<'a, vk::Format>,
    depth_format: vk::Format,
    stencil_format: vk::Format,
    sample_count: MSAA,
    pub(super) cache: SizedCowMut<'a, DrawCommandCache>,
    pub(super) wait_scope: vk::PipelineStageFlags2,
    alloc: DrawCommandAlloc<'a>,
}

impl<'a> DrawCommandStorage<'a> {

    #[inline(always)]
    pub unsafe fn new(
        command_buffer: vk::CommandBuffer,
        color_formats: &[vk::Format],
        depth_format: vk::Format,
        stencil_format: vk::Format,
        sample_count: MSAA,
        cache: SizedCowMut<'a, DrawCommandCache>,
        alloc: DrawCommandAlloc<'a>,
    ) -> Result<Self> {
        let mut color_f = NonNullVec32::with_capacity(
            color_formats.len() as u32,
            &*alloc,
        )?.into_static();
        color_f.append(&color_formats);
        Ok(Self {
            command_buffer,
            color_formats: color_f,
            depth_format,
            stencil_format,
            sample_count,
            cache,
            wait_scope: vk::PipelineStageFlags2::NONE,
            alloc,
        })
    }
}

impl<'a> Drop for DrawCommandStorage<'a> {

    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            let alloc = &*self.alloc;
            self.color_formats.drop_and_free(alloc);
            let cache = self.cache.deref_mut();
            for call in &mut cache.shader_resource_binds {
                call.sets.drop_and_free(alloc);
                call.barriers.drop_and_free(alloc);
            }
            for call in &mut cache.draw_calls {
                call.vertex_buffers.drop_and_free(alloc);
            }
            cache.clear();
        }
    }
}

pub struct DrawCommands<'a, 'b> {
    pub(super) resources: &'a Resources,
    pub(super) storage: &'a mut DrawCommandStorage<'b>,
    pub(super) buffers: DynResourceReadGuard<'a, BufferMeta, BufferId>,
    pub(super) images: DynResourceReadGuard<'a, ImageMeta, ImageId>,
}

impl<'a, 'b> DrawCommands<'a, 'b> {

    pub fn bind_pipeline(&mut self, id: GraphicsPipelineId) -> Result<()> {
        let pipeline = self.resources.get_graphics_pipeline(id)?;
        if pipeline.samples() != self.storage.sample_count {
            return Err(Error::just_context(format_compact!(
                "pipeline sample count {} must match pass sample count {}",
                self.storage.sample_count, pipeline.samples(),
            )))
        }
        if *pipeline.color_output_formats() != **self.storage.color_formats {
            return Err(Error::just_context(
                "pipeline color output formats don't match with pass"
            ))
        }
        if pipeline.depth_output_format() != self.storage.depth_format {
            return Err(Error::just_context(format_compact!(
                "pipeline depth output format {:?} doesn't match with pass depth format {:?}",
                pipeline.depth_output_format(), self.storage.depth_format,
            )))
        }
        if pipeline.stencil_output_format() != self.storage.stencil_format {
            return Err(Error::just_context(format_compact!(
                "pipeline stencil output format {:?} doesn't match with pass stencil format {:?}",
                pipeline.stencil_output_format(), self.storage.stencil_format,
            )))
        }
        let handle = pipeline.handle().clone();
        unsafe {
            self.resources.vk().device().cmd_bind_pipeline(
                self.storage.command_buffer, vk::PipelineBindPoint::GRAPHICS, handle.handle()
            );
        }
        self.storage.cache.pipelines.push(handle);
        Ok(())
    }

    pub fn bind_shader_resources<F>(
        &mut self,
        barrier_info: &[BindingBarrierInfo],
        mut f: F,
    ) -> Result<()>
        where
            F: FnMut(u32) -> Option<ShaderResourceId>,
    {
        let cache = self.storage.cache.deref_mut();
        let Some(pipeline) = cache.pipelines.last() else {
            return Err(Error::just_context("attempting to bind shader resources with no pipeline bound"))
        };
        let tmp_alloc = self.resources.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let mut barriers = NonNullVec32
            ::with_capacity(barrier_info.len() as u32, &*self.storage.alloc)?
            .into_static();
        barriers.append(barrier_info);
        let mut bind_call = ShaderResourceBindCall {
            sets: NonNullVec32::with_capacity(
                pipeline.shader_set().descriptor_set_layouts().len() as u32,
                &*self.storage.alloc,
            )?.into_static(),
            barriers,
        };
        let sets = self.resources.get_shader_set_resources(
            pipeline.shader_set(),
            &tmp_alloc,
            |index| {
                let id = f(index);
                bind_call.sets.push(id);
                id
            },
        )?;
        if sets.is_empty() {
            return Ok(())
        }
        cache.shader_resource_binds.push(bind_call);
        unsafe {
            self.resources.vk().device().cmd_bind_descriptor_sets(
                self.storage.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.shader_set().pipeline_layout(),
                0, &sets, &[]
            );
        }
        Ok(())
    }

    pub fn push_constants<'c, F>(
        &self,
        f: F,
    ) -> Result<()>
        where
            F: FnMut(PushConstantRange) -> &'c [u8],
    {
        let Some(pipeline) = self.storage.cache.pipelines.last() else {
            return Err(Error::just_context("attempting to push constants with no pipeline bound"))
        };
        let tmp_alloc = self.resources.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let push_constants = self.resources.get_shader_set_push_constant_ranges(
            pipeline.shader_set(),
            &tmp_alloc,
            f
        )?;
        for (pc, bytes) in &push_constants {
            unsafe {
                self.resources.vk().device().cmd_push_constants(
                    self.storage.command_buffer,
                    pipeline.shader_set().pipeline_layout(),
                    pc.stage.into(),
                    pc.offset,
                    bytes,
                );
            }
        }
        Ok(())
    }

    pub fn draw_indexed<const VERTEX_BINDING_COUNT: usize>(
        &mut self,
        info: IndexedDrawInfo,
        index_buffer: IndexBufferInfo,
        vertex_bindings: [VertexBufferInfo; VERTEX_BINDING_COUNT],
    ) -> Result<()>
    {
        let cache = self.storage.cache.deref_mut();
        if cache.pipelines.is_empty() {
            return Err(Error::just_context("attempting to draw with no pipeline bound"))
        }
        self.storage.wait_scope = vk::PipelineStageFlags2::VERTEX_INPUT;
        let index_buf_size = info.index_count as vk::DeviceSize * info.index_type.index_size();
        let mut call = DrawCall {
            index_buffer: Some(DrawBufferRange::new(
                index_buffer.id,
                index_buffer.offset,
                index_buf_size,
            )),
            vertex_buffers: NonNullVec32::with_capacity(
                VERTEX_BINDING_COUNT as u32,
                &*self.storage.alloc,
            )?,
        };
        unsafe {
            let command_buffer = self.storage.command_buffer;
            let index_buf = self.buffers.get(index_buffer.id)?;
            if index_buffer.offset + index_buf_size > index_buf.properties().size
            {
                return Err(Error::new(
                    BufferError::OutOfRange {
                        buffer_size: index_buf.properties().size,
                        requested_offset: index_buffer.offset,
                        requested_size: index_buf_size,
                    },
                    "given buffer size and offset are out of range of index buffer",
                ))
            }
            if let Some(err) = index_buf.validate_usage(vk::BufferUsageFlags::INDEX_BUFFER) {
                return Err(Error::new(err, "index buffer has incompatible usage"))
            }
            let mut vert_bufs = ArrayVec::<vk::Buffer, VERTEX_BINDING_COUNT>::new();
            let mut vert_offsets = ArrayVec::<vk::DeviceSize, VERTEX_BINDING_COUNT>::new();
            let mut vert_sizes = ArrayVec::<vk::DeviceSize, VERTEX_BINDING_COUNT>::new();
            for buf_info in vertex_bindings.iter().copied() {
                let buf = self.buffers.get(buf_info.id)?;
                if buf_info.offset + buf_info.size > buf.properties().size {
                    return Err(Error::new(
                        BufferError::OutOfRange {
                            buffer_size: buf.properties().size,
                            requested_offset: buf_info.offset, requested_size: buf_info.size,
                        },
                        "given buffer size and offset are out of range of vertex buffer",
                    ))
                }
                if let Some(err) = buf.validate_usage(vk::BufferUsageFlags::VERTEX_BUFFER) {
                    return Err(Error::new(err, format_compact!("vertex buffer has incompatible usage")))
                }
                vert_bufs.push(buf.handle());
                vert_offsets.push(buf_info.offset);
                vert_sizes.push(buf_info.size);
                call.vertex_buffers.push(buf_info);
            }
            let vk = self.resources.vk();
            vk.extended_dynamic_state_device()
                .cmd_bind_vertex_buffers2(
                    command_buffer,
                    0,
                    &vert_bufs, &vert_offsets, Some(&vert_sizes),
                    None,
                );
            vk.maintenance5_device().cmd_bind_index_buffer2(
                command_buffer, index_buf.handle(),
                index_buffer.offset, index_buf_size, info.index_type.into()
            );
            vk.device().cmd_draw_indexed(
                command_buffer,
                info.index_count, info.instance_count,
                info.first_index, info.vertex_offset,
                info.first_instance,
            );
        }
        cache.draw_calls.push(call);
        Ok(())
    }

    pub fn draw<const VERTEX_BINDING_COUNT: usize>(
        &mut self,
        first_vertex: u32,
        vertex_count: u32,
        first_instance: u32,
        instance_count: u32,
        vertex_bindings: [VertexBufferInfo; VERTEX_BINDING_COUNT],
    ) -> Result<()>
    {
        let cache = self.storage.cache.deref_mut();
        if cache.pipelines.is_empty() {
            return Err(Error::just_context("attempting to draw with no pipeline bound"))
        };
        self.storage.wait_scope = vk::PipelineStageFlags2::VERTEX_INPUT;
        let mut call = DrawCall {
            index_buffer: None,
            vertex_buffers: NonNullVec32::with_capacity(
                VERTEX_BINDING_COUNT as u32,
                &*self.storage.alloc,
            )?,
        };
        unsafe {
            let mut vert = ArrayVec::<vk::Buffer, VERTEX_BINDING_COUNT>::new();
            let mut vert_offsets = ArrayVec::<vk::DeviceSize, VERTEX_BINDING_COUNT>::new();
            let mut vert_sizes = ArrayVec::<vk::DeviceSize, VERTEX_BINDING_COUNT>::new();
            for buf_info in vertex_bindings.iter().copied() {
                let buf = self.buffers.get(buf_info.id)?;
                if buf_info.offset + buf_info.size > buf.properties().size {
                    return Err(Error::new(
                        BufferError::OutOfRange {
                            buffer_size: buf.properties().size,
                            requested_offset: buf_info.size, requested_size: buf_info.size,
                        },
                        format_compact!("given buffer size and offset are out of range of vertex buffer"),
                    ))
                }
                if let Some(err) = buf.validate_usage(vk::BufferUsageFlags::VERTEX_BUFFER) {
                    return Err(Error::new(err, format_compact!("vertex buffer has incompatible usage")))
                }
                vert.push(buf.handle());
                vert_offsets.push(buf_info.offset);
                vert_sizes.push(buf_info.size);
                call.vertex_buffers.push(buf_info);
            }
            let command_buffer = self.storage.command_buffer;
            let vk = self.resources.vk();
            vk.extended_dynamic_state_device().cmd_bind_vertex_buffers2(
                command_buffer,
                0,
                &vert, &vert_offsets,
                Some(&vert_sizes), None
            );
            vk.device().cmd_draw(
                command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance
            );
        }
        cache.draw_calls.push(call);
        Ok(())
    }

    #[inline(always)]
    pub fn draw_bufferless(&self, vertex_count: u32, instance_count: u32) -> Result<()> {
        if self.storage.cache.pipelines.is_empty() {
            return Err(Error::just_context("attempting to draw with no pipeline bound"))
        };
        unsafe {
            self.resources.vk().device().cmd_draw(self.storage.command_buffer, vertex_count, instance_count, 0, 0);
        }
        Ok(())
    }
}
