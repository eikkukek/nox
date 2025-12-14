use rustc_hash::FxHashMap;

use nox::{
    alloc::arena_alloc::{ArenaAlloc, ArenaGuard}, error::Context, mem::{
        Allocator, vec_types::{GlobalVec, Vector}
    }, *
};

use nox_font::{text_segment, Face};

use nox_geom::*;

use crate::*;
use crate::error::{Error, Result};

pub(crate) const COLOR_PICKER_PIPELINE_HASH: &str = "nox_gui color picker";
pub(crate) const COLOR_PICKER_HUE_PIPELINE_HASH: &str = "nox_gui color picker hue";
pub(crate) const COLOR_PICKER_ALPHA_PIPELINE_HASH: &str = "nox_gui color picker alpha";

#[derive(Default)]
struct BasePipelines {
    base_pipeline_layout: Option<gpu::PipelineLayoutId>,
    base_pipeline: gpu::GraphicsPipelineId,
    text_pipeline_layout: Option<gpu::PipelineLayoutId>,
    text_pipeline: gpu::GraphicsPipelineId,
    texture_pipeline_layout: Option<gpu::PipelineLayoutId>,
    texture_pipeline: gpu::GraphicsPipelineId,
    base_shaders: Option<[gpu::ShaderId; 2]>,
    text_shaders: Option<[gpu::ShaderId; 2]>,
    texture_shaders: Option<[gpu::ShaderId; 2]>,
}

impl BasePipelines {

    pub fn clean_up(&mut self, gpu: &mut gpu::GpuContext) -> Result<()> {
        gpu.destroy_graphics_pipeline(self.base_pipeline);
        gpu.destroy_graphics_pipeline(self.text_pipeline);
        gpu.destroy_graphics_pipeline(self.texture_pipeline);
        if let Some(layout) = self.base_pipeline_layout {
            gpu.destroy_pipeline_layout(layout);
        }
        if let Some(layout) = self.text_pipeline_layout {
            gpu.destroy_pipeline_layout(layout);
        }
        if let Some(layout) = self.texture_pipeline_layout {
            gpu.destroy_pipeline_layout(layout);
        }
        if let Some(shaders) = self.base_shaders {
            gpu.destroy_shader(shaders[0]);
            gpu.destroy_shader(shaders[1]);
        }
        if let Some(shaders) = self.text_shaders {
            gpu.destroy_shader(shaders[0]);
            gpu.destroy_shader(shaders[1]);
        }
        if let Some(shaders) = self.texture_shaders {
            gpu.destroy_shader(shaders[0]);
            gpu.destroy_shader(shaders[1]);
        }
        Ok(())
    }
}

pub struct CustomPipelineInfo<'a> {
    pub vertex_shader: gpu::ShaderId,
    pub fragment_shader: gpu::ShaderId,
    pub vertex_input_bindings: &'a [gpu::VertexInputBinding],
}

impl<'a> CustomPipelineInfo<'a> {

    pub fn new(
        vertex_shader: gpu::ShaderId,
        fragment_shader: gpu::ShaderId,
        vertex_input_bindings: &'a [gpu::VertexInputBinding],
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
    pub vertex_shader: gpu::ShaderId,
    fragment_shader: gpu::ShaderId,
    pipeline_layout: gpu::PipelineLayoutId,
    vertex_input_bindings: GlobalVec<gpu::VertexInputBinding>,
    pipeline: gpu::GraphicsPipelineId,
}

impl CustomPipeline {

    fn clean_up(&self, r: &mut gpu::GlobalResources) {
        r.destroy_shader(self.vertex_shader);
        r.destroy_shader(self.fragment_shader);
        r.destroy_pipeline_layout(self.pipeline_layout);
        r.destroy_graphics_pipeline(self.pipeline);
    }
}

pub struct Workspace<'a, Style>
    where
        Style: UiStyle,
{
    text_renderer: TextRenderer<'a>,
    style: Style,
    windows: FxHashMap<u32, Window>,
    active_windows: GlobalVec<u32>,
    main_pass_id: gpu::PassId,
    window_passes: FxHashMap<gpu::PassId, u32>,
    vertex_buffer: Option<RingBuf>,
    index_buffer: Option<RingBuf>,
    tmp_alloc: ArenaAlloc,
    image_loader: ImageLoader,
    device_alloc: Option<gpu::LinearDeviceAllocId>,
    device_alloc_block_size: u64,
    base_pipelines: BasePipelines,
    custom_pipelines: FxHashMap<CompactString, CustomPipeline>,
    frame: u64,
    ring_buffer_size: usize,
    prev_cursor_position: Vec2,
    inv_aspect_ratio: f32,
    unit_scale: f32,
    flags: u32,
    min_sample_shading: f32,
    output_samples: gpu::MSAA,
    output_format: gpu::ColorFormat,
}

impl<'a, Style> Workspace<'a, Style>
    where
        Style: UiStyle,
{

    const BEGAN: u32 = 0x1;
    const CURSOR_IN_WINDOW: u32 = 0x2;
    const REQUIRES_TRANSFER_COMMANDS: u32 = 0x4;

    const BLEND_STATE: gpu::ColorOutputBlendState = gpu::ColorOutputBlendState {
        src_color_blend_factor: gpu::BlendFactor::SrcAlpha,
        dst_color_blend_factor: gpu::BlendFactor::OneMinusSrcAlpha,
        color_blend_op: gpu::BlendOp::Add,
        src_alpha_blend_factor: gpu::BlendFactor::One,
        dst_alpha_blend_factor: gpu::BlendFactor::OneMinusSrcAlpha,
        alpha_blend_op: gpu::BlendOp::Add,
    };

    pub fn init(
        fonts: impl IntoIterator<Item = (impl Into<CompactString>, Face<'a>)>,
        style: Style,
        font_curve_tolerance: f32,
        gpu: &mut gpu::GpuContext,
        device_alloc_block_size: u64,
        pipeline_cache: Option<gpu::PipelineCacheId>,
        output_format: gpu::ColorFormat,
        output_samples: gpu::MSAA,
        tmp_alloc: &impl Allocator,
    ) -> Result<Self>
    {
        let mut text_renderer = TextRenderer::new(fonts, font_curve_tolerance);
        text_renderer.render(&[text_segment("0123456789", style.font_regular())], false, 0.0);
        let mut s = Self {
            text_renderer,
            style,
            windows: Default::default(),
            active_windows: Default::default(),
            main_pass_id: Default::default(),
            window_passes: FxHashMap::default(),
            vertex_buffer: None,
            index_buffer: None,
            tmp_alloc: ArenaAlloc::new(1 << 16).unwrap(),
            image_loader: ImageLoader::new(),
            device_alloc: None,
            device_alloc_block_size,
            base_pipelines: Default::default(),
            custom_pipelines: FxHashMap::default(),
            frame: 0,
            ring_buffer_size: 1 << 23,
            prev_cursor_position: Default::default(),
            inv_aspect_ratio: 0.0,
            unit_scale: 0.0,
            flags: 0,
            min_sample_shading: 0.2,
            output_samples: gpu::MSAA::None,
            output_format: Default::default(),
        };
        s.recreate_graphics_pipelines(
            gpu,
            output_samples,
            output_format,
            pipeline_cache,
            tmp_alloc,
        )?;
        let mut color_picker_shaders = [Default::default(); 4];
        color_picker_shaders[0] = gpu.create_shader(
            COLOR_PICKER_VERTEX_SHADER,
            "nox_gui color picker vertex shader", gpu::ShaderStage::Vertex
        )?;
        color_picker_shaders[1] = gpu.create_shader(
            COLOR_PICKER_FRAGMENT_SHADER,
            "nox_gui color picker fragment shader", gpu::ShaderStage::Fragment
        )?;
        color_picker_shaders[2] = gpu.create_shader(
            COLOR_PICKER_FRAGMENT_SHADER_HUE,
            "nox_gui color picker fragment shader hue", gpu::ShaderStage::Fragment
        )?;
        color_picker_shaders[3] = gpu.create_shader(
            COLOR_PICKER_FRAGMENT_SHADER_ALPHA,
            "nox_gui color picker fragment shader alpha", gpu::ShaderStage::Fragment
        )?;
        s.create_custom_pipelines(
            gpu,
            &[
                (
                    COLOR_PICKER_PIPELINE_HASH,
                    CustomPipelineInfo::new(
                        color_picker_shaders[0],
                        color_picker_shaders[1],
                        &[
                            gpu::VertexInputBinding
                                ::new::<0, ColorPickerVertex>(0, gpu::VertexInputRate::Vertex),
                        ],
                    )
                ),
                (
                    COLOR_PICKER_HUE_PIPELINE_HASH,
                    CustomPipelineInfo::new(
                        color_picker_shaders[0],
                        color_picker_shaders[2],
                        &[
                            gpu::VertexInputBinding
                                ::new::<0, ColorPickerVertex>(0, gpu::VertexInputRate::Vertex),
                        ],
                    )
                ),
                (
                    COLOR_PICKER_ALPHA_PIPELINE_HASH,
                    CustomPipelineInfo::new(
                        color_picker_shaders[0],
                        color_picker_shaders[3],
                        &[
                            gpu::VertexInputBinding
                                ::new::<0, ColorPickerVertex>(0, gpu::VertexInputRate::Vertex),
                        ],
                    ),
                ),
            ],
            pipeline_cache,
            tmp_alloc,
        )?;
        Ok(s)
    }
    
    /// Recreates graphics pipelines.
    pub fn recreate_graphics_pipelines(
        &mut self,
        gpu: &mut gpu::GpuContext,
        output_samples: gpu::MSAA,
        output_format: gpu::ColorFormat,
        cache_id: Option<gpu::PipelineCacheId>,
        alloc: &impl Allocator,
    ) -> Result<()>
    {
        if self.output_samples == output_samples && self.output_format == output_format {
            return Ok(())
        }
        if output_samples == gpu::MSAA::None {
            return Err(Error::just_context("MSAA samples undefined"))
        }
        self.output_samples = output_samples;
        self.output_format = output_format; 
        gpu.destroy_graphics_pipeline(self.base_pipelines.base_pipeline);
        gpu.destroy_graphics_pipeline(self.base_pipelines.text_pipeline);
        gpu.destroy_graphics_pipeline(self.base_pipelines.texture_pipeline);
        let base_shaders =
            if let Some(shaders) = self.base_pipelines.base_shaders {
                shaders
            } else {
                *self.base_pipelines.base_shaders.insert([
                    gpu.create_shader(BASE_VERTEX_SHADER, "nox_gui base vertex shader", gpu::ShaderStage::Vertex)?,
                    gpu.create_shader(BASE_FRAGMENT_SHADER, "nox_gui base fragment shader", gpu::ShaderStage::Fragment)?,
                ])
            };
        let text_shaders =
            if let Some(shaders) = self.base_pipelines.text_shaders {
                shaders
            } else {
                *self.base_pipelines.text_shaders.insert([
                    gpu.create_shader(TEXT_VERTEX_SHADER, "nox_gui text vertex shader", gpu::ShaderStage::Vertex)?,
                    gpu.create_shader(TEXT_FRAGMENT_SHADER, "nox_gui text fragment shader", gpu::ShaderStage::Fragment)?,
                ])
            };
        let texture_shaders =
            if let Some(shaders) = self.base_pipelines.texture_shaders {
                shaders
            } else {
                *self.base_pipelines.texture_shaders.insert([
                    gpu.create_shader(TEXTURE_VERTEX_SHADER, "nox_gui texture vertex shader", gpu::ShaderStage::Vertex)?,
                    gpu.create_shader(TEXTURE_FRAGMENT_SHADER, "nox_gui texture fragment shader", gpu::ShaderStage::Fragment)?
                ])
            };
        let base_layout =
            if let Some(layout) = self.base_pipelines.base_pipeline_layout {
                layout
            } else {
                *self.base_pipelines.base_pipeline_layout.insert(gpu.create_pipeline_layout(base_shaders)?)
            };
        let text_layout = 
            if let Some(layout) = self.base_pipelines.text_pipeline_layout {
                layout
            } else {
                *self.base_pipelines.text_pipeline_layout.insert(gpu.create_pipeline_layout(text_shaders)?)
            };
        let texture_layout =
            if let Some(layout) = self.base_pipelines.texture_pipeline_layout {
                layout
            } else {
                *self.base_pipelines.texture_pipeline_layout.insert(gpu.create_pipeline_layout(texture_shaders)?)
            };
        let mut base_info = gpu::GraphicsPipelineInfo::new(base_layout);
        let min_sample_shading = self.min_sample_shading;
        base_info
            .with_vertex_input_binding(gpu::VertexInputBinding::new::<0, Vertex>(0, gpu::VertexInputRate::Vertex))
            .with_sample_shading(gpu::SampleShadingInfo::new(output_samples, min_sample_shading, false, false))
            .with_color_output(
                output_format,
                gpu::WriteMask::all(),
                Some(Self::BLEND_STATE)
            );
        let mut text_info = gpu::GraphicsPipelineInfo::new(text_layout);
        text_info
            .with_vertex_input_binding(
                gpu::VertexInputBinding::new::<0, font::Vertex>(0, gpu::VertexInputRate::Vertex)
            )
            .with_vertex_input_binding(
                gpu::VertexInputBinding::new::<1, font::VertexOffset>(1, gpu::VertexInputRate::Instance)
            )
            .with_vertex_input_binding(
                gpu::VertexInputBinding::new::<2, BoundedTextInstance>(2, gpu::VertexInputRate::Instance)
            )
            .with_sample_shading(gpu::SampleShadingInfo::new(output_samples, min_sample_shading, false, false))
            .with_color_output(output_format, gpu::WriteMask::all(), Some(Self::BLEND_STATE));
        let mut texture_info = gpu::GraphicsPipelineInfo::new(texture_layout);
        texture_info
            .with_sample_shading(gpu::SampleShadingInfo::new(output_samples, min_sample_shading, false, false))
            .with_color_output(output_format, gpu::WriteMask::all(), Some(Self::BLEND_STATE));
        let mut custom_pipelines = GlobalVec::new();
        let mut pipeline_infos = GlobalVec::from(mem::slice![base_info, text_info, texture_info]);
        for (_, pipeline) in &mut self.custom_pipelines {
            gpu.destroy_graphics_pipeline(pipeline.pipeline);
            let mut pipeline_info = gpu::GraphicsPipelineInfo::new(pipeline.pipeline_layout);
            for &binding in &pipeline.vertex_input_bindings {
                pipeline_info
                    .with_vertex_input_binding(binding);
            }
            pipeline_info
                .with_sample_shading(gpu::SampleShadingInfo::new(output_samples, min_sample_shading, false, false))
                .with_color_output(
                    output_format,
                    gpu::WriteMask::all(),
                    Some(Self::BLEND_STATE)
                );
            pipeline_infos.push(pipeline_info);
            custom_pipelines.push(pipeline);
        }
        gpu.create_graphics_pipelines(
            &pipeline_infos,
            cache_id,
            alloc,
            |i, p| {
                if i == 0 {
                    self.base_pipelines.base_pipeline = p;
                } else if i == 1 {
                    self.base_pipelines.text_pipeline = p;
                } else if i == 2 {
                    self.base_pipelines.texture_pipeline = p;
                } else {
                    custom_pipelines[i - 3].pipeline = p;
                }
            }
        )?;
        Ok(())
    }

    pub fn create_custom_pipelines<'b>(
        &mut self,
        gpu: &mut gpu::GpuContext,
        infos: &[(&'b str, CustomPipelineInfo<'b>)],
        cache_id: Option<gpu::PipelineCacheId>,
        alloc: &impl Allocator,
    ) -> Result<()>
    {
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
            let pipeline_layout = gpu
                .create_pipeline_layout([vertex_shader, fragment_shader])?;
            let mut pipeline_info = gpu::GraphicsPipelineInfo::new(pipeline_layout);
            for &binding in info.vertex_input_bindings {
                pipeline_info.with_vertex_input_binding(binding);
            }
            pipeline_info
                .with_sample_shading(gpu::SampleShadingInfo::new(output_samples, self.min_sample_shading, false, false))
                .with_color_output(
                    output_format,
                    gpu::WriteMask::all(),
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
        gpu.create_graphics_pipelines(&pipeline_infos, cache_id, alloc,
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
    }

    #[inline(always)]
    pub fn get_custom_pipeline(&self, key: &str) -> Option<gpu::GraphicsPipelineId> {
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
    fn requires_transfer_commands(&self) -> bool {
        self.flags & Self::REQUIRES_TRANSFER_COMMANDS == Self::REQUIRES_TRANSFER_COMMANDS
    }

    #[inline(always)]
    pub fn begin(
        &mut self,
        win: &win::WindowContext
    ) -> Result<()>
    {
        if self.began() {
            return Err(Error::just_context("end not called"));
        }
        self.window_passes.clear();
        self.unit_scale = self.style.pixels_per_unit() / win.window_size_f32().1;
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

    pub fn window<F>(
        &mut self,
        win: &mut win::WindowContext,
        id: u32,
        title: &str,
        initial_position: [f32; 2],
        initial_size: [f32; 2],
        mut f: F,
    ) -> Result<()>
        where
            F: FnMut(&mut UiContext<Window, Style>),
    {
        if !self.began() {
            return Err(Error::just_context("begin not called"));
        }
        let window = self.windows.entry(id).or_insert_with(|| Window::new(
            title,
            initial_position,
            initial_size,
        ));
        window.set_last_frame(self.frame);
        window.begin();
        if title != window.title {
            window.title = title.into();
            window.title_text = None;
        }
        let title_text = window.title_text.get_or_insert_with(|| self.text_renderer.render(
            &[text_segment(window.title.as_str(), self.style.font_regular())],
            false,
            0.0,
        ).unwrap_or_default());
        let start_off = vec2(
            self.style.item_pad_outer().x,
            self.style.calc_text_box_height_from_text_height(
                title_text.row_height * self.style.font_scale() * self.style.title_add_scale()) +
                self.style.item_pad_outer().y,
        );
        f(&mut UiContext::new(
            win,
            window,
            &self.style,
            start_off,
            &mut self.text_renderer,
            &mut self.image_loader,
        ));
        if !self.active_windows.contains(&id) {
            self.active_windows.push(id);
        }
        Ok(())
    }

    pub fn end(
        &mut self,
        win: &mut win::WindowContext,
        gpu: &mut gpu::GpuContext,
    ) -> Result<()>
    {
        if !self.began() {
            return Err(Error::just_context("begin not called"))
        }
        self.active_windows.retain(|id| {
            let win = self.windows.get(id).unwrap();
            win.last_frame() == self.frame
        });
        let aspect_ratio = win.aspect_ratio() as f32;
        self.inv_aspect_ratio = 1.0 / aspect_ratio;
        let window_size: Vec2 = win.window_size_f32().into();
        let unit_scale = self.unit_scale;
        let mut cursor_pos: Vec2 = win.normalized_cursor_position_f32().into();
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
            let tmp_alloc = ArenaGuard::new(&self.tmp_alloc);
            let WindowUpdateResult { cursor_in_window, requires_transfer_commands } = window.update(
                win,
                gpu,
                &self.style,
                &mut self.text_renderer,
                cursor_pos,
                delta_cursor_pos,
                cursor_in_some_window,
                window_size,
                aspect_ratio,
                unit_scale,
                &tmp_alloc,
            )?;
            if cursor_in_window && win.mouse_button_state(win::MouseButton::Left).pressed() {
                window_pressed = Some(i);
            }
            cursor_in_some_window |= cursor_in_window;
            or_flag!(self.flags, Self::REQUIRES_TRANSFER_COMMANDS, requires_transfer_commands);
            window.triangulate();
            window.refresh_position(aspect_ratio, unit_scale, window_size);
        }
        if let Some(idx) = window_pressed {
            let id = self.active_windows.remove(idx);
           self.active_windows.push(id);
        }
        if self.cursor_in_window() && !cursor_in_some_window && self.style.override_cursor() {
            win.set_cursor(win::CursorIcon::Default);
        }
        self.flags &= !(Self::CURSOR_IN_WINDOW | Self::BEGAN);
        self.flags |= Self::CURSOR_IN_WINDOW * cursor_in_some_window as u32;
        Ok(())
    }

    pub fn render(
        &mut self,
        frame_graph: &mut gpu::FrameGraph,
        output_image: (gpu::ResourceId, Option<gpu::ImageRangeInfo>),
        resolve_image: Option<gpu::WriteResolveInfo>,
        load_op: gpu::AttachmentLoadOp,
        clear_value: gpu::ClearColorValue,
    ) -> Result<()>
    {
        let mut reads = GlobalVec::new();
        let mut signal_semaphores = GlobalVec::new();
        let output_samples = self.output_samples;
        let output_format = self.output_format;
        for &id in &self.active_windows {
            self.windows    
                .get_mut(&id)
                .unwrap()
                .render(
                    frame_graph,
                    output_format,
                    &mut |read| {
                        reads.push(read);
                    },
                    &mut |id, value| {
                        signal_semaphores.push((id, value));
                    },
                )?;
        }
        self.main_pass_id = frame_graph.add_pass(
            gpu::PassInfo {
                max_reads: reads.len() as u32,
                max_color_writes: 1,
                msaa_samples: output_samples,
                signal_semaphores: signal_semaphores.len() as u32,
                ..Default::default()
            },
            |pass| {
                for &read in &reads {
                    pass.with_read(read)?;
                }
                for &(id, value) in &signal_semaphores {
                    pass.with_signal_semaphore(id, value)?;
                }
                pass.with_write(gpu::WriteInfo::new(output_image.0)
                    .with_range(output_image.1)
                    .with_resolve(resolve_image)
                    .with_load_op(load_op)
                    .with_store_op(gpu::AttachmentStoreOp::Store)
                    .with_clear_value(clear_value)
                )?;
                Ok(())
            }
        )?;
        Ok(())
    }

    pub fn render_work(
        &mut self,
        commands: &mut gpu::RenderCommands,
        pass_id: gpu::PassId,
        sampler: gpu::SamplerId,
    ) -> Result<()>
    {
        if self.vertex_buffer.is_none() {
            self.init_buffers(commands.gpu_mut())?;
        } 
        let base_pipeline = self.base_pipelines.base_pipeline;
        let text_pipeline = self.base_pipelines.text_pipeline;
        let texture_pipeline = self.base_pipelines.texture_pipeline;
        let texture_pipeline_layout = self.base_pipelines.texture_pipeline_layout.unwrap();
        let inv_aspect_ratio = self.inv_aspect_ratio;
        let unit_scale = self.unit_scale;
        let requires_transfer_commands = self.requires_transfer_commands();
        let vertex_buffer = self.vertex_buffer.as_mut().unwrap();
        let index_buffer = self.index_buffer.as_mut().unwrap();
        if pass_id == self.main_pass_id {
            if requires_transfer_commands {
                let device_alloc = self.device_alloc.unwrap();
                unsafe {
                    commands.reset_linear_device_alloc(device_alloc)?;
                }
                let tmp_alloc = ArenaGuard::new(&self.tmp_alloc);
                commands.synced_transfer_commands(
                    device_alloc,
                    |cmd| {
                        for id in &self.active_windows {
                            let window = self.windows.get_mut(id).unwrap();
                            window.transfer_work(
                                cmd, sampler,
                                texture_pipeline_layout,
                                &tmp_alloc,
                            ).context("window transfer work failed")?;
                        }
                        Ok(())
                    },
                )?;
                self.flags &= !Self::REQUIRES_TRANSFER_COMMANDS;
            }
            let tmp_alloc = ArenaGuard::new(&self.tmp_alloc);
            for id in &self.active_windows {
                let window = self.windows.get_mut(id).unwrap();
                window.render_work(
                    commands,
                    &self.style,
                    sampler,
                    pass_id,
                    base_pipeline,
                    text_pipeline,
                    texture_pipeline,
                    texture_pipeline_layout,
                    vertex_buffer,
                    index_buffer,
                    inv_aspect_ratio,
                    unit_scale,
                    &tmp_alloc,
                    &mut |hash| {
                        self.custom_pipelines
                            .get(&CompactString::new(hash))
                            .map(|v| v.pipeline)
                    }
                )?;
            }
        } else if let Some(id) = self.window_passes.get_mut(&pass_id) {
            let tmp_alloc = ArenaGuard::new(&self.tmp_alloc);
            self.windows.get_mut(id).unwrap().render_work(
                commands,
                &mut self.style,
                sampler,
                pass_id,
                base_pipeline,
                text_pipeline,
                texture_pipeline,
                texture_pipeline_layout,
                vertex_buffer,
                index_buffer,
                inv_aspect_ratio,
                unit_scale,
                &tmp_alloc,
                &mut |hash| {
                    self.custom_pipelines
                        .get(&CompactString::new(hash))
                        .map(|v| v.pipeline)
                }
            )?;
        }
        Ok(())
    }

    fn init_buffers(&mut self, gpu: &mut gpu::GpuContext) -> Result<()> {
        let buffered_frames = gpu.buffered_frames();
        let vertex_buffer = gpu.create_buffer(
            self.ring_buffer_size as u64,
            &[gpu::BufferUsage::VertexBuffer],
            gpu::ResourceBinderBuffer::DefaultBinderMappable,
        )?;
        let vertex_buffer_map = unsafe {
            gpu.map_buffer(vertex_buffer)?
        };
        self.vertex_buffer = Some(RingBuf::new(
            vertex_buffer,
            vertex_buffer_map,
            buffered_frames,
            self.ring_buffer_size,
        )?);
        let index_buffer = gpu.create_buffer(
            self.ring_buffer_size as u64,
            &[gpu::BufferUsage::IndexBuffer],
            gpu::ResourceBinderBuffer::DefaultBinderMappable,
        )?;
        let index_buffer_map = unsafe {
            gpu.map_buffer(index_buffer)?
        };
        self.index_buffer = Some(RingBuf::new(
            index_buffer,
            index_buffer_map, 
            buffered_frames,
            self.ring_buffer_size,
        )?);
        self.device_alloc = Some(
            gpu.create_default_linear_device_alloc_mappable(self.device_alloc_block_size)?
        );
        Ok(())
    }

    pub unsafe fn clean_up(&mut self, gpu: &mut gpu::GpuContext) -> Result<()> {
        if let Some(buf) = self.vertex_buffer.take() {
            gpu.destroy_buffer(buf.id());
        };
        if let Some(buf) = self.index_buffer.take() {
            gpu.destroy_buffer(buf.id());
        }
        self.base_pipelines.clean_up(gpu)?;
        for pipeline in &self.custom_pipelines {
            pipeline.1.clean_up(gpu);
        }
        self.custom_pipelines.clear();
        Ok(())
    }
}
