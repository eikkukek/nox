use image::EncodableLayout;
use nox::{
    Platform, Version,
    gpu::{self, ext},
    log,
    mem::collections::{EntryExt, HashMap},
    sync::{Arc, SwapLock, atomic::{self, AtomicU64}},
};
const BLEND_STATE: gpu::ColorOutputBlendState = gpu::ColorOutputBlendState {
    src_color_blend_factor: gpu::BlendFactor::SrcAlpha,
    dst_color_blend_factor: gpu::BlendFactor::OneMinusSrcAlpha,
    color_blend_op: gpu::BlendOp::Add,
    src_alpha_blend_factor: gpu::BlendFactor::One,
    dst_alpha_blend_factor: gpu::BlendFactor::OneMinusSrcAlpha,
    alpha_blend_op: gpu::BlendOp::Add,
};

#[inline(always)]
pub fn load_rgba_image(path: &str) -> ::image::ImageResult<::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>>> {
    let image = ::image::ImageReader::open(path)?.decode()?;
    Ok(image.to_rgba8())
}

fn main() {
    nox::init();
    let platform = Platform::new();
    let instance = gpu::Instance::new(
        &platform,
        "test",
        Version::new(1, 0, 0),
        &[gpu::InstanceLayer::new(
            gpu::LAYER_KHRONOS_VALIDATION,
            false,
        )],
    ).unwrap();
    let device_attributes = gpu
        ::default_device_attributes()
        .with_device_extension(ext::push_descriptor::Extension);
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
    log::info!("selected device: {}",
        logical_device.physical_device().device_name()
    );
    let queue = logical_device.device_queues()[0].clone();
    let globals = nox::create_globals();
    let window = globals.add(|event_loop| {
        Ok(event_loop.create_window(
            nox::win::default_attributes()
                .with_resizable(true)
                .with_title("mip map")
        )?)
    });
    let pipeline_cache = globals.add(|event_loop| {
        Ok(gpu::PipelineCache::new(
            event_loop.gpu().device().clone(),
            None
        )?)
    });
    let shader_set = globals.add(|event_loop| {
        let vertex_shader = gpu::Shader::new(
            event_loop.gpu(), gpu
                ::default_shader_attributes()
                .with_name("vertex shader")
                .with_stage(gpu::ShaderStage::Vertex)
                .with_glsl("
                    #version 450

                    vec2 positions[6] = vec2[](
                        vec2(1.0, 1.0),
                        vec2(0.0, 1.0),
                        vec2(0.0, 0.0),
                        vec2(1.0, 0.0),
                        vec2(1.0, 1.0),
                        vec2(0.0, 0.0)

                    );

                    vec2 uvs[6] = vec2[](
                        vec2(1.0, 1.0),
                        vec2(0.0, 1.0),
                        vec2(0.0, 0.0),
                        vec2(1.0, 0.0),
                        vec2(1.0, 1.0),
                        vec2(0.0, 0.0)
                    );

                    layout(location = 0) out vec2 out_uv;

                    void main() {
                        uint idx = gl_VertexIndex;
                        out_uv = uvs[idx];
                        const float scale = 0.5;
                        gl_Position = vec4((positions[idx] - vec2(0.5, 0.5)) * scale, 0.0f, 1.0f);
                    }
                ")
            )?;
            let fragment_shader = gpu::Shader::new(
                event_loop.gpu(),
                gpu::default_shader_attributes().with_name("fragment shader")
                    .with_stage(gpu::ShaderStage::Fragment)
                    .with_glsl("
                        #version 450

                        layout(location = 0) in vec2 in_uv;
                        layout(location = 0) out vec4 out_color;

                        layout(set = 0, binding = 0) uniform sampler2D tex;

                        void main() {
                            out_color = texture(tex, in_uv);
                        }
                    ")
            )?;
            Ok(event_loop
                .gpu()
                .create_shader_set(
                    [vertex_shader, fragment_shader],
                    gpu::default_shader_set_attributes()
                        .with_descriptor_set_layout_flags(
                            0,
                            gpu::DescriptorSetLayoutFlags::PUSH_DESCRIPTOR,
                        )
                )?)
    });
    let image = globals.add(|event_loop| {
        let staging_binder = gpu::
            GlobalBinder::new(
                event_loop.gpu().device().clone(),
                gpu::MemoryProperties::HOST_VISIBLE | gpu::MemoryProperties::HOST_COHERENT,
                gpu::MemoryProperties::HOST_VISIBLE | gpu::MemoryProperties::HOST_COHERENT,
            );
        let view_binder = gpu::
            GlobalBinder::new(
                event_loop.gpu().device().clone(),
                gpu::MemoryProperties::DEVICE_LOCAL,
                gpu::MemoryProperties::HOST_VISIBLE,
            );
        let new_img = load_rgba_image("ferris.png").unwrap();
        let gpu = event_loop.gpu();
        let (width, height) = new_img.dimensions();
        let mem_size = (width * height) as gpu::DeviceSize * 4;
        let mip_levels = 32 - (width | height).leading_zeros();
        let (mut staging_id, mut image_id) = Default::default();
        gpu.create_resources(
            [gpu::BufferCreateInfo::new(
                &mut staging_id,
                &staging_binder,
                mem_size,
                gpu::BufferUsages::TRANSFER_SRC,
            ).unwrap()],
            [gpu::ImageCreateInfo
                ::new(&mut image_id, &view_binder)
                .with_dimensions((width, height))
                .with_format(gpu::Format::R8g8b8a8Srgb, false)
                .with_usage(
                    gpu::ImageUsages::TRANSFER_DST |
                    gpu::ImageUsages::TRANSFER_SRC |
                    gpu::ImageUsages::SAMPLED)
                .with_mip_levels(mip_levels)
            ])?;
        let mut map = gpu.map_buffer(staging_id)?;
        unsafe {
            map.write_bytes(new_img.as_bytes())
        }
        let view_id = gpu.create_image_view(
            image_id,
            gpu::ImageRange::whole_range(gpu::ImageAspects::COLOR),
        )?;
        Ok((view_id, staging_id, width, height))
    });
    let timeline_semaphore = globals.add(|event_loop| {
        let mut id = Default::default();
        event_loop
            .gpu()
            .create_timeline_semaphores([(&mut id, 3)])?;
        Ok(id)
    });
    let mut timeline_value = 3;
    #[derive(Default)]
    struct FrameBufferState {
        pipelines: SwapLock<HashMap<gpu::Format, gpu::GraphicsPipelineId>>,
        extent: AtomicU64,
    }
    let fb_state: Arc<FrameBufferState> = Default::default();
    let sampler = gpu::SamplerCreateInfo
        ::default()
        .min_filter(gpu::Filter::Linear)
        .mag_filter(gpu::Filter::Linear)
        .max_lod(gpu::Sampler::LOD_CLAMP_NONE)
        .address_mode(
            gpu::SamplerAddressMode::ClampToBorder,
            gpu::SamplerAddressMode::ClampToBorder,
            gpu::SamplerAddressMode::ClampToBorder,
        ).build(logical_device.clone()).unwrap();
    nox::Nox::new(
        platform,
        logical_device,
        nox::default_attributes(),
        &globals,
        |event_loop, event| {
            match event {
                nox::Event::Update => {
                    let window = *window;
                    if !event_loop.is_window_valid(window) {
                        event_loop.exit();
                    }
                    let mut commands = event_loop.gpu().schedule_commands();
                    let (view, staging, width, height) = *image;
                    let fb_state = fb_state.clone();
                    let sampler = sampler.clone();
                    commands.new_commands::<gpu::NewGraphicsCommands>(
                        queue.clone(),
                        move |cmd| {
                            if timeline_value == 3 {
                                let mut cmd = cmd.copy_commands();
                                cmd.copy_buffer_to_image(
                                    staging,
                                    view.image_id(),
                                    &[gpu::BufferImageCopy
                                        ::default()
                                        .image_subresource(gpu::ImageSubresourceLayers
                                            ::default()
                                            .aspect_mask(gpu::ImageAspects::COLOR)
                                        ).image_extent((width, height))
                                    ],
                                    gpu::CommandOrdering::Strict
                                )?;
                                cmd.gen_mip_map(view.image_id(), gpu::Filter::Linear)?;
                            }
                            let (image_view, format) = cmd.swapchain_image_view(
                                window.surface_id()
                            )?;
                            cmd.render(
                                gpu::RenderingInfo::default(),
                                &[
                                    gpu::PassAttachment
                                        ::new(image_view)
                                        .with_load_op(gpu::AttachmentLoadOp::Clear)
                                        .with_clear_value(
                                            gpu::ClearColorValue::Float([0.01, 0.01, 0.01, 0.5])
                                        ),
                                ],
                                &gpu::DepthStencilAttachment::None,
                                |pass| {
                                    let frame_buffer_size = fb_state.extent.load(
                                        atomic::Ordering::Acquire
                                    );
                                    let (width, height) = (
                                        (frame_buffer_size & 0xFFFFFFFF) as u32,
                                        (frame_buffer_size >> 32) as u32,
                                    );
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
                                        pipeline_cmd.push_descriptor_bindings(&[
                                            gpu::PushDescriptorBinding::new(
                                                "tex",
                                                0,
                                                gpu::DescriptorInfos::images(&[
                                                    gpu::DescriptorImageInfo {
                                                        sampler: Some(sampler.clone()),
                                                        image_view: Some(view),
                                                    },
                                                ]),
                                                gpu::CommandBarrierInfo::new(
                                                    gpu::CommandOrdering::Strict,
                                                    gpu::ExplicitAccess::SHADER_READ,
                                                )
                                            )?,
                                        ])?;
                                        pipeline_cmd.begin_drawing(
                                            gpu::DrawInfo
                                                ::default()
                                                .vertex_count(6),
                                                &[], None,
                                                |cmd| {
                                                    cmd.draw()?;
                                                    Ok(())
                                                }
                                            )?;
                                        Ok(())
                                    })?;
                                    Ok(())
                                }
                            )?;
                            Ok(())
                        },
                    )?.with_wait_semaphore(
                        *timeline_semaphore,
                        timeline_value - 2,
                        gpu::MemoryDependencyHint::TRANSFER,
                    ).with_signal_semaphore(
                        *timeline_semaphore,
                        timeline_value + 1,
                    );
                    timeline_value += 1;
                    Ok(())
                },
                nox::Event::GpuEvent(event) => {
                    match event {
                        gpu::Event::SwapchainCreated {
                            surface_id: _, new_format, new_size, image_count: _
                        } => {
                            log::debug!("new surface format: {new_format}");
                            let mut extent = new_size.0 as u64;
                            extent |= (new_size.1 as u64) << 32;
                            fb_state.extent.store(extent, atomic::Ordering::Release);
                            if fb_state.pipelines.load().get(&new_format).is_none() {
                                fb_state.pipelines
                                .modify(|pipelines| {
                                    pipelines
                                        .entry(new_format)
                                        .or_try_insert_with_key(|&format| {
                                            let mut id = Default::default();
                                            let mut batch = event_loop
                                                .gpu()
                                                .create_pipeline_batch(pipeline_cache.clone())?;
                                            batch 
                                                .with_graphics_pipelines([
                                                    gpu::GraphicsPipelineCreateInfo
                                                        ::new(&mut id, *shader_set)
                                                        .with_color_output(
                                                            format,
                                                            gpu::ColorComponents::default(),
                                                            Some(BLEND_STATE),
                                                        )
                                                ]);
                                            let _ = batch.build()?;
                                            nox::error::Result::Ok(id)
                                        })?;
                                    nox::Result::Ok(())
                                })?;
                            } 
                            Ok(())
                        },
                    }
                }
                _ => Ok(())
            }
        }
    ).unwrap().run();
}
