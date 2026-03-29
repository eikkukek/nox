use ahash::AHashMap;

use compact_str::format_compact;

use nox::{
    error::*,
    mem::{
        vec::Vec32,
        collections::EntryExt,
        option::OptionExt,
    },
    sync::*,
    *,
};

use gpu::{
    VertexInput,
    Commands,
};

use nox_font::{text_segment};

use nox_geom::*;

use crate::*;

pub(crate) const COLOR_PICKER_PIPELINE_HASH: ConstName = ConstName::new("nox_gui color picker");
pub(crate) const COLOR_PICKER_HUE_PIPELINE_HASH: ConstName = ConstName::new("nox_gui color picker hue");
pub(crate) const COLOR_PICKER_ALPHA_PIPELINE_HASH: ConstName = ConstName::new("nox_gui color picker alpha");

#[derive(Default)]
struct BasePipelines {
    pub base_pipeline: gpu::GraphicsPipelineId,
    pub text_pipeline: gpu::GraphicsPipelineId,
    pub texture_pipeline: gpu::GraphicsPipelineId,
}

impl BasePipelines {

    fn clean_up(&mut self, gpu: &gpu::Gpu) {
        gpu.destroy_pipelines(
            self.base_pipeline.batch_id(),
            [self.base_pipeline, self.text_pipeline, self.texture_pipeline],
            []
        ).ok();
    }
}

pub struct CustomPipelineInfo {
    pub name: ConstName,
    pub shader_set_id: gpu::ShaderSetId,
    pub vertex_inputs: Vec32<(gpu::VertexInputBinding, Vec32<gpu::VertexInputAttribute>)>,
}

impl CustomPipelineInfo {

    pub fn new(
        name: ConstName,
        shader_set_id: gpu::ShaderSetId,
        vertex_inputs: impl IntoIterator<
            Item = (gpu::VertexInputBinding, impl IntoIterator<Item = gpu::VertexInputAttribute>)
        >,
    ) -> Self
    {
        Self {
            name,
            shader_set_id,
            vertex_inputs: vertex_inputs
                .into_iter()
                .map(|(binding, attr)| {
                    (binding, attr.into_iter().collect())
                }).collect(),
        }
    }
}

struct CustomPipeline {
    shader_set_id: gpu::ShaderSetId,
    vertex_inputs: Vec32<(gpu::VertexInputBinding, Vec32<gpu::VertexInputAttribute>)>,
    pipelines: AHashMap<(gpu::Format, gpu::MsaaSamples), gpu::GraphicsPipelineId>,
}

impl CustomPipeline {

    fn clean_up(&self, gpu: &gpu::Gpu) {
        gpu.delete_shader_set(self.shader_set_id);
        for (_, &pipeline) in &self.pipelines {
            gpu.destroy_pipelines(
                pipeline.batch_id(),
                [pipeline],
                []
            ).ok();
        }
    }
}

pub struct Inner {
    gpu: gpu::Gpu,
    text_renderer: TextRenderer,
    sampler: gpu::Sampler,
    style: UiStyle,
    windows: AHashMap<u32, Window>,
    active_windows: Vec32<u32>,
    vertex_buffer: RingBuf,
    index_buffer: RingBuf,
    image_loader: ImageLoader,
    pipeline_cache: Option<gpu::PipelineCache>,
    frame: u64,
    command_dependencies: Vec32<gpu::CommandDependency>,
    base_pipelines: AHashMap<(gpu::Format, gpu::MsaaSamples), BasePipelines>,
    custom_pipelines: AHashMap<ConstName, CustomPipeline>,
    render_format: gpu::Format,
    render_samples: gpu::MsaaSamples,
    base_shader_set: gpu::ShaderSetId,
    text_shader_set: gpu::ShaderSetId,
    texture_shader_set: gpu::ShaderSetId,
    prev_cursor_position: Vec2,
    flags: u32,
    min_sample_shading: f32,
    set_cursor: Option<win::CursorIcon>,
}

impl Inner {

    fn recreate_pipelines(
        &mut self,
        render_format: gpu::Format,
        render_samples: gpu::MsaaSamples,
    ) -> Result<()>
    {
        let mut batch = self.gpu.create_pipeline_batch(self.pipeline_cache.clone())?;
        let min_sample_shading = self.min_sample_shading;
        self.base_pipelines
            .entry((render_format, render_samples))
            .or_try_insert_with(|| {
                let (
                    mut base_pipeline,
                    mut text_pipeline,
                    mut texture_pipeline
                ) = Default::default();
                batch
                    .with_graphics_pipelines([
                        gpu::GraphicsPipelineCreateInfo
                            ::new(&mut base_pipeline, self.base_shader_set)
                            .with_vertex_input(
                                gpu::VertexInputBinding::new::<Vertex>(
                                    0, gpu::VertexInputRate::Vertex
                                ),
                                &mut Vertex::get_attributes(0),
                            )?.with_color_output(
                                render_format,
                                gpu::ColorComponents::RGBA,
                                Some(Workspace::BLEND_STATE)
                            ).with_sample_shading(gpu::SampleShadingInfo
                                ::default()
                                .samples(render_samples)
                                .min_shading(min_sample_shading)
                            ),
                        gpu::GraphicsPipelineCreateInfo
                            ::new(&mut text_pipeline, self.text_shader_set)
                            .with_vertex_input(
                                gpu::VertexInputBinding::new::<font::Vertex>(
                                    0, gpu::VertexInputRate::Vertex
                                ),
                                &mut font::Vertex::get_attributes(0),
                            )?.with_vertex_input(
                                gpu::VertexInputBinding::new::<font::VertexOffset>(
                                    1, gpu::VertexInputRate::Instance
                                ),
                                &mut font::VertexOffset::get_attributes(1)
                            )?.with_vertex_input(
                                gpu::VertexInputBinding::new::<BoundedTextInstance>(
                                    2, gpu::VertexInputRate::Instance
                                ),
                                &mut BoundedTextInstance::get_attributes(2)
                            )?.with_color_output(
                                render_format,
                                gpu::ColorComponents::RGBA,
                                Some(Workspace::BLEND_STATE)
                            ).with_sample_shading(gpu::SampleShadingInfo
                                ::default()
                                .samples(render_samples)
                                .min_shading(min_sample_shading)
                            ),
                        gpu::GraphicsPipelineCreateInfo
                            ::new(&mut texture_pipeline, self.texture_shader_set)
                            .with_color_output(
                                render_format,
                                gpu::ColorComponents::RGBA,
                                Some(Workspace::BLEND_STATE)
                            ).with_sample_shading(gpu::SampleShadingInfo
                                ::default()
                                .samples(render_samples)
                                .min_shading(min_sample_shading)
                            )
                    ]);
                Result::Ok(BasePipelines { base_pipeline, text_pipeline, texture_pipeline })
            })?;
        for pipeline in self.custom_pipelines.values_mut() {
            pipeline.pipelines.entry((render_format, render_samples))
                .or_try_insert_with(|| {
                    let mut id = Default::default();
                    let mut info = gpu::GraphicsPipelineCreateInfo
                            ::new(&mut id, pipeline.shader_set_id)
                            .with_color_output(
                                render_format,
                                gpu::ColorComponents::RGBA,
                                Some(Workspace::BLEND_STATE))
                            .with_sample_shading(gpu::SampleShadingInfo
                                ::default()
                                .samples(render_samples)
                                .min_shading(min_sample_shading)
                            );
                    for (binding, attributes) in pipeline.vertex_inputs.iter_mut() {
                        info = info.with_vertex_input(
                            *binding, attributes,
                        )?;
                    }
                    batch.with_graphics_pipelines([info]);
                    Ok(id)
                })?;
        }
        if batch.is_empty() {
            batch.discard();
        } else {
            let _ = batch.build()?;
        }
        Ok(())
    }

    fn add_custom_pipelines(
        &mut self,
        infos: impl IntoIterator<Item = CustomPipelineInfo>,
    ) -> Result<()>
    {
        for info in infos.into_iter() {
            if self.custom_pipelines.contains_key(&info.name) {
                return Err(Error::just_context(format_compact!(
                    "custom pipeline with name {} already exists",
                    info.name,
                )))
            }
            self.custom_pipelines.insert(
                info.name,
                CustomPipeline {
                    shader_set_id: info.shader_set_id,
                    vertex_inputs: info.vertex_inputs.clone(),
                    pipelines: AHashMap::default(),
                }
            );
        }
        if self.render_format != gpu::Format::Undefined {
            self.recreate_pipelines(self.render_format, self.render_samples)?;
        }
        Ok(())
    }

    #[inline]
    fn get_custom_pipeline(&self, key: ConstName) -> Option<gpu::GraphicsPipelineId> {
        self.custom_pipelines
            .get(&key)
            .and_then(|p| p.pipelines
                .get(&(self.render_format, self.render_samples))
                .copied()
            )
    }

    #[inline]
    fn began(&self) -> bool {
        self.flags & Workspace::BEGAN == Workspace::BEGAN
    }

    #[inline]
    fn cursor_in_window(&self) -> bool {
        self.flags & Workspace::CURSOR_IN_WINDOW == Workspace::CURSOR_IN_WINDOW
    }
}

#[derive(Clone)]
pub struct Workspace {
    inner: Arc<RwLock<Inner>>,
}

pub struct Attributes {
    fonts: Vec32<(ConstName, font::OwnedFace)>,
    style: UiStyle,
    font_curve_tolerance: f32,
    staging_alloc_block_size: gpu::DeviceSize,
}

impl Attributes {

    #[inline]
    pub fn with_style_attribute<F, T>(
        mut self,
        f: F,
        value: T
    ) -> Self
        where F: FnOnce(&mut UiStyle, T) -> &mut UiStyle
    {
        f(&mut self.style, value);
        self
    }

    #[inline]
    pub fn with_font_curve_tolerance(mut self, tolerance: f32) -> Self {
        self.font_curve_tolerance = tolerance;
        self
    }

    #[inline]
    pub fn with_staging_alloc_block_size(mut self, size: gpu::DeviceSize) -> Self {
        self.staging_alloc_block_size = size;
        self
    }
}

pub fn default_attribures(
    fonts: impl IntoIterator<Item = (&'static str, font::OwnedFace)>,
    regular_font: &'static str,
) -> Attributes {
    Attributes {
        fonts: fonts
            .into_iter()
            .map(|(name, face)| (ConstName::new(name), face))
            .collect(),
        style: UiStyle {
            regular_font: ConstName::new(regular_font),
            ..Default::default()
        },
        font_curve_tolerance: 0.02,
        staging_alloc_block_size: 1 << 25,
    }
}

impl Workspace
{

    const BEGAN: u32 = 0x1;
    const CURSOR_IN_WINDOW: u32 = 0x2;

    const BLEND_STATE: gpu::ColorOutputBlendState = gpu::ColorOutputBlendState {
        src_color_blend_factor: gpu::BlendFactor::SrcAlpha,
        dst_color_blend_factor: gpu::BlendFactor::OneMinusSrcAlpha,
        color_blend_op: gpu::BlendOp::Add,
        src_alpha_blend_factor: gpu::BlendFactor::One,
        dst_alpha_blend_factor: gpu::BlendFactor::OneMinusSrcAlpha,
        alpha_blend_op: gpu::BlendOp::Add,
    };

    pub fn new(
        event_loop: &event_loop::ActiveEventLoop<'_>,
        sampler: Option<gpu::Sampler>,
        pipeline_cache: Option<gpu::PipelineCache>,
        attributes: Attributes,
    ) -> Result<Self>
    {
        let mut text_renderer = TextRenderer::new(
            attributes.fonts,
            attributes.font_curve_tolerance,
        );
        let style = attributes.style;
        text_renderer.render(&[text_segment("0123456789", &style.regular_font)], false, 0.0);
        let ring_buffer_size = 1 << 23;
        let gpu = event_loop.gpu();
        if !gpu.device().get_device_attribute(gpu::ext::push_descriptor::Attributes::IS_ENABLED)
            .bool().unwrap_or(false)
        {
            return Err(Error::just_context(format_compact!(
                "push descriptor device extension is not enabled"
            )))
        }
        let ring_memory_bnder = gpu::GlobalBinder::new(
            gpu.device().clone(),
            gpu::MemoryProperties::HOST_VISIBLE,
            gpu::MemoryProperties::HOST_VISIBLE | gpu::MemoryProperties::HOST_COHERENT
        );
        let base_shader_set = gpu.create_shader_set(
            [
                gpu::Shader::new(
                    gpu,
                    gpu::default_shader_attributes()
                        .with_name("base vertex shader")
                        .with_stage(gpu::ShaderStage::Vertex)
                        .with_glsl(BASE_VERTEX_SHADER)
                )?,
                gpu::Shader::new(
                    gpu,
                    gpu::default_shader_attributes()
                        .with_name("base fragment shader")
                        .with_stage(gpu::ShaderStage::Fragment)
                        .with_glsl(BASE_FRAGMENT_SHADER)
                )?,
            ],
            gpu::default_shader_set_attributes(),
        ).context("failed to create base shader set")?;
        let text_shader_set = gpu.create_shader_set(
            [
                gpu::Shader::new(
                    gpu,
                    gpu::default_shader_attributes()
                        .with_name("text vertex shader")
                        .with_stage(gpu::ShaderStage::Vertex)
                        .with_glsl(TEXT_VERTEX_SHADER)
                )?,
                gpu::Shader::new(
                    gpu,
                    gpu::default_shader_attributes()
                        .with_name("text fragment shader")
                        .with_stage(gpu::ShaderStage::Fragment)
                        .with_glsl(TEXT_FRAGMENT_SHADER)
                )?,
            ],
            gpu::default_shader_set_attributes(),
        ).context("failed to create text shader set")?;
        let texture_shader_set = gpu.create_shader_set(
            [
                gpu::Shader::new(
                    gpu,
                    gpu::default_shader_attributes()
                        .with_name("texture vertex shader")
                        .with_stage(gpu::ShaderStage::Vertex)
                        .with_glsl(TEXTURE_VERTEX_SHADER)
                )?,
                gpu::Shader::new(
                    gpu,
                    gpu::default_shader_attributes()
                        .with_name("texture fragment shader")
                        .with_stage(gpu::ShaderStage::Fragment)
                        .with_glsl(TEXTURE_FRAGMENT_SHADER)
                )?,
            ],
            gpu::default_shader_set_attributes()
                .with_descriptor_set_layout_flags(0, gpu::DescriptorSetLayoutFlags::PUSH_DESCRIPTOR)
        ).context("failed to create texture shader set")?;
        let mut s = Self { inner: Arc::new(RwLock::new(Inner {
            gpu: gpu.clone(),
            sampler: sampler.unwrap_or_try_else(|| {
                gpu::SamplerCreateInfo
                    ::default()
                    .min_filter(gpu::Filter::Linear)
                    .mag_filter(gpu::Filter::Linear)
                    .max_lod(gpu::Sampler::LOD_CLAMP_NONE)
                    .address_mode(
                        gpu::SamplerAddressMode::ClampToBorder,
                        gpu::SamplerAddressMode::ClampToBorder,
                        gpu::SamplerAddressMode::ClampToBorder,
                    ).build(gpu.device().clone())
            })?,
            text_renderer,
            style,
            windows: Default::default(),
            active_windows: Default::default(),
            vertex_buffer: RingBuf::new(
                gpu.clone(),
                &ring_memory_bnder,
                ring_buffer_size,
                gpu::BufferUsages::VERTEX_BUFFER,
            )?,
            index_buffer: RingBuf::new(
                gpu.clone(),
                &ring_memory_bnder,
                ring_buffer_size,
                gpu::BufferUsages::INDEX_BUFFER,
            )?,
            base_shader_set,
            text_shader_set,
            texture_shader_set,
            image_loader: ImageLoader::new(gpu.clone())?,
            command_dependencies: Vec32::new(),
            pipeline_cache,
            base_pipelines: AHashMap::default(),
            custom_pipelines: AHashMap::default(),
            render_format: gpu::Format::Undefined,
            render_samples: gpu::MsaaSamples::X1,
            frame: 0,
            prev_cursor_position: Default::default(),
            flags: 0,
            min_sample_shading: 0.2,
            set_cursor: None,
        }))};
        let color_picker_vertex_shader = gpu::Shader::new(
            gpu,
            gpu::default_shader_attributes()
                .with_name("color picker vertex shader")
                .with_stage(gpu::ShaderStage::Vertex)
                .with_glsl(COLOR_PICKER_VERTEX_SHADER)
        )?;
        s.add_custom_pipelines(
            [
                CustomPipelineInfo::new(
                    COLOR_PICKER_PIPELINE_HASH,
                    gpu.create_shader_set(
                        [
                            color_picker_vertex_shader.clone(),
                            gpu::Shader::new(
                                gpu,
                                gpu::default_shader_attributes()
                                    .with_name("color picker fragment shader")
                                    .with_stage(gpu::ShaderStage::Fragment)
                                    .with_glsl(COLOR_PICKER_FRAGMENT_SHADER)
                            )?,
                        ],
                        gpu::default_shader_set_attributes(),
                    ).context("failed to create color picker shader set")?,
                    [(
                        gpu::VertexInputBinding::new::<Vertex>(0, gpu::VertexInputRate::Vertex),
                        Vertex::get_attributes(0).into_iter(),
                    )]
                ),
                CustomPipelineInfo::new(
                    COLOR_PICKER_HUE_PIPELINE_HASH,
                    gpu.create_shader_set(
                        [
                            color_picker_vertex_shader.clone(),
                            gpu::Shader::new(
                                gpu,
                                gpu::default_shader_attributes()
                                    .with_name("color picker hue fragment shader")
                                    .with_stage(gpu::ShaderStage::Fragment)
                                    .with_glsl(COLOR_PICKER_FRAGMENT_SHADER_HUE)
                            )?
                        ],
                        gpu::default_shader_set_attributes(),
                    ).context("failed to create color picker hue shader set")?,
                    [(
                        gpu::VertexInputBinding::new::<Vertex>(0, gpu::VertexInputRate::Vertex),
                        Vertex::get_attributes(0).into_iter(),
                    )]
                ),
                CustomPipelineInfo::new(
                    COLOR_PICKER_ALPHA_PIPELINE_HASH,
                    gpu.create_shader_set(
                        [
                            color_picker_vertex_shader.clone(),
                            gpu::Shader::new(
                                gpu,
                                gpu::default_shader_attributes()
                                    .with_name("color picker alpha fragment shader")
                                    .with_stage(gpu::ShaderStage::Fragment)
                                    .with_glsl(COLOR_PICKER_FRAGMENT_SHADER_ALPHA)
                            )?,
                        ],
                        gpu::default_shader_set_attributes()
                    ).context("failed to create color picker alpha shader set")?,
                    [(
                        gpu::VertexInputBinding::new::<Vertex>(0, gpu::VertexInputRate::Vertex),
                        Vertex::get_attributes(0).into_iter(),
                    )]
                )
            ]
        )?;
        Ok(s)
    }

    pub fn style_attribute<F, T>(
        &mut self,
        f: F,
        value: T
    )
        where F: FnOnce(&mut UiStyle, T) -> &mut UiStyle
    {
        let mut inner = self.inner.write();
        f(&mut inner.style, value);
    }

    pub fn reallocate_frame_resources(
        &mut self,
        image_count: u32,
        render_format: gpu::Format,
        render_samples: gpu::MsaaSamples,
    ) -> Result<()> {
        let mut inner = self.inner.write();
        inner.vertex_buffer.allocate_frame_regions(image_count);
        inner.index_buffer.allocate_frame_regions(image_count);
        inner.render_format = render_format;
        inner.render_samples = render_samples;
        inner.recreate_pipelines(render_format, render_samples)
    } 

    pub fn add_custom_pipelines(
        &mut self,
        infos: impl IntoIterator<Item = CustomPipelineInfo>,
    ) -> Result<()>
    {
        self.inner.write().add_custom_pipelines(infos)
    } 

    #[inline(always)]
    pub fn get_custom_pipeline(&self, key: ConstName) -> Option<gpu::GraphicsPipelineId> {
        self.inner.write().get_custom_pipeline(key)
    } 

    #[inline(always)]
    pub fn begin<'a>(
        &mut self,
        mut win: win::WindowContext<'a>,
    ) -> Result<RecordUi<'_, 'a>>
    {
        let mut inner = self.inner.write();
        if inner.began() {
            return Err(Error::just_context("end not called"));
        }
        let unit_scale = inner.style.pixels_per_unit / win.size_f32().1;
        inner.frame += 1;
        inner.flags |= Self::BEGAN;
        if let Some(cursor) = inner.set_cursor.take() {
            win.set_cursor(cursor);
        }
        inner.command_dependencies.clear();
        Ok(RecordUi { inner, win, ws: self.clone(), unit_scale, })
    } 
}

#[derive(Clone, Copy)]
pub struct CachedUiData {
    pub cursor_pos: Vec2,
    pub delta_cursor_pos: Vec2,
    pub window_size: Vec2,
    pub aspect_ratio: f32,
    pub inv_aspect_ratio: f32,
    pub cursor_in_other_window: bool,
    pub unit_scale: f32,
    pub mouse_delta_lines: (f32, f32),
    pub mouse_delta_pixels: (f64, f64),
    pub mouse_button_left_state: win::InputState,
    pub delta_time: f32,
}

impl CachedUiData {
    
    #[inline]
    pub fn viewport_and_scissor(&self) -> (gpu::Viewport, gpu::Scissor) {
        (
            gpu::Viewport
                ::default()
                .width(self.window_size.x)
                .height(self.window_size.y),
            gpu::Scissor::default()
                .width(self.window_size.x as u32)
                .height(self.window_size.y as u32)
        )
    }
}

pub struct RecordUi<'a, 'b> {
    inner: RwLockWriteGuard<'a, Inner>,
    win: win::WindowContext<'b>,
    unit_scale: f32,
    ws: Workspace,
}

impl<'a, 'b> RecordUi<'a, 'b> {

    pub fn window<F>(
        &mut self,
        id: u32,
        title: &str,
        initial_position: [f32; 2],
        initial_size: [f32; 2],
        mut f: F,
    ) -> Result<&mut Self>
        where
            F: FnMut(&mut UiContext<Window>),
    {
        let inner = &mut * self.inner;
        if !inner.began() {
            return Err(Error::just_context("begin not called"));
        }
        let window = inner.windows.entry(id).or_insert_with(|| Window::new(
            title,
            initial_position,
            initial_size,
        ));
        window.set_last_frame(inner.frame);
        window.begin();
        if title != window.title {
            window.title = title.into();
            window.title_text = None;
        }
        let title_text = window.title_text.get_or_insert_with(|| inner.text_renderer.render(
            &[text_segment(window.title.as_str(), &inner.style.regular_font)],
            false,
            0.0,
        ).unwrap_or_default());
        let start_off = vec2(
            inner.style.item_pad_outer.x,
            inner.style.calc_text_box_height_from_text_height(
                title_text.row_height * inner.style.font_scale * inner.style.title_add_scale) +
                inner.style.item_pad_outer.y,
        );
        f(&mut UiContext::new(
            &mut self.win,
            window,
            &inner.style,
            start_off,
            &mut inner.text_renderer,
            &mut inner.image_loader,
            &mut inner.command_dependencies,
        ));
        if !inner.active_windows.contains(&id) {
            inner.active_windows.push(id);
        }
        Ok(self)
    }

    pub fn end<F>(
        mut self,
        queue: gpu::DeviceQueue,
        get_render_attachment: F,
    ) -> Result<()>
        where
            F: for<'c, 'd> FnOnce(&mut gpu::GraphicsCommands<'c, 'd>) -> EventResult<gpu::PassAttachment>
                + Send + Sync + 'static,
    {
        let inner = &mut *self.inner;
        if !inner.began() {
            return Err(Error::just_context("begin not called"))
        }
        inner.active_windows.retain(|id| {
            let win = inner.windows.get(id).unwrap();
            win.last_frame() == inner.frame
        });
        let aspect_ratio = self.win.aspect_ratio() as f32;
        let inv_aspect_ratio = 1.0 / aspect_ratio;
        let window_size: Vec2 = self.win.size_f32().into();
        let unit_scale = self.unit_scale;
        let mut cursor_pos: Vec2 = self.win.normalized_cursor_position_f32().into();
        cursor_pos *= 2.0;
        cursor_pos -= vec2(1.0, 1.0);
        cursor_pos.x *= aspect_ratio;
        cursor_pos /= unit_scale;
        let delta_cursor_pos = cursor_pos - inner.prev_cursor_position;
        inner.prev_cursor_position = cursor_pos;
        let mouse_delta_lines = self.win.mouse_scroll_delta_lines();
        let mouse_delta_pixels = self.win.mouse_scroll_delta_pixels();
        let mouse_button_left_state = self.win.mouse_button_state(win::MouseButton::Left);
        let delta_time = self.win.delta_time_secs_f32();
        let ws = self.ws.clone();
        let mut scheduler = inner.gpu.schedule_commands();
        scheduler.new_commands::<gpu::NewGraphicsCommands>(
            queue,
            move |cmd| {
                let inner = ws.inner.write();
                let mut rec = RecordCmd { inner, };
                let mut cursor_in_some_window = false;
                let mut window_pressed = None;
                let active_windows: *mut Vec32<_> = &mut rec.inner.active_windows;
                for (i, id) in unsafe { &*active_windows }.iter().enumerate().rev() {
                    let window: *mut Window = rec.inner.windows.get_mut(id).unwrap();
                    let cached_data = CachedUiData {
                        cursor_pos,
                        delta_cursor_pos,
                        window_size,
                        aspect_ratio,
                        inv_aspect_ratio,
                        cursor_in_other_window: cursor_in_some_window,
                        unit_scale,
                        mouse_delta_lines,
                        mouse_delta_pixels,
                        mouse_button_left_state,
                        delta_time,
                    };
                    let style: *const UiStyle = &rec.inner.style;
                    let window = unsafe { &mut *window };
                    let cursor_in_window = window.end(&mut rec, unsafe { &*style }, cached_data)
                        .context("failed to end window")?.cursor_in_window;
                    if cursor_in_window && mouse_button_left_state.pressed() {
                        window_pressed = Some(i);
                    }
                    cursor_in_some_window |= cursor_in_window;
                    window.triangulate();
                    window.refresh_position(aspect_ratio, unit_scale, window_size);
                }
                let (semaphore_id, value) = rec.inner.vertex_buffer
                    .finish_frame()
                    .context("failed to flush vertex buffer")?;
                cmd.add_signal_semaphore(semaphore_id, value);
                let (semaphore_id, value) = rec.inner.index_buffer
                    .finish_frame()
                    .context("failed to flush index buffer")?;
                cmd.add_signal_semaphore(semaphore_id, value);
                if let Some(idx) = window_pressed {
                    let id = rec.inner.active_windows.remove(idx as u32);
                    rec.inner.active_windows.push(id);
                }
                if rec.inner.cursor_in_window() &&
                    !cursor_in_some_window && rec.inner.style.override_cursor
                {
                    rec.set_cursor(win::CursorIcon::Default);
                }
                rec.inner.flags &= !(Workspace::CURSOR_IN_WINDOW | Workspace::BEGAN);
                or_flag!(rec.inner.flags, Workspace::CURSOR_IN_WINDOW, cursor_in_some_window);
                let cached_data = CachedUiData {
                    cursor_pos,
                    delta_cursor_pos,
                    window_size,
                    aspect_ratio,
                    inv_aspect_ratio,
                    cursor_in_other_window: cursor_in_some_window,
                    unit_scale,
                    mouse_delta_lines,
                    mouse_delta_pixels,
                    mouse_button_left_state,
                    delta_time,
                };
                let attachment = get_render_attachment(cmd)?;
                cmd.render(
                    gpu::RenderingInfo
                        ::default()
                        .msaa_samples(rec.inner.render_samples),
                    &[attachment],
                    &Default::default(),
                    |pass| {
                        pass.dynamic_draw(|cmd| {
                            let active_windows: *const Vec32<_> = &mut rec.inner.active_windows;
                            let sampler = rec.inner.sampler.clone();
                            for id in unsafe { &*active_windows }.iter() {
                                let window: *mut Window = rec.inner.windows.get_mut(id).unwrap();
                                let style: *const UiStyle = &rec.inner.style;
                                unsafe { &mut* window }.draw(
                                    cmd,
                                    &mut rec,
                                    cached_data,
                                    unsafe { &*style },
                                    sampler.clone(),
                                )?;
                            }
                            Ok(())
                        })?;
                        Ok(())
                    },
                ).context("gui failed to render")?;
                Ok(())
            }
        )?.with_dependencies(inner.command_dependencies.iter().copied());
        Ok(())
    }
}

pub struct RecordCmd<'a> {
    inner: RwLockWriteGuard<'a, Inner>,
}

impl RecordCmd<'_> {
   
    #[inline]
    pub fn text_renderer(&mut self) -> &mut TextRenderer {
        &mut self.inner.text_renderer
    }

    #[inline]
    pub fn set_cursor(&mut self, cursor: win::CursorIcon) {
        self.inner.set_cursor = Some(cursor);
    } 

    #[inline]
    pub fn allocate_indices<T>(&mut self, count: u32) -> Result<RingBufMem<T>> {
        self.inner.index_buffer.allocate(count as usize)
    }

    #[inline]
    pub fn allocate_vertices<T>(&mut self, count: u32) -> Result<RingBufMem<T>> {
        self.inner.vertex_buffer.allocate(count as usize)
    }

    #[inline]
    pub fn base_pipeline(&self) -> gpu::GraphicsPipelineId {
        let inner = &*self.inner;
        inner.base_pipelines
            .get(&(inner.render_format, inner.render_samples))
            .unwrap().base_pipeline
    }

    #[inline]
    pub fn text_pipeline(&self) -> gpu::GraphicsPipelineId {
        let inner = &*self.inner;
        inner.base_pipelines
            .get(&(inner.render_format, inner.render_samples))
            .unwrap().text_pipeline
    }

    #[inline]
    pub fn texture_pipeline(&self) -> gpu::GraphicsPipelineId {
        let inner = &*self.inner;
        inner.base_pipelines
            .get(&(inner.render_format, inner.render_samples))
            .unwrap().texture_pipeline
    }


    #[inline]
    pub fn get_custom_pipeline(&self, name: ConstName) -> Option<gpu::GraphicsPipelineId> {
        let inner = &*self.inner;
        inner.get_custom_pipeline(name)
    }

    #[inline]
    pub fn index_buffer_id(&self) -> gpu::BufferId {
        self.inner.index_buffer.id()
    }

    #[inline]
    pub fn vertex_buffer_id(&self) -> gpu::BufferId {
        self.inner.vertex_buffer.id()
    }
}

impl Drop for Inner
{
    fn drop(&mut self) {
        for pipelines in self.base_pipelines.values_mut() {
            pipelines.clean_up(&self.gpu);
        }
        for pipeline in self.custom_pipelines.values_mut() {
            pipeline.clean_up(&self.gpu);
        }
    }
}
