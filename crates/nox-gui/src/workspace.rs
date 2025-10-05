use core::hash::Hash;

use rustc_hash::FxHashMap;

use nox::{
    mem::{
        vec_types::{GlobalVec, Vector},
        Allocator,
    },
    *
};

use nox_font::{VertexTextRenderer, Face};

use nox_geom::{
    Vec2,
    vec2,
};

use crate::*;

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
    style: Style<FontHash>,
    windows: FxHashMap<u32, Window>,
    active_windows: GlobalVec<u32>,
    vertex_buffer: Option<RingBuf>,
    index_buffer: Option<RingBuf>,
    pipelines: Pipelines,
    ring_buffer_size: usize,
    prev_cursor_position: Vec2,
    inv_aspect_ratio: f32,
}

impl<'a, FontHash> Workspace<'a, FontHash>
    where
        FontHash: Clone + PartialEq + Eq + Hash
{

    pub fn new(
        fonts: impl IntoIterator<Item = (FontHash, Face<'a>)>,
        font_regular: FontHash,
        font_curve_tolerance: f32,
    ) -> Self
    {
        Self {
            text_renderer: VertexTextRenderer::new(fonts, font_curve_tolerance),
            style: Style::new(font_regular),
            windows: Default::default(),
            active_windows: Default::default(),
            vertex_buffer: None,
            index_buffer: None,
            pipelines: Default::default(),
            ring_buffer_size: 1 << 23,
            prev_cursor_position: Default::default(),
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
            let &mut base_shaders = self.pipelines.base_shaders
                .get_or_insert([
                    v.create_shader(BASE_VERTEX_SHADER, "nox_gui base vertex shader", ShaderStage::Vertex)?,
                    v.create_shader(BASE_FRAGMENT_SHADER, "nox_gui base fragment shader", ShaderStage::Fragment)?,
                ]
            );
            let &mut base_layout = self.pipelines.base_pipeline_layout.get_or_insert(
                v.create_pipeline_layout(base_shaders)?
            );
            let mut base_info = GraphicsPipelineInfo::new(base_layout);
            base_info
                .with_vertex_input_binding(VertexInputBinding::new::<0, Vertex>(0, VertexInputRate::Vertex))
                .with_vertex_input_binding(VertexInputBinding::new::<1, font::VertexOffset>(1, VertexInputRate::Instance))
                .with_sample_shading(SampleShadingInfo::new(samples, 0.2, false, false))
                .with_color_output(output_format, WriteMask::all(), None);
            v.create_graphics_pipelines(
                &[base_info],
                cache_id,
                alloc,
                |_, p| {
                    self.pipelines.base_pipeline = Some(p)
                }
            )?;
            Ok(())
        })
    }

    pub fn update_window<F>(
        &mut self,
        id: u32,
        title: &str,
        initial_size: [f32; 2],
        initial_position: [f32; 2],
        mut f: F,
    ) -> Result<(), Error>
        where
            F: FnMut(WindowContext<FontHash>) -> Result<(), Error>
    {
        let window = self.windows.entry(id).or_insert(Window::new(
            title,
            initial_size,
            initial_position,
            self.style.rounding,
        ));
        f(WindowContext::new(window, &self.style, &mut self.text_renderer))?;
        self.active_windows.push(id);
        Ok(())
    }

    pub fn end<I: Interface>(
        &mut self,
        nox: &Nox<'_, I>,
    )
    {
        let aspect_ratio = nox.aspect_ratio() as f32;
        self.inv_aspect_ratio = 1.0 / aspect_ratio;
        let mut cursor_pos: Vec2 = nox.normalized_cursor_position_f32().into();
        cursor_pos *= 2.0;
        cursor_pos -= vec2(1.0, 1.0);
        cursor_pos.x *= aspect_ratio;
        let delta_cursor_pos = cursor_pos - self.prev_cursor_position;
        self.prev_cursor_position = cursor_pos;
        let mut cursor_in_window = false;
        for id in &self.active_windows {
            let window = self.windows.get_mut(id).unwrap();
            cursor_in_window |= window.update(
                nox,
                &self.style,
                &mut self.text_renderer,
                cursor_pos,
                delta_cursor_pos,
                cursor_in_window
            );
            window.triangulate();
        }
        if !cursor_in_window && self.style.override_cursor {
            nox.set_cursor(CursorIcon::Default);
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
        let vertex_buffer = self.vertex_buffer.as_mut().unwrap();
        let index_buffer = self.index_buffer.as_mut().unwrap();
        let no_offset = unsafe {
            vertex_buffer.allocate::<font::VertexOffset>(render_commands, 1)?
        };
        unsafe {
            let tmp = font::VertexOffset {
                offset: [0.0, 0.0],
            };
            (&tmp as *const font::VertexOffset)
                .copy_to_nonoverlapping(no_offset.ptr.as_ptr(), 1);
        }
        for id in &self.active_windows {
            self.windows.get_mut(id).unwrap().render_commands(
                render_commands,
                &self.style,
                inv_aspect_ratio,
                vertex_buffer,
                index_buffer,
                base_pipeline,
                DrawBufferInfo { id: vertex_buffer.id(), offset: no_offset.offset }
            )?;
        }
        vertex_buffer.finish_frame();
        index_buffer.finish_frame();
        self.active_windows.clear();
        Ok(())
    }

    fn init_buffers(&mut self, render_commands: &mut RenderCommands) -> Result<(), Error> {
        let buffered_frames = render_commands.buffered_frames();
        render_commands.edit_resources(|r| {
            let vertex_buffer = r.create_buffer(
                self.ring_buffer_size as u64,
                &[BufferUsage::VertexBuffer],
                &mut r.default_binder_mappable()
            )?;
            let vertex_buffer_map = unsafe {
                r.map_buffer(vertex_buffer).unwrap()
            };
            self.vertex_buffer = Some(RingBuf::new(
                vertex_buffer,
                vertex_buffer_map,
                buffered_frames,
                self.ring_buffer_size,
            )?);
            let index_buffer = r.create_buffer(
                self.ring_buffer_size as u64,
                &[BufferUsage::IndexBuffer],
                &mut r.default_binder_mappable()
            )?;
            let index_buffer_map = unsafe {
                r.map_buffer(index_buffer).unwrap()
            };
            self.index_buffer = Some(RingBuf::new(
                index_buffer,
                index_buffer_map, 
                buffered_frames,
                self.ring_buffer_size,
            )?);
            Ok(())
        })?;
        Ok(())
    }

    pub fn clean_up(&mut self, context: &RendererContext) {
        context.edit_resources(|r| {
            if let Some(buf) = self.vertex_buffer.take() {
                r.destroy_buffer(buf.id());
            };
            if let Some(buf) = self.index_buffer.take() {
                r.destroy_buffer(buf.id());
            }
            if let Some(pipeline) = self.pipelines.base_pipeline.take() {
                r.destroy_graphics_pipeline(pipeline);
            }
            Ok(())
        }).ok();
    }
}
