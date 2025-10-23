use core::hash::Hash;

use rustc_hash::FxHashMap;

use nox::{
    mem::{
        vec_types::{GlobalVec, Vector},
        Allocator,
    },
    *
};

use nox_font::{text_segment, Face, VertexTextRenderer};

use nox_geom::*;

use crate::*;

pub(crate) const COLOR_PICKER_PIPELINE_HASH: &str = "nox_gui color picker";
pub(crate) const COLOR_PICKER_HUE_PIPELINE_HASH: &str = "nox_gui color picker hue";
pub(crate) const COLOR_PICKER_ALPHA_PIPELINE_HASH: &str = "nox_gui color picker alpha";

#[derive(Default)]
struct BasePipelines {
    base_pipeline_layout: Option<PipelineLayoutId>,
    base_pipeline: Option<GraphicsPipelineId>,
    text_pipeline_layout: Option<PipelineLayoutId>,
    text_pipeline: Option<GraphicsPipelineId>,
    base_shaders: Option<[ShaderId; 2]>,
    text_shaders: Option<[ShaderId; 2]>,
}

pub struct CustomPipelineInfo<'a> {
    pub vertex_shader: ShaderId,
    pub fragment_shader: ShaderId,
    pub vertex_input_bindings: &'a [VertexInputBinding],
}

impl<'a> CustomPipelineInfo<'a> {

    pub fn new(
        vertex_shader: ShaderId,
        fragment_shader: ShaderId,
        vertex_input_bindings: &'a [VertexInputBinding],
    ) -> Self
    {
        Self {
            vertex_shader,
            fragment_shader,
            vertex_input_bindings,
        }
    }
}

struct CustomPipeline {
    pub vertex_shader: ShaderId,
    fragment_shader: ShaderId,
    pipeline_layout: PipelineLayoutId,
    vertex_input_bindings: GlobalVec<VertexInputBinding>,
    pipeline: GraphicsPipelineId,
}

impl CustomPipeline {

    fn clean_up(&self, r: &mut GlobalResources) {
        r.destroy_shader(self.vertex_shader);
        r.destroy_shader(self.fragment_shader);
        r.destroy_pipeline_layout(self.pipeline_layout);
        r.destroy_graphics_pipeline(self.pipeline);
    }
}

pub struct Workspace<'a, I, FontHash, Style>
    where
        I: Interface,
        FontHash: Clone + PartialEq + Eq + Hash,
        Style: WindowStyle<FontHash>,
{
    text_renderer: VertexTextRenderer<'a, FontHash>,
    style: Style,
    windows: FxHashMap<u32, Window<I, FontHash, Style>>,
    active_windows: GlobalVec<u32>,
    vertex_buffer: Option<RingBuf>,
    index_buffer: Option<RingBuf>,
    base_pipelines: BasePipelines,
    custom_pipelines: FxHashMap<CompactString, CustomPipeline>,
    frame: u64,
    ring_buffer_size: usize,
    prev_cursor_position: Vec2,
    inv_aspect_ratio: f32,
    unit_scale: f32,
    flags: u32,
    min_sample_shading: f32,
    output_samples: MSAA,
    output_format: ColorFormat,
}

impl<'a, I, FontHash, Style> Workspace<'a, I, FontHash, Style>
    where
        I: Interface,
        FontHash: Clone + PartialEq + Eq + Hash,
        Style: WindowStyle<FontHash>,
{

    const BEGAN: u32 = 0x1;
    const CURSOR_IN_WINDOW: u32 = 0x2;

    const BLEND_STATE: ColorOutputBlendState = ColorOutputBlendState {
        src_color_blend_factor: BlendFactor::SrcAlpha,
        dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
        color_blend_op: BlendOp::Add,
        src_alpha_blend_factor: BlendFactor::One,
        dst_alpha_blend_factor: BlendFactor::OneMinusSrcAlpha,
        alpha_blend_op: BlendOp::Add,
    };

    pub fn new(
        fonts: impl IntoIterator<Item = (FontHash, Face<'a>)>,
        style: Style,
        font_curve_tolerance: f32,
    ) -> Self
    {
        let mut text_renderer = VertexTextRenderer::new(fonts, font_curve_tolerance);
        text_renderer.render(&[text_segment("0123456789", &style.font_regular())], false, 0.0);
        Self {
            text_renderer,
            style,
            windows: Default::default(),
            active_windows: Default::default(),
            vertex_buffer: None,
            index_buffer: None,
            base_pipelines: Default::default(),
            custom_pipelines: FxHashMap::default(),
            frame: 0,
            ring_buffer_size: 1 << 23,
            prev_cursor_position: Default::default(),
            inv_aspect_ratio: 0.0,
            unit_scale: 0.0,
            flags: 0,
            min_sample_shading: 0.2,
            output_samples: MSAA::None,
            output_format: Default::default(),
        }
    }

    /// (re)creates required graphics pipelines
    pub fn create_graphics_pipelines(
        &mut self,
        render_context: &mut RendererContext,
        output_samples: MSAA,
        output_format: ColorFormat,
        cache_id: Option<PipelineCacheId>,
        alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        if self.output_samples == output_samples && self.output_format == output_format {
            return Ok(())
        }
        if output_samples == MSAA::None {
            return Err(
                Error::UserError("nox_gui: output samples must be defined".into())
            )
        }
        self.output_samples = output_samples;
        self.output_format = output_format;
        let mut color_picker_shaders = [Default::default(); 4];
        render_context.edit_resources(|r| {
            color_picker_shaders[0] = r.create_shader(
                COLOR_PICKER_VERTEX_SHADER,
                "nox_gui color picker vertex shader", ShaderStage::Vertex
            )?;
            color_picker_shaders[1] = r.create_shader(
                COLOR_PICKER_FRAGMENT_SHADER,
                "nox_gui color picker fragment shader", ShaderStage::Fragment
            )?;
            color_picker_shaders[2] = r.create_shader(
                COLOR_PICKER_FRAGMENT_SHADER_HUE,
                "nox_gui color picker fragment shader hue", ShaderStage::Fragment
            )?;
            color_picker_shaders[3] = r.create_shader(
                COLOR_PICKER_FRAGMENT_SHADER_ALPHA,
                "nox_gui color picker fragment shader alpha", ShaderStage::Fragment
            )?;
            Ok(())
        })?;
        self.create_custom_pipelines(
            render_context,
            &[
                (
                    COLOR_PICKER_PIPELINE_HASH,
                    CustomPipelineInfo::new(
                        color_picker_shaders[0],
                        color_picker_shaders[1],
                        &[
                            VertexInputBinding
                                ::new::<0, ColorPickerVertex>(0, VertexInputRate::Vertex),
                        ],
                    )
                ),
                (
                    COLOR_PICKER_HUE_PIPELINE_HASH,
                    CustomPipelineInfo::new(
                        color_picker_shaders[0],
                        color_picker_shaders[2],
                        &[
                            VertexInputBinding
                                ::new::<0, ColorPickerVertex>(0, VertexInputRate::Vertex),
                        ],
                    )
                ),
                (
                    COLOR_PICKER_ALPHA_PIPELINE_HASH,
                    CustomPipelineInfo::new(
                        color_picker_shaders[0],
                        color_picker_shaders[3],
                        &[
                            VertexInputBinding
                                ::new::<0, ColorPickerVertex>(0, VertexInputRate::Vertex),
                        ],
                    ),
                ),
            ],
            cache_id,
            alloc
        )?;
        render_context.edit_resources(|r| {
            if let Some(pipeline) = self.base_pipelines.base_pipeline.take() {
                r.destroy_graphics_pipeline(pipeline);
            }
            if let Some(pipeline) = self.base_pipelines.text_pipeline.take() {
                r.destroy_graphics_pipeline(pipeline);
            }

            let &mut base_shaders = self.base_pipelines.base_shaders
                .get_or_insert([
                    r.create_shader(BASE_VERTEX_SHADER, "nox_gui base vertex shader", ShaderStage::Vertex)?,
                    r.create_shader(BASE_FRAGMENT_SHADER, "nox_gui base fragment shader", ShaderStage::Fragment)?,
                ]
            );
            let &mut text_shaders = self.base_pipelines.text_shaders
                .get_or_insert([
                    r.create_shader(TEXT_VERTEX_SHADER, "nox_gui text vertex shader", ShaderStage::Vertex)?,
                    r.create_shader(TEXT_FRAGMENT_SHADER, "nox_gui text fragment shader", ShaderStage::Fragment)?,
                ]
            );
            let &mut base_layout = self.base_pipelines.base_pipeline_layout.get_or_insert(
                r.create_pipeline_layout(base_shaders)?
            );
            let &mut text_layout = self.base_pipelines.text_pipeline_layout.get_or_insert(
                r.create_pipeline_layout(text_shaders)?
            );
            let mut base_info = GraphicsPipelineInfo::new(base_layout);
            let min_sample_shading = self.min_sample_shading;
            base_info
                .with_vertex_input_binding(VertexInputBinding::new::<0, Vertex>(0, VertexInputRate::Vertex))
                .with_sample_shading(SampleShadingInfo::new(output_samples, min_sample_shading, false, false))
                .with_color_output(
                    output_format,
                    WriteMask::all(),
                    Some(Self::BLEND_STATE)
                );
            let mut text_info = GraphicsPipelineInfo::new(text_layout);
            text_info
                .with_vertex_input_binding(
                    VertexInputBinding::new::<0, font::Vertex>(0, VertexInputRate::Vertex)
                )
                .with_vertex_input_binding(
                    VertexInputBinding::new::<1, font::VertexOffset>(1, VertexInputRate::Instance)
                )
                .with_vertex_input_binding(
                    VertexInputBinding::new::<2, BoundedTextInstance>(2, VertexInputRate::Instance)
                )
                .with_sample_shading(SampleShadingInfo::new(output_samples, min_sample_shading, false, false))
                .with_color_output(output_format, WriteMask::all(), Some(Self::BLEND_STATE));
            let mut custom_pipelines = GlobalVec::new();
            let mut pipeline_infos = GlobalVec::from(mem::slice![base_info, text_info]);
            for (_, pipeline) in &mut self.custom_pipelines {
                r.destroy_graphics_pipeline(pipeline.pipeline);
                let mut pipeline_info = GraphicsPipelineInfo::new(pipeline.pipeline_layout);
                for &binding in &pipeline.vertex_input_bindings {
                    pipeline_info
                        .with_vertex_input_binding(binding);
                }
                pipeline_info
                    .with_sample_shading(SampleShadingInfo::new(output_samples, min_sample_shading, false, false))
                    .with_color_output(
                        output_format,
                        WriteMask::all(),
                        Some(Self::BLEND_STATE)
                    );
                pipeline_infos.push(pipeline_info);
                custom_pipelines.push(pipeline);
            }
            r.create_graphics_pipelines(
                &pipeline_infos,
                cache_id,
                alloc,
                |i, p| {
                    if i == 0 {
                        self.base_pipelines.base_pipeline = Some(p)
                    } else if i == 1 {
                        self.base_pipelines.text_pipeline = Some(p);
                    } else {
                        custom_pipelines[i - 2].pipeline = p;
                    }
                }
            )?;
            Ok(())
        })
    }

    pub fn create_custom_pipelines<'b>(
        &mut self,
        render_context: &mut RendererContext,
        infos: &[(&'b str, CustomPipelineInfo<'b>)],
        cache_id: Option<PipelineCacheId>,
        alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        render_context.edit_resources(|r| {
            let mut pipelines = GlobalVec::new();
            let mut pipeline_infos = GlobalVec::new();
            let output_samples = self.output_samples;
            let output_format = self.output_format;
            for (hash, info) in infos {
                let hash = CompactString::new(hash);
                if self.custom_pipelines.contains_key(&hash) {
                    continue
                }
                let vertex_shader = info.vertex_shader;
                let fragment_shader = info.fragment_shader;
                let pipeline_layout = r
                    .create_pipeline_layout([vertex_shader, fragment_shader])?;
                let mut pipeline_info = GraphicsPipelineInfo::new(pipeline_layout);
                for &binding in info.vertex_input_bindings {
                    pipeline_info.with_vertex_input_binding(binding);
                }
                pipeline_info
                    .with_sample_shading(SampleShadingInfo::new(output_samples, self.min_sample_shading, false, false))
                    .with_color_output(
                        output_format,
                        WriteMask::all(),
                        Some(Self::BLEND_STATE),
                    );
                pipeline_infos.push(pipeline_info);
                pipelines.push((
                    Some(hash),
                    (
                        vertex_shader,
                        fragment_shader,
                        pipeline_layout,
                        info.vertex_input_bindings,
                    ),
                ));
            }
            r.create_graphics_pipelines(&pipeline_infos, cache_id, alloc,
                |i, p| {
                    let (hash, pipeline) = &mut pipelines[i];
                    self.custom_pipelines
                        .insert(
                            hash.take().unwrap(),
                            CustomPipeline {
                                vertex_shader: pipeline.0,
                                fragment_shader: pipeline.1,
                                pipeline_layout: pipeline.2,
                                vertex_input_bindings: pipeline.3.into(),
                                pipeline: p,
                            }
                        );
                }
            )?;
            Ok(())
        })
    }

    #[inline(always)]
    pub fn get_custom_pipeline(&self, key: &str) -> Option<GraphicsPipelineId> {
        self.custom_pipelines
            .get(key.into())
            .map(|v| v.pipeline)
    }

    #[inline(always)]
    fn began(&self) -> bool {
        self.flags & Self::BEGAN == Self::BEGAN
    }

    #[inline(always)]
    fn cursor_in_window(&self) -> bool {
        self.flags & Self::CURSOR_IN_WINDOW == Self::CURSOR_IN_WINDOW
    }

    #[inline(always)]
    pub fn begin(&mut self) -> Result<(), Error>
    {
        if self.began() {
            return Err(Error::UserError(
                "nox_gui: attempting to call Workspace::begin twice before calling Workspace::end".into()
            ))
        }
        if let Some(buf) = &mut self.vertex_buffer {
            buf.finish_frame();
        }
        if let Some(buf) = &mut self.index_buffer {
            buf.finish_frame();
        }
        self.frame += 1;
        self.flags |= Self::BEGAN;
        Ok(())
    }

    pub fn update_window<F>(
        &mut self,
        id: u32,
        title: &str,
        initial_position: [f32; 2],
        initial_size: [f32; 2],
        mut f: F,
    ) -> Result<(), Error>
        where
            F: FnMut(&mut WindowContext<I, FontHash, Style>)
    {
        if !self.began() {
            return Err(Error::UserError(
                "nox_gui: attempting to update window before calling Workspace::begin".into()
            ));
        }
        let window = self.windows.entry(id).or_insert(Window::new(
            title,
            initial_position,
            initial_size,
        ));
        window.set_last_frame(self.frame);
        f(&mut WindowContext::new(title, window, &self.style, &mut self.text_renderer));
        if !self.active_windows.contains(&id) {
            self.active_windows.push(id);
        }
        Ok(())
    }

    pub fn end(
        &mut self,
        nox: &mut Nox<'_, I>,
    ) -> Result<(), Error>
    {
        if !self.began() {
            return Err(Error::UserError(
                "nox_gui: attempting to call Workspace::end before calling Workspace::begin".into()
            ))
        }
        self.active_windows.retain(|id| {
            let win = self.windows.get(id).unwrap();
            win.last_frame() == self.frame
        });
        let aspect_ratio = nox.aspect_ratio() as f32;
        self.inv_aspect_ratio = 1.0 / aspect_ratio;
        let window_size: Vec2 = nox.window_size_f32().into();
        let unit_scale = window_size.y / self.style.pixels_per_unit();
        self.unit_scale = unit_scale;
        let mut cursor_pos: Vec2 = nox.normalized_cursor_position_f32().into();
        cursor_pos *= 2.0;
        cursor_pos -= vec2(1.0, 1.0);
        cursor_pos.x *= aspect_ratio;
        cursor_pos /= unit_scale;
        let delta_cursor_pos = cursor_pos - self.prev_cursor_position;
        self.prev_cursor_position = cursor_pos;
        let mut cursor_in_some_window = false;
        let mut window_pressed = None;
        for (i, id) in self.active_windows.iter_mut().enumerate().rev() {
            let window = self.windows.get_mut(id).unwrap();
            let cursor_in_window = window.update(
                nox,
                &self.style,
                &mut self.text_renderer,
                cursor_pos,
                delta_cursor_pos,
                cursor_in_some_window,
                window_size,
                aspect_ratio,
                unit_scale,
            );
            if cursor_in_window && nox.was_mouse_button_pressed(MouseButton::Left) {
                window_pressed = Some(i);
            }
            cursor_in_some_window |= cursor_in_window;
            window.triangulate();
            window.refresh_position(aspect_ratio, unit_scale, window_size);
        }
        if let Some(idx) = window_pressed {
            let id = self.active_windows.remove(idx);
           self.active_windows.push(id);
        }
        if self.cursor_in_window() && !cursor_in_some_window && self.style.override_cursor() {
            nox.set_cursor(CursorIcon::Default);
        }
        self.flags &= !(Self::CURSOR_IN_WINDOW | Self::BEGAN);
        self.flags |= Self::CURSOR_IN_WINDOW * cursor_in_some_window as u32;
        Ok(())
    }

    pub fn render_commands(
        &mut self,
        render_commands: &mut RenderCommands,
    ) -> Result<(), Error>
    {
        if self.vertex_buffer.is_none() {
            self.init_buffers(render_commands)?;
        } 
        let Some(base_pipeline) = self.base_pipelines.base_pipeline else {
            return Err(Error::UserError(
                "nox_gui: attempting to render Workspace before creating graphics pipelines".into()
            ))
        };
        let Some(text_pipeline) = self.base_pipelines.text_pipeline else {
            return Err(Error::UserError(
                "nox_gui: attempting to render Workspace before creating graphics pipelines".into()
            ))
        };
        let inv_aspect_ratio = self.inv_aspect_ratio;
        let unit_scale = self.unit_scale;
        let vertex_buffer = self.vertex_buffer.as_mut().unwrap();
        let index_buffer = self.index_buffer.as_mut().unwrap();
        for id in &self.active_windows {
            self.windows.get_mut(id).unwrap().render_commands(
                render_commands,
                &self.style,
                base_pipeline,
                text_pipeline,
                vertex_buffer,
                index_buffer,
                inv_aspect_ratio,
                unit_scale,
                &mut |hash| {
                    self.custom_pipelines
                        .get(&CompactString::new(hash))
                        .map(|v| v.pipeline)
                }
            )?;
        }
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
            if let Some(pipeline) = self.base_pipelines.base_pipeline.take() {
                r.destroy_graphics_pipeline(pipeline);
            }
            for pipeline in &self.custom_pipelines {
                pipeline.1.clean_up(r);
            }
            self.custom_pipelines.clear();
            Ok(())
        }).ok();
    }
}
