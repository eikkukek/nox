use nox::{
    event_loop::ActiveEventLoop,
    gpu::{self, MemoryBinder},
    mem::collections::{EntryExt, HashMap},
    sync::{Arc, SwapLock, atomic::{self, AtomicU64}},
};

use nox_font::*;

#[derive(Default)]
struct FrameBufferState {
    pipelines: SwapLock<HashMap<gpu::Format, gpu::GraphicsPipelineId>>,
    extent: AtomicU64,
    images: SwapLock<[gpu::ImageViewId; 3]>,
}

pub struct App {
    window: nox::win::WindowId,
    fb_state: Arc<FrameBufferState>,
    vertex_buffers: Arc<[gpu::BufferId]>,
    vertex_offset_buffers: Arc<[gpu::BufferId]>,
    index_buffers: Arc<[gpu::BufferId]>,
    shader_set: gpu::ShaderSetId,
    rendered_text: Arc<RenderedText>,
    semaphore: gpu::TimelineSemaphoreId,
    semaphore_value: u64,
    image_alloc: gpu::LinearBinder,
    frame_semaphore: gpu::TimelineSemaphoreId,
    frame: u64,
}

impl App {

    pub fn new(
        event_loop: &ActiveEventLoop<'_>,
    ) -> nox::EventResult<Self> {
        let regular: Box<[u8]> =
            include_bytes!("../adobe-garamond/AGaramondPro-Regular.otf")
            .iter().copied().collect();
        let regular = parse_owned_face(regular, 0).unwrap();
        let italic: Box<[u8]> =
            include_bytes!("../adobe-garamond/AGaramondPro-Italic.otf")
            .iter().copied().collect();
        let italic = parse_owned_face(italic, 0).unwrap();
        let bold: Box<[u8]> = include_bytes!("../adobe-garamond/AGaramondPro-Bold.otf")
            .iter().copied().collect();
        let bold = parse_owned_face(bold, 0).unwrap();
        let mut text = VertexTextRenderer::new([
            ("regular", regular),
            ("italic", italic),
            ("bold", bold)
        ], 0.02);
        let rendered_text = Arc::new(text.render(
            &[
                text_segment("To AV moi @ 2 gå ", &"italic"),
                text_segment("this is bold ", &"bold"),
                text_segment("this is regular", &"regular"),
            ],
            true,
            5.0
        ).unwrap());
        let vertex_shader = gpu::Shader::new(
            event_loop.gpu(),
            gpu::default_shader_attributes()
                .with_glsl("#version 450

                layout(location = 0) in vec2 in_pos;

                layout(location = 1) in vec2 in_offset;

                layout(push_constant) uniform PushConstant {
                    float text_width;
                    float font_height;
                    uint text_rows;
                    float aspect_ratio;
                } pc;

                void main() {
                    vec2 pos =
                        in_pos +
                        vec2(in_offset.x, in_offset.y) -
                        vec2(1.0, 0.5) -
                        vec2(pc.text_width / 2.0, pc.text_rows * pc.font_height / 2.0);
                    pos.y *= pc.aspect_ratio * pc.font_height;
                    pos /= 10.0;
                    gl_Position = vec4(pos, 0.0, 1.0);
                }").with_stage(gpu::ShaderStage::Vertex)
                .with_name("vertex shader")
        )?;
        let fragment_shader = gpu::Shader::new(
            event_loop.gpu(),
            gpu::default_shader_attributes()
                .with_glsl("#version 450

                layout(location = 0) out vec4 out_color;

                void main() {
                    float alpha = 1.0;
                    out_color = vec4(1.0, 1.0, 1.0, alpha);
                }
                ").with_stage(gpu::ShaderStage::Fragment)
                .with_name("fragment shader")
        )?;
        let shader_set = event_loop.gpu().create_shader_set(
            [vertex_shader, fragment_shader],
            gpu::default_shader_set_attributes(),
        )?;
        let vertex_buffers: Arc<[gpu::BufferId]> = (0..rendered_text.len())
            .map(|_| Default::default())
            .collect();
        let vertex_offset_buffers: Arc<[gpu::BufferId]> = (0..rendered_text.len())
            .map(|_| Default::default())
            .collect();
        let index_buffers: Arc<[gpu::BufferId]> = (0..rendered_text.len())
            .map(|_| Default::default())
            .collect();
        let mut staging_buffer = Default::default();
        let memory_binder = gpu::GlobalBinder
            ::new(
                event_loop.gpu().device().clone(),
                gpu::MemoryProperties::HOST_VISIBLE | gpu::MemoryProperties::HOST_COHERENT,
                gpu::MemoryProperties::HOST_VISIBLE | gpu::MemoryProperties::HOST_COHERENT,
            );
        let staging_buffer_size = 1 << 24;
        event_loop
            .gpu()
            .create_resources(
                [gpu::BufferCreateInfo::new(
                    &mut staging_buffer,
                    &memory_binder,
                    staging_buffer_size,
                    gpu::BufferUsages::TRANSFER_SRC,
                ).unwrap()],
                []
            )?;
        let map = event_loop.gpu().map_buffer(staging_buffer)?;
        let mut staging_used = 0;
        for (i, (_, text)) in rendered_text.iter().enumerate() {
            let n_vertices = size_of_val(text.trigs.vertices.as_slice());
            assert!(staging_used + n_vertices <= map.size);
            unsafe {
                text.trigs.vertices.as_ptr().cast::<u8>()
                .copy_to_nonoverlapping(map.map.add(staging_used), n_vertices);
            }
            staging_used += n_vertices;
            let n_offsets = size_of_val(text.offsets.as_slice());
            assert!(staging_used + n_offsets <= map.size);
            unsafe {
                text.offsets.as_ptr().cast::<u8>()
                .copy_to_nonoverlapping(map.map.add(staging_used), n_offsets);
            }
            staging_used += n_offsets;
            let n_indices = size_of_val(text.trigs.indices.as_slice());
            assert!(staging_used + n_indices <= map.size);
            unsafe {
                text.trigs.indices.as_ptr().cast::<u8>()
                .copy_to_nonoverlapping(map.map.add(staging_used), n_indices);
            }
            staging_used += n_indices;
            let (mut vertex, mut offset, mut index) = Default::default();
            event_loop
                .gpu()
                .create_resources(
                    [
                        gpu::BufferCreateInfo::new(
                            &mut vertex,
                            &memory_binder,
                            n_vertices as gpu::DeviceSize,
                            gpu::BufferUsages::VERTEX_BUFFER |
                            gpu::BufferUsages::TRANSFER_DST,
                        ).unwrap(),
                        gpu::BufferCreateInfo::new(
                            &mut offset,
                            &memory_binder,
                            n_offsets as gpu::DeviceSize,
                            gpu::BufferUsages::VERTEX_BUFFER |
                            gpu::BufferUsages::TRANSFER_DST,
                        ).unwrap(),
                        gpu::BufferCreateInfo::new(
                            &mut index,
                            &memory_binder,
                            n_indices as gpu::DeviceSize,
                            gpu::BufferUsages::INDEX_BUFFER |
                            gpu::BufferUsages::TRANSFER_DST,
                        ).unwrap()
                    ],
                    []
                )?;
            unsafe {
                vertex_buffers.as_ptr()
                    .add(i)
                    .cast_mut().write(vertex);
                vertex_offset_buffers.as_ptr()
                    .add(i)
                    .cast_mut().write(offset);
                index_buffers.as_ptr()
                    .add(i)
                    .cast_mut().write(index);
            }
        }
        let text = rendered_text.clone();
        let vert = vertex_buffers.clone();
        let off = vertex_offset_buffers.clone();
        let ind = index_buffers.clone();
        let mut semaphore = Default::default();
        event_loop.gpu().create_timeline_semaphores([(&mut semaphore, 0)])?;
        event_loop
            .gpu()
            .schedule_commands()
            .new_commands::<gpu::NewCopyCommands>(
                event_loop.gpu().any_device_queue(gpu::QueueFlags::GRAPHICS).unwrap(),
                move |cmd| {
                    let mut staging_used = 0;
                    for (i, (_, text)) in text.iter().enumerate() {
                        let n_vertices = size_of_val(text.trigs.vertices.as_slice());
                        cmd.copy_buffer(
                            staging_buffer,
                            vert[i],
                            &[gpu::BufferCopy
                                ::default()
                                .src_offset(staging_used as gpu::DeviceSize)
                                .dst_offset(0)
                                .size(n_vertices as gpu::DeviceSize)
                            ],
                            gpu::CommandOrdering::Strict,
                        )?;
                        staging_used += n_vertices;
                        let n_offsets = size_of_val(text.offsets.as_slice());
                        cmd.copy_buffer(
                            staging_buffer,
                            off[i],
                            &[gpu::BufferCopy
                                ::default()
                                .src_offset(staging_used as gpu::DeviceSize)
                                .dst_offset(0)
                                .size(n_offsets as gpu::DeviceSize)
                            ],
                            gpu::CommandOrdering::Strict,
                        )?;
                        staging_used += n_offsets;
                        let n_indices = size_of_val(text.trigs.indices.as_slice());
                        cmd.copy_buffer(
                            staging_buffer,
                            ind[i],
                            &[gpu::BufferCopy
                                ::default()
                                .src_offset(staging_used as gpu::DeviceSize)
                                .dst_offset(0)
                                .size(n_indices as gpu::DeviceSize)
                            ],
                            gpu::CommandOrdering::Strict,
                        )?;
                        staging_used += n_indices;
                    }
                    Ok(())
                }
            )?.with_signal_semaphore(semaphore, 1);
        let mut frame_semaphore = Default::default();
        event_loop.gpu().create_timeline_semaphores([(&mut frame_semaphore, 3)])?;
        Ok(Self {
            window: event_loop.create_window(nox::win
                ::default_attributes()
                .with_resizable(true)
                .with_title("font test")
            )?,
            fb_state: Default::default(),
            vertex_buffers,
            vertex_offset_buffers,
            index_buffers,
            shader_set,
            rendered_text,
            semaphore,
            semaphore_value: 1,
            image_alloc: gpu::LinearBinder::new(
                event_loop.gpu().device().clone(),
                1 << 25,
                gpu::MemoryProperties::DEVICE_LOCAL,
                gpu::MemoryProperties::HOST_VISIBLE
            )?,
            frame_semaphore,
            frame: 3,
        })
    }

    pub fn event(
        &mut self,
        event_loop: &ActiveEventLoop<'_>,
        event: nox::Event,
    ) -> nox::EventResult<()> {
        match event {
            nox::Event::Update => {
                if !event_loop.is_window_valid(self.window) {
                    event_loop.exit();
                    return Ok(())
                }
                let window = self.window;
                let fb_state = self.fb_state.clone();
                let text = self.rendered_text.clone();
                let vert = self.vertex_buffers.clone();
                let off = self.vertex_offset_buffers.clone();
                let ind = self.index_buffers.clone();
                let frame = self.frame;
                event_loop.gpu()
                    .schedule_commands()
                    .new_commands::<gpu::NewGraphicsCommands>(
                        event_loop.gpu().any_device_queue(gpu::QueueFlags::GRAPHICS).unwrap(),
                        move |cmd| {
                            let (swapchain_image, format) = cmd.swapchain_image_view(window.surface_id())?;
                            let extent = fb_state.extent.load(atomic::Ordering::Acquire);
                            let (width, height) = (
                                (extent & 0xFFFFFFFF) as u32,
                                (extent >> 32) as u32,
                            );
                            cmd.render(
                                gpu::RenderingInfo
                                    ::default()
                                    .msaa_samples(gpu::MsaaSamples::X8),
                                &[gpu::PassAttachment::new(
                                    fb_state.images.load()[(frame % 3) as usize]
                                ).with_resolve(gpu::ResolveInfo::new(
                                    swapchain_image, gpu::ResolveMode::Average,
                                )).with_load_op(gpu::AttachmentLoadOp::Clear)],
                                &Default::default(),
                                |pass| {
                                    pass.dynamic_draw(|cmd| {
                                        let mut pipeline_cmd = cmd.bind_pipeline(
                                            *fb_state.pipelines.load().get(&format).unwrap(),
                                            &[gpu::Viewport
                                                ::default()
                                                .width(width as f32)
                                                .height(height as f32)
                                            ],
                                            &[gpu::Scissor
                                                ::default()
                                                .width(width)
                                                .height(height)
                                            ],
                                        )?;
                                        #[repr(C)]
                                        #[derive(Clone, Copy)]
                                        struct PC {
                                            _text_width: f32,
                                            _font_height: f32,
                                            _text_rows: u32,
                                            _aspect_ratio: f32,
                                        }
                                        let pc = PC {
                                            _text_width: text.text_width,
                                            _font_height: text.row_height,
                                            _text_rows: text.text_rows,
                                            _aspect_ratio:
                                                width as f32 /
                                                height as f32,
                                        };
                                        pipeline_cmd.push_constants(0, &[pc])?;
                                        for (i, (_, text)) in text.iter().enumerate() {
                                            pipeline_cmd.begin_drawing_indexed(
                                                gpu::IndexedDrawInfo {
                                                    index_count: text.trigs.indices.len(),
                                                    instance_count: text.offsets.len() as u32,
                                                    ..Default::default()
                                                },
                                                gpu::IndexBufferInfo::new(ind[i], 0),
                                                &[
                                                    gpu::DrawBufferRange::new(
                                                        vert[i], 0, None,
                                                    ),
                                                    gpu::DrawBufferRange::new(
                                                        off[i], 0, None,
                                                    ),
                                                ],
                                                None,
                                                |cmd| Ok(cmd.draw_indexed()?)
                                            )?;
                                        }
                                        Ok(())
                                    })?;
                                    Ok(())
                                },
                            )?;
                            Ok(())
                        },
                    )?.with_wait_semaphore(
                        self.semaphore,
                        self.semaphore_value,
                        gpu::MemoryDependencyHint::NONE,
                    ).with_wait_semaphore(
                        self.frame_semaphore,
                        self.frame - 2,
                        gpu::MemoryDependencyHint::COLOR_OUTPUT,
                    ).with_signal_semaphore(
                        self.frame_semaphore,
                        self.frame + 1
                    );
                    self.frame += 1;
                Ok(())
            },
            nox::Event::GpuEvent(event) => match event {
                gpu::Event::SwapchainCreated { surface_id: _, new_format, new_size, image_count: _ } => {
                    if event_loop.gpu().wait_for_semaphores(&[(self.frame_semaphore, self.frame - 1)], u64::MAX)? 
                    {
                        if self.fb_state.images.load()[0] != gpu::ImageViewId::default() {
                            event_loop.gpu().destroy_resources(
                                [],
                                self.fb_state.images.load().iter().map(|view| {
                                    view.image_id()
                                })
                            )?;
                        }
                        unsafe {
                            self.image_alloc.release_resources();
                        }
                        let (mut a, mut b, mut c) = Default::default();
                        event_loop.gpu().create_resources(
                            [],
                            [
                                gpu::ImageCreateInfo
                                    ::new(&mut a, &self.image_alloc)
                                    .with_format(new_format, false)
                                    .with_dimensions(new_size)
                                    .with_usage(gpu::ImageUsages::COLOR_ATTACHMENT)
                                    .with_samples(gpu::MsaaSamples::X8),
                                gpu::ImageCreateInfo
                                    ::new(&mut b, &self.image_alloc)
                                    .with_format(new_format, false)
                                    .with_dimensions(gpu::Dimensions::new(new_size.0, new_size.1, 1))
                                    .with_usage(gpu::ImageUsages::COLOR_ATTACHMENT)
                                    .with_samples(gpu::MsaaSamples::X8),
                                gpu::ImageCreateInfo
                                    ::new(&mut c, &self.image_alloc)
                                    .with_format(new_format, false)
                                    .with_dimensions(gpu::Dimensions::new(new_size.0, new_size.1, 1))
                                    .with_usage(gpu::ImageUsages::COLOR_ATTACHMENT)
                                    .with_samples(gpu::MsaaSamples::X8),
                            ]
                        )?;
                        self.fb_state.images.modify(|images| {
                            *images = [
                                event_loop.gpu().create_image_view(
                                    a,
                                    gpu::ImageRange::new(gpu::ImageSubresourceRange
                                        ::default()
                                        .aspect_mask(gpu::ImageAspects::COLOR),
                                    None)
                                )?,
                                event_loop.gpu().create_image_view(
                                    b,
                                    gpu::ImageRange::new(gpu::ImageSubresourceRange
                                        ::default()
                                        .aspect_mask(gpu::ImageAspects::COLOR),
                                    None)
                                )?,
                                event_loop.gpu().create_image_view(
                                    c,
                                    gpu::ImageRange::new(gpu::ImageSubresourceRange
                                        ::default()
                                        .aspect_mask(gpu::ImageAspects::COLOR),
                                    None)
                                )?,
                            ];
                            nox::Result::Ok(())
                        })?;
                    }
                    let mut extent = new_size.0 as u64;
                    extent |= (new_size.1 as u64) << 32;
                    self.fb_state.extent.store(extent, atomic::Ordering::Release);
                    if self.fb_state.pipelines.load().get(&new_format).is_none() {
                        self.fb_state.pipelines
                        .modify(|pipelines| {
                            pipelines
                                .entry(new_format)
                                .or_try_insert_with_key(|&format| {
                                    let mut id = Default::default();
                                    let _ = event_loop
                                        .gpu()
                                        .create_pipeline_batch(None)?
                                        .with_graphics_pipelines([
                                            gpu::GraphicsPipelineCreateInfo
                                                ::new(&mut id, self.shader_set)
                                                .with_color_output(
                                                    format,
                                                    gpu::ColorComponents::default(),
                                                    None,
                                                ).with_vertex_input(
                                                    gpu::VertexInputBinding::new::<nox_font::Vertex>(
                                                        0,
                                                        gpu::VertexInputRate::Vertex,
                                                    ),
                                                    &mut nox_font::Vertex::get_attributes(0)
                                                )?.with_vertex_input(
                                                    gpu::VertexInputBinding::new::<nox_font::VertexOffset>(
                                                        1,
                                                        gpu::VertexInputRate::Instance
                                                    ),
                                                    &mut nox_font::VertexOffset::get_attributes(1),
                                                )?.with_sample_shading(gpu::SampleShadingInfo   
                                                    ::default().samples(gpu::MsaaSamples::X8)
                                                ),
                                        ]).build()?;
                                    nox::error::Result::Ok(id)
                                })?;
                            nox::Result::Ok(())
                        })?;
                    } 
                    Ok(())
                },
            },
            _ => Ok(()),
        }
    }
}

fn main() {
    nox::init();
    let platform = nox::Platform::new();
    let instance = gpu::Instance::new(
        &platform,
        "test",
        nox::Version::new(1, 0, 0),
        &[gpu::InstanceLayer::new(
            gpu::LAYER_KHRONOS_VALIDATION,
            true
        )],
    ).unwrap();
    let device_attributes = gpu
        ::default_device_attributes();
    let devices = instance.enumerate_suitable_physical_devices(
        device_attributes
    ).unwrap();
    let mut idx = 0;
    for (i, device) in devices.iter() {
        if device.device_type() ==
            gpu::PhysicalDeviceType::DiscreteGpu
        {
            idx = i;
        }
    }
    let physical_device = devices.get(idx);
    let queue_family_index = physical_device
        .queue_families()
        .properties()
        .iter()
        .enumerate()
        .find_map(|(i, properties)|
            (properties.queue_flags.contains(gpu::QueueFlags::GRAPHICS))
            .then_some(i as u32)
        ).unwrap();
    let queue_create_info = gpu::DeviceQueueCreateInfo::new(
        "graphics queue",
        queue_family_index,
        0
    );
    let logical_device = devices
        .create_logical_device(idx, &[queue_create_info])
        .unwrap();
    let globals = nox::create_globals();
    let mut app = globals.add(|event_loop| {
        App::new(event_loop)
    });
    nox::Nox::new(
        platform,
        logical_device,
        nox::default_attributes(),
        &globals,
        |event_loop, event| {
            app.event(event_loop, event)
        },
    ).unwrap().run();
}
