use core::{
    hash::Hash,
    ptr::NonNull,
};

use rustc_hash::FxHashMap;

use nox::{
    mem::{
        align_of, align_up, size_of,
        vec_types::{ArrayVec, GlobalVec, Vector},
        Allocator,
    },
    *
};

use nox_font::{VertexTextRenderer, Face};

use nox_geom::{
    Vec2,
    vec2,
};

use crate::{ColorRGBA, Window, WindowContext};

#[repr(C)]
#[derive(VertexInput)]
pub(crate) struct Vertex {
    pos: Vec2,
}

impl From<[f32; 2]> for Vertex {

    fn from(value: [f32; 2]) -> Self {
        Self {
            pos: value.into(),
        }
    }
}

#[repr(C)]
#[derive(VertexInput)]
pub(crate) struct VertexUv {
    pos: [f32; 2],
    uv: [f32; 2],
}

#[repr(C)]
pub(crate) struct PushConstants {
    pub vert_off: Vec2,
    pub inv_aspect_ratio: f32,
    _pad: [u8; 4],
    pub color: ColorRGBA,
}

pub(crate) fn push_constants(
    vert_off: Vec2,
    inv_aspect_ratio: f32,
    color: ColorRGBA,
) ->PushConstants 
{
    PushConstants {
        vert_off,
        inv_aspect_ratio,
        _pad: Default::default(),
        color,
    }
}

#[derive(Clone, Copy, Debug)]
struct RingBufReg {
    head: usize,
    tail: usize,
}

fn ring_buf_reg(head: usize, tail: usize) -> RingBufReg {
    RingBufReg { head, tail }
}

pub(crate) struct RingBuf {
    buffer: BufferId,
    map: NonNull<u8>,
    current_reg: RingBufReg,
    frame_regions: ArrayVec<RingBufReg, {MAX_BUFFERED_FRAMES as usize}>,
}

pub(crate) struct RingBufMem<T> {
    pub ptr: NonNull<T>,
    pub offset: u64,
}

impl<T> Default for RingBufMem<T> {

    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            offset: 0,
        }
    }
}

impl RingBuf {

    pub fn allocate<T>(
        &mut self,
        render_commands: &mut RenderCommands,
        count: usize,
        buf_size: usize,
    ) -> Result<RingBufMem<T>, Error>
    {
        let RingBufReg { head, tail } = self.current_reg;
        let size = count * size_of!(T);
        let mut offset = align_up(tail, align_of!(T));
        let mut new_tail = offset + size;
        // wrapped around to current head
        if tail < head && new_tail > head {
            return Err(Error::UserError("GUI ring buffer out of memory".into()))
        }
        // wrap around
        if new_tail > buf_size
        {
            new_tail = 0;
            offset = 0;
        }
        let oldest_region = self.frame_regions.back().unwrap();
        // new tail reaches oldest head
        if tail < oldest_region.tail && tail > oldest_region.head
        {
            render_commands.wait_for_previous_frame()?;
            for reg in &mut self.frame_regions {
                *reg = ring_buf_reg(0, 0);
            }
        }
        self.current_reg = ring_buf_reg(head, new_tail);
        Ok(RingBufMem {
            offset: offset as u64,
            ptr: unsafe { self.map.add(offset).cast() },
        })
    }

    fn finish_frame(&mut self) {
        self.frame_regions.pop();
        self.frame_regions.insert(0, self.current_reg).unwrap();
        self.current_reg = ring_buf_reg(self.current_reg.tail, self.current_reg.tail);
    }
}

const BASE_VERTEX_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;

    layout(push_constant) uniform PushConstant {
        vec2 vert_off;
        float inv_aspect_ratio;
    } pc;

    void main() {
        vec2 pos = in_pos;
        pos.x *= pc.inv_aspect_ratio;
        pos += pc.vert_off;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
";

const BASE_FRAGMENT_SHADER: &'static str = "
    #version 450

    layout(location = 0) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        layout(offset = 16) vec4 color;
    } pc;

    void main() {
        out_color = pc.color;
    }
";

#[derive(Default)]
struct Pipelines {
    base_pipeline_layout: Option<PipelineLayoutId>,
    base_pipeline: Option<GraphicsPipelineId>,
    base_shaders: Option<[ShaderId; 2]>,
}

pub struct Workspace<'a, FontHash>
    where
        FontHash: Clone + PartialEq + Eq + Hash
{
    text_renderer: VertexTextRenderer<'a, FontHash>,
    windows: FxHashMap<u32, Window>,
    active_windows: GlobalVec<u32>,
    vertex_buffer: Option<RingBuf>,
    index_buffer: Option<RingBuf>,
    pipelines: Pipelines,
    ring_buffer_size: usize,
    inv_aspect_ratio: f32,
}

impl<'a, FontHash> Workspace<'a, FontHash>
    where
        FontHash: Clone + PartialEq + Eq + Hash
{

    pub fn new(
        fonts: impl IntoIterator<Item = (FontHash, Face<'a>)>,
        font_curve_tolerance: f32,
    ) -> Self
    {
        Self {
            text_renderer: VertexTextRenderer::new(fonts, font_curve_tolerance),
            windows: Default::default(),
            active_windows: Default::default(),
            vertex_buffer: None,
            index_buffer: None,
            pipelines: Default::default(),
            ring_buffer_size: 1 << 23,
            inv_aspect_ratio: 1.0,
        }
    }

    /// (re)creates required graphics pipelines
    pub fn create_graphics_pipelines(
        &mut self,
        render_context: &mut RendererContext,
        samples: MSAA,
        output_format: ColorFormat,
        cache_id: Option<PipelineCacheId>,
        alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        render_context.edit_resources(|v| {
            if let Some(pipeline) = self.pipelines.base_pipeline.take() {
                v.destroy_graphics_pipeline(pipeline);
            }
            let &mut shaders = self.pipelines.base_shaders
                .get_or_insert([
                    v.create_shader(BASE_VERTEX_SHADER, "nox_gui base vertex shader", ShaderStage::Vertex)?,
                    v.create_shader(BASE_FRAGMENT_SHADER, "nox_gui base fragment shader", ShaderStage::Fragment)?,
            ]);
            let &mut base_layout = self.pipelines.base_pipeline_layout.get_or_insert(
                v.create_pipeline_layout(shaders)?
            );
            let mut base_info = GraphicsPipelineInfo::new(base_layout);
            base_info
                .with_vertex_input_binding(VertexInputBinding::new::<0, Vertex>(0, VertexInputRate::Vertex))
                .with_sample_shading(SampleShadingInfo::new(samples, 0.2, false, false))
                .with_color_output(output_format, WriteMask::all(), None);
            v.create_graphics_pipelines(
                &[base_info],
                cache_id,
                alloc,
                |_, p| { self.pipelines.base_pipeline = Some(p) }
            )?;
            Ok(())
        })
    }

    pub fn update_window<F>(
        &mut self,
        id: u32,
        mut f: F,
        initial_size: [f32; 2],
        initial_position: [f32; 2],
    ) -> Result<(), Error>
        where
            F: FnMut(WindowContext) -> Result<(), Error>
    {
        let window = self.windows.entry(id).or_insert(Window::new(initial_size, initial_position));
        f(WindowContext::new(window, &Default::default()))?;
        self.active_windows.push(id);
        Ok(())
    }

    pub fn end<I: Interface>(
        &mut self,
        nox: &Nox<'_, I>,
    )
    {
        self.inv_aspect_ratio = 1.0 / nox.aspect_ratio() as f32;
        let mut cursor_pos: Vec2 = nox.normalized_cursor_position_f32().into();
        cursor_pos *= 2.0;
        cursor_pos -= vec2(1.0, 1.0);
        let style = Default::default();
        for id in &self.active_windows {
            let window = self.windows.get_mut(id).unwrap();
            window.update(nox, cursor_pos, &style);
            window.triangulate();
        }
    }

    pub fn render_commands(
        &mut self,
        render_commands: &mut RenderCommands,
    ) -> Result<(), Error>
    {
        if self.vertex_buffer.is_none() {
            self.init_buffers(render_commands)?;
        } 
        let Some(base_pipeline) = self.pipelines.base_pipeline else {
            return Err(Error::UserError(
                "nox_gui: attempting to render Workspace before creating graphics pipelines".into()
            ))
        };
        let inv_aspect_ratio = self.inv_aspect_ratio;
        render_commands.bind_pipeline(base_pipeline)?;
        let vertex_buffer = self.vertex_buffer.as_mut().unwrap();
        let index_buffer = self.index_buffer.as_mut().unwrap();
        for id in &self.active_windows {
            self.windows.get(id).unwrap().render_commands(
                render_commands,
                inv_aspect_ratio,
                vertex_buffer.buffer,
                index_buffer.buffer,
                |render_commands, count|
                    vertex_buffer.allocate(render_commands, count, self.ring_buffer_size),
                |render_commands, count|
                    index_buffer.allocate(render_commands, count, self.ring_buffer_size),
            )?;
        }
        vertex_buffer.finish_frame();
        index_buffer.finish_frame();
        self.active_windows.clear();
        Ok(())
    }

    fn init_buffers(&mut self, render_commands: &mut RenderCommands) -> Result<(), Error> {
        let buffered_frames = render_commands.buffered_frames() as usize;
        render_commands.edit_resources(|r| {
            let vertex_buffer = r.create_buffer(
                self.ring_buffer_size as u64,
                &[BufferUsage::VertexBuffer],
                &mut r.default_binder_mappable()
            )?;
            let vertex_buffer_map = unsafe {
                r.map_buffer(vertex_buffer).unwrap()
            };
            self.vertex_buffer = Some(RingBuf {
                buffer: vertex_buffer,
                map: vertex_buffer_map,
                current_reg: ring_buf_reg(0, 0),
                frame_regions: ArrayVec::with_len(
                    ring_buf_reg(0, 0),
                    buffered_frames,
                ).unwrap(),
            });
            let index_buffer = r.create_buffer(
                self.ring_buffer_size as u64,
                &[BufferUsage::IndexBuffer],
                &mut r.default_binder_mappable()
            )?;
            let index_buffer_map = unsafe {
                r.map_buffer(index_buffer).unwrap()
            };
            self.index_buffer = Some(RingBuf {
                buffer: index_buffer,
                map: index_buffer_map, 
                current_reg: ring_buf_reg(0, 0),
                frame_regions: ArrayVec::with_len(
                    ring_buf_reg(0, 0),
                    buffered_frames,
                ).unwrap(),
            });
            Ok(())
        })?;
        Ok(())
    }

    pub fn clean_up(&mut self, context: &RendererContext) {
        context.edit_resources(|r| {
            if let Some(buf) = self.vertex_buffer.take() {
                r.destroy_buffer(buf.buffer);
            };
            if let Some(buf) = self.index_buffer.take() {
                r.destroy_buffer(buf.buffer);
            }
            if let Some(pipeline) = self.pipelines.base_pipeline.take() {
                r.destroy_graphics_pipeline(pipeline);
            }
            Ok(())
        }).ok();
    }
}
