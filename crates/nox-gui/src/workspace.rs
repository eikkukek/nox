use core::{
    hash::Hash,
    ptr::NonNull,
};

use nox::{
    mem::{
        align_of, align_up, size_of,
        slot_map::{GlobalSlotMap, SlotIndex},
        vec_types::{ArrayVec, GlobalVec, Vector},
        Allocator,
    },
    *
};

use nox_font::{VertexTextRenderer, Face};

use nox_geom::earcut::earcut;

use crate::Widget;

#[repr(C)]
#[derive(VertexInput)]
struct Vertex {
    pos: [f32; 2],
}

#[repr(C)]
#[derive(VertexInput)]
struct VertexUv {
    pos: [f32; 2],
    uv: [f32; 2],
}

#[derive(Clone, Copy, Debug)]
struct RingBufReg {
    head: usize,
    tail: usize,
}

fn ring_buf_reg(head: usize, tail: usize) -> RingBufReg {
    RingBufReg { head, tail }
}

struct RingBuf {
    buffer: BufferId,
    map: NonNull<u8>,
    current_reg: RingBufReg,
    frame_regions: ArrayVec<RingBufReg, {MAX_BUFFERED_FRAMES as usize}>,
}

pub struct RingBufMem<T> {
    ptr: NonNull<T>,
    offset: u64,
}

impl RingBuf {

    fn allocate<T>(
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
    } pc;

    void main() {
        gl_Position = vec4(in_pos + pc.vert_off, 0.0, 1.0);
    }
";

const BASE_FRAGMENT_SHADER: &'static str = "
    #version 450

    layout(location = 0) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        layout(offset = 16) vec4 color;
    } pc;

    void main() {
        out_color = vec4(1.0);
    }
";

#[derive(Default)]
struct Pipelines {
    base_pipeline_layout: Option<PipelineLayoutId>,
    base_pipeline: Option<GraphicsPipelineId>,
    base_shaders: Option<[ShaderId; 2]>,
}

pub struct WidgetId(SlotIndex<Widget>);

pub struct Workspace<'a, FontHash>
    where
        FontHash: Clone + PartialEq + Eq + Hash
{
    text_renderer: VertexTextRenderer<'a, FontHash>,
    widgets: GlobalSlotMap<Widget>,
    vertex_buffer: Option<RingBuf>,
    index_buffer: Option<RingBuf>,
    pipelines: Pipelines,
    ring_buffer_size: usize,
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
            widgets: Default::default(),
            vertex_buffer: None,
            index_buffer: None,
            pipelines: Default::default(),
            ring_buffer_size: 1 << 23,
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

    pub fn add_widget(
        &mut self,
        size: [f32; 2],
        position: [f32; 2],
    ) -> WidgetId
    {
        WidgetId(self.widgets.insert(Widget::new(size, position)))
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

    pub fn render(
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
        render_commands.bind_pipeline(base_pipeline)?;
        let vertex_buffer = self.vertex_buffer.as_mut().unwrap();
        let index_buffer = self.index_buffer.as_mut().unwrap();
        for widget in &self.widgets {
            let mut points = GlobalVec::<[f32; 2]>::new();
            widget.main_rect.to_points(
                &mut |p| { points.push(p.into()); }
            );
            let (vertices, indices) = earcut(&points, &[], false).unwrap();
            let vert_alloc = vertex_buffer.allocate::<Vertex>(render_commands, vertices.len(), self.ring_buffer_size)?;
            unsafe {
                vertices.as_ptr().copy_to_nonoverlapping(vert_alloc.ptr.cast::<[f32; 2]>().as_ptr(), vertices.len());
            }
            let idx_alloc = index_buffer.allocate::<u32>(render_commands, indices.len(), self.ring_buffer_size)?;
            let idx_ptr = idx_alloc.ptr;
            for i in 0..indices.len() {
                unsafe {
                    idx_ptr
                        .add(i)
                        .write(indices[i] as u32);
                }
            }
            render_commands.draw_indexed(
                DrawInfo {
                    index_count: indices.len() as u32,
                    ..Default::default()
                },
                [
                    DrawBufferInfo {
                        id: vertex_buffer.buffer,
                        offset: vert_alloc.offset,
                    },
                ],
                DrawBufferInfo {
                    id: index_buffer.buffer,
                    offset: idx_alloc.offset,
                }
            )?;
        }
        vertex_buffer.finish_frame();
        index_buffer.finish_frame();
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
