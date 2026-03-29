use std::{
    fs::{self, File},
};

use memmap2::Mmap;

use nox::{gpu, event_loop, sync::{*, atomic::AtomicU64}};
use gpu::{MemoryBinder, Commands};
use nox::error::Context;

use nox_gui::*;

pub fn my_widget_show<'a>(
    ui: &mut impl UiReact<'a>,
    reaction: &mut Reaction,
    value: &mut bool,
    _label: &str,
) {
    let size = ui.standard_interact_height() * vec2(2.0, 1.0);
    reaction.size = size;
    let id = reaction.id();
    let offset = reaction.offset();
    if reaction.clicked() {
        *value = !*value;
    }
    let radius = size.y * 0.5;
    let rect = shapes::rect(Default::default(), size, radius);
    let mut center = offset + vec2(radius, radius);
    let t = ui.animated_bool(reaction.id(), *value);
    center.x += nox_gui::geom::lerp(0.0, size.x - radius * 2.0, t);
    let visuals = ui.style().interact_visuals(reaction);
    ui.paint(move |painter, row| {
        let off = vec2(0.0, row.height_halved - size.y * 0.5);
        painter
            .rect(
                id,
                offset + off,
                UiRect::bg(&visuals)
                    .rect(rect.min, rect.max, rect.rounding)
            ).circle(
                id,
                center + off,
                UiCircle::fg(&visuals)
                    .circle(vec2(0.0, 0.0), radius * 0.75)
                    .steps(18)
            );
    });
}

impl_widget!(my_widget_show, my_widget_ui(value: &'a mut bool, label: &str));

/*
pub fn my_widget_ui<'a, Surface: UiReactSurface, Style: UiStyle>(
    value: &'a mut bool,
    label: &str,
) -> (&'a mut bool, impl FnMut(&mut UiReactCtx<Surface, Style>, &mut ReactionEntry, &mut bool)) {
    (value, |ui, reaction, value| my_widget_show(ui, reaction, value, label))
}
*/

#[derive(Clone, Copy, PartialEq, Eq)]
enum MyEnum {
    First,
    Second,
    Third,
}

struct Example {
    aspect_ratio: f32,
    slider_value: f32,
    slider_value_uint: u8,
    radio_value: MyEnum,
    drag_value_int: i8,
    input_text: String,
    color: ColorSRGBA,
    show_other_window: bool,
    semaphore_id: gpu::TimelineSemaphoreId,
    frame_data: Arc<(AtomicU64, SwapLock<[gpu::ImageViewId; 3]>)>,
    tag_color: ColorHSVA,
    resizeable: bool,
    clamp_width: bool,
    clamp_height: bool,
    msaa: gpu::MsaaSamples,
    device_binder: gpu::LinearBinder,
}

impl Example {

    fn new(
        event_loop: &event_loop::ActiveEventLoop<'_>,
    ) -> nox::EventResult<Self>
    {
        let mut semaphore_id = Default::default();
        event_loop.gpu().create_timeline_semaphores([(&mut semaphore_id, 0)])?;
        Ok(Self {
            aspect_ratio: 1.0,
            slider_value: 0.0,
            slider_value_uint: 0,
            radio_value: MyEnum::First,
            drag_value_int: 0,
            input_text: Default::default(),
            color: Default::default(), 
            show_other_window: false,
            semaphore_id,
            frame_data: Default::default(),
            tag_color: ColorHSVA::new(0.0, 0.53, 1.0, 0.9),
            resizeable: true,
            clamp_width: true,
            clamp_height: true,
            msaa: gpu::MsaaSamples::X8,
            device_binder: gpu::LinearBinder::new(
                event_loop.gpu().device().clone(),
                1 << 29,
                gpu::MemoryProperties::DEVICE_LOCAL,
                gpu::MemoryProperties::DEVICE_LOCAL
            )?,
        })
    }
}

fn main() {
    nox::init();
    let font = include_bytes!("../HackNerdFontMono-Regular.ttf");
    let font: Box<[_]> = font.iter().copied().collect();
    let platform = nox::Platform::new();
    let instance = gpu::Instance::new(
        &platform,
        "gui example",
        nox::Version::default(),
        //&[gpu::InstanceLayer::new(gpu::LAYER_KHRONOS_VALIDATION, false)],
        &[],
    ).unwrap();
    let devices = instance.enumerate_suitable_physical_devices(
        gpu::default_device_attributes()
            .with_device_extension(gpu::ext::push_descriptor::Extension)
    ).unwrap();
    let mut idx = 0;
    if let Some((i, _)) = devices
        .iter()
        .find(|(_, device)| device.device_type() == gpu::PhysicalDeviceType::DiscreteGpu)
    {
        idx = i;
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
    let queue = logical_device.device_queues()[0].clone();
    let globals = nox::create_globals();
    let mut cache_dir = std
        ::env::current_exe()
        .unwrap_or_default();
    cache_dir.pop();
    cache_dir.push("my_cache.cache");
    let pipeline_cache = if fs::exists(&cache_dir).context("io error").unwrap() {
        let file = File
            ::open(&cache_dir)
            .context("failed to open pipeline cache file").unwrap();
        let map = unsafe {
            Mmap::map(&file)
                .context("failed to map pipeline cache file").unwrap()
        };
        gpu::PipelineCache::new(
            logical_device.clone(),
            Some(&map)
        ).unwrap()
    } else {
        File::create_new(&cache_dir)
            .context("failed to create pipeline cache file").unwrap();
        gpu::PipelineCache::new(
            logical_device.clone(),
            None
        ).unwrap()
    };
    let mut workspace = globals.add(move |event_loop| Ok(Workspace::new(
        event_loop,
        None,
        Some(pipeline_cache.clone()),
        nox_gui::default_attribures(
            [("regular", font::parse_owned_face(font, 0).unwrap())], 
            "regular",
        ),
    )?));
    let mut example = globals.add(Example::new);
    let window = globals.add(|event_loop| Ok(event_loop.create_window(
        nox::win::default_attributes()
            .with_resizable(true)
    )?));
    nox::Nox::new(
        platform,
        logical_device,
        nox::default_attributes()
            .with_desired_buffered_frames(3.try_into().unwrap()),
        &globals,
        |event_loop, event| {
            match event {
                nox::Event::GpuEvent(event) => match event {
                    gpu::Event::SwapchainCreated {
                        surface_id: _, new_format, new_size, image_count
                    } => {
                        workspace.reallocate_frame_resources(
                            image_count,
                            new_format,
                            gpu::MsaaSamples::X8
                        )?;
                        let gpu = event_loop.gpu();
                        let (frame, render_views) = &*example.frame_data;
                        if !gpu.wait_for_semaphores(
                            &[(example.semaphore_id, frame.load(atomic::Ordering::Acquire))],
                            core::time::Duration::from_secs(2)
                        )? {
                            return Err(nox::EventError::just_context(
                                "timeout"
                            ))
                        }
                        render_views.modify(|views| {
                            if views[0] != Default::default() {
                                gpu.destroy_resources(
                                    [],
                                    views
                                        .iter().map(|id| id.image_id())
                                )?;
                            }
                            unsafe {
                                example.device_binder.release_resources();
                            }
                            let mut ids = [Default::default(); 3];
                            gpu.create_resources(
                                [],
                                ids.iter_mut().map(|id| {
                                    gpu::ImageCreateInfo::new(
                                        id,
                                        &example.device_binder,
                                    ).with_dimensions(new_size)
                                    .with_format(new_format, false)
                                    .with_usage(gpu::ImageUsages::COLOR_ATTACHMENT)
                                    .with_samples(gpu::MsaaSamples::X8)
                                })
                            )?;
                            *views = [
                                gpu.create_image_view(
                                    ids[0],
                                    gpu::ImageRange::whole_range(gpu::ImageAspects::COLOR),
                                )?,
                                gpu.create_image_view(
                                    ids[1],
                                    gpu::ImageRange::whole_range(gpu::ImageAspects::COLOR),
                                )?,
                                gpu.create_image_view(
                                    ids[2],
                                    gpu::ImageRange::whole_range(gpu::ImageAspects::COLOR),
                                )?
                            ];
                            nox::Result::Ok(())
                        })?;
                        Ok(())
                    },
                },
                nox::Event::Update => {
                    let Some(win) = event_loop.get_window(*window) else {
                        event_loop.exit();
                        return Ok(())
                    };
                    let mut ui = workspace.begin(win)?;
                    ui.window(
                        0,
                        "Widgets",
                        [0.0, 0.0], [1.0, 1.0],
                        |ui| {
                            ui.checkbox(&mut example.resizeable, "Resizeable");
                            ui.checkbox(&mut example.clamp_width, "Clamp width");
                            ui.checkbox(&mut example.clamp_height, "Clamp height");
                            ui.resizeable(example.resizeable);
                            ui.clamp_height(example.clamp_height);
                            ui.clamp_width(example.clamp_width); 
                            ui.collapsing("Show/hide widgets", |ui| {

                                ui.checkbox(&mut example.show_other_window, "Show other window");
                                ui.end_row();

                                /*
                                ui.color_picker(&mut self.color);
                                ui.tag("Color picker");
                                ui.end_row();
                                */

                                let image_source = image_source!("ferris.png");
                                let image_size = ui.standard_interact_height() * 2.0;
                                ui.image("ferris", image_source, geom::vec2(image_size, image_size))
                                    .hover_text("ferris!");

                                if ui.button("Print \"hello\"").clicked() {
                                    println!("hello");
                                }
                                ui.end_row();

                                ui.radio_button(&mut example.radio_value, MyEnum::First, "First");
                                ui.radio_button(&mut example.radio_value, MyEnum::Second, "Second");
                                ui.radio_button(&mut example.radio_value, MyEnum::Third, "Third");

                                ui.end_row();

                                ui.selectable_tag(&mut example.radio_value, MyEnum::First, "First");
                                ui.selectable_tag(&mut example.radio_value, MyEnum::Second, "Second");
                                ui.selectable_tag(&mut example.radio_value, MyEnum::Third, "Third");

                                ui.end_row();

                                /*
                                ui.combo_box("My Combo", |builder| {
                                    builder.item(&mut self.radio_value, MyEnum::First, "First");
                                    builder.item(&mut self.radio_value, MyEnum::Second, "Second");
                                    builder.item(&mut self.radio_value, MyEnum::Third, "Third");
                                });
                                */

                                ui.end_row();

                                ui.input_text(
                                    &mut example.input_text,
                                    "Input text here",
                                    None,
                                );

                                ui.collapsing("Sliders", |ui| {
                                    ui.collapsing("f32", |ui| {
                                        ui.slider(&mut example.slider_value, 0.0, 100.0, 200.0);
                                        //ui.tag("f32 1");
                                        ui.end_row();
                                        ui.slider(&mut example.slider_value, 0.0, 200.0, 400.0);
                                        //ui.tag("f32 2");
                                    });
                                    ui.collapsing("u8", |ui| {
                                        ui.slider(&mut example.slider_value_uint, 0, 10, 20.0);
                                    });
                                });

                                ui.drag_value(
                                    &mut example.drag_value_int,
                                    i8::MIN,
                                    i8::MAX,
                                    500.0,
                                    0.01,
                                    None,
                                );
                                //ui.tag("Drag value");
                                ui.end_row();
                                ui.add(my_widget_ui(&mut example.show_other_window, "test"))
                                    .hover_text("Simple custom widget");
                            });
                        }
                    )?;
                    let surface_id = window.surface_id();
                    let semaphore = example.semaphore_id;
                    let frame_data = example.frame_data.clone();
                    ui.end(
                        queue.clone(),
                        move |cmd| {
                            let (frame, views) = &*frame_data;
                            let frame = frame.fetch_add(1, atomic::Ordering::AcqRel);
                            cmd.add_wait_semaphore(
                                semaphore, frame, gpu::MemoryDependencyHint::VERTEX_INPUT,
                            );
                            cmd.add_signal_semaphore(semaphore, frame + 1);
                            let (swapchain_view, _) = cmd.swapchain_image_view(surface_id)?;
                            Ok(gpu::PassAttachment::new(views.load()[(frame % 3) as usize])
                                .with_load_op(gpu::AttachmentLoadOp::Clear)
                                .with_clear_value([0.01, 0.01, 0.01, 0.1])
                                .with_resolve(gpu::ResolveInfo::new(
                                    swapchain_view, gpu::ResolveMode::Average
                            )))
                        },
                    )?;
                    Ok(())
                },
                _ => { Ok(()) }
            }
        }
    ).unwrap().run();
    /*
    nox::Nox::new(
        token,
        |token, _win, gpu| {
            let example = example.borrow_or_try_init(token, || {
                Example::init(gpu)
            })?;
            workspace.borrow_or_try_init(token, || {
                Workspace::init(
                    [("regular", font::Face::parse(&font, 0).unwrap())], 
                    DefaultStyle::new("regular"),
                    0.2,
                    gpu,
                    1 << 26,
                    Some(example.pipeline_cache),
                    example.output_format,
                    example.msaa,
                    &GlobalAlloc,
                )
            })?;
            Ok(())
        },
        |token, event| {
            match event {
                Event::FrameBufferCreated { gpu, new_size, new_format: _ } => {
                    let example = example
                        .borrow_mut(token)
                        .unwrap();
                    unsafe {
                       example.device_alloc.reset();
                    }
                    gpu.destroy_image(example.output_image);
                    gpu.destroy_image(example.output_resolve_image);
                    example.output_image = gpu
                        .create_image(gpu::ResourceBinderImage::Owned(&mut example.device_alloc, None), |builder| {
                            builder
                                .with_dimensions(new_size)
                                .with_format(example.output_format, false)
                                .with_samples(example.msaa)
                                .with_usage(gpu::ImageUsage::ColorAttachment);
                    })?;
                    example.output_resolve_image = gpu
                        .create_image(gpu::ResourceBinderImage::Owned(&mut example.device_alloc, None), |builder| {
                            builder
                                .with_dimensions(new_size)
                                .with_format(example.output_format, false)
                                .with_usage(gpu::ImageUsage::ColorAttachment)
                                .with_usage(gpu::ImageUsage::Sampled);
                    })?;
                    Ok(())
                },
                Event::Update { win, mut gpu } => {
                    let workspace = workspace
                        .borrow_mut(token)
                        .unwrap();
                    let example = example
                        .borrow_mut(token)
                        .unwrap();
                    workspace.begin(win)?;
                    workspace.window(win, 0, "Widgets", [0.0, 0.0], [0.5, 0.5],
                        |ui| {
                            ui.checkbox(&mut example.resizeable, "Resizeable");
                            ui.checkbox(&mut example.clamp_width, "Clamp width");
                            ui.checkbox(&mut example.clamp_height, "Clamp height");
                            ui.resizeable(example.resizeable);
                            ui.clamp_height(example.clamp_height);
                            ui.clamp_width(example.clamp_width);

                            ui.collapsing("Show/hide widgets", |ui| {

                                ui.checkbox(&mut example.show_other_window, "Show other window");
                                ui.end_row();

                                /*
                                ui.color_picker(&mut self.color);
                                ui.tag("Color picker");
                                ui.end_row();
                                */

                                let image_source = image_source!("ferris.png");
                                let image_size = ui.standard_interact_height() * 2.0;
                                ui
                                    .image("ferris", image_source, geom::vec2(image_size, image_size))
                                    .hover_text("ferris!");
                                if ui.button("Print \"hello\"").clicked() {
                                    println!("hello");
                                }
                                ui.end_row();

                                ui.radio_button(&mut example.radio_value, MyEnum::First, "First");
                                ui.radio_button(&mut example.radio_value, MyEnum::Second, "Second");
                                ui.radio_button(&mut example.radio_value, MyEnum::Third, "Third");

                                ui.end_row();

                                ui.selectable_tag(&mut example.radio_value, MyEnum::First, "First");
                                ui.selectable_tag(&mut example.radio_value, MyEnum::Second, "Second");
                                ui.selectable_tag(&mut example.radio_value, MyEnum::Third, "Third");

                                ui.end_row();

                                /*
                                ui.combo_box("My Combo", |builder| {
                                    builder.item(&mut self.radio_value, MyEnum::First, "First");
                                    builder.item(&mut self.radio_value, MyEnum::Second, "Second");
                                    builder.item(&mut self.radio_value, MyEnum::Third, "Third");
                                });
                                */

                                ui.end_row();

                                ui.input_text(
                                    &mut example.input_text,
                                    "Input text here",
                                    None,
                                );

                                ui.collapsing("Sliders", |ui| {
                                    ui.collapsing("f32", |ui| {
                                        ui.slider(&mut example.slider_value, 0.0, 100.0, 200.0);
                                        //ui.tag("f32 1");
                                        ui.end_row();
                                        ui.slider(&mut example.slider_value, 0.0, 200.0, 400.0);
                                        //ui.tag("f32 2");
                                    });
                                    ui.collapsing("u8", |ui| {
                                        ui.slider(&mut example.slider_value_uint, 0, 10, 20.0);
                                    });
                                });

                                ui.drag_value(
                                    &mut example.drag_value_int,
                                    i8::MIN,
                                    i8::MAX,
                                    500.0,
                                    0.01,
                                    None,
                                );
                                //ui.tag("Drag value");
                                ui.end_row();
                                ui.add(my_widget_ui(&mut example.show_other_window, "test"))
                                    .hover_text("Simple custom widget");
                            });
                        }
                    )?;
                    /*
                    if self.show_other_window {
                        let mut fmt = String::new();
                        <String as core::fmt::Write>::write_fmt(&mut fmt, format_args!("fps: {:.0}", 1.0 / ctx.delta_time_secs_f32())).unwrap();
                        self.workspace.window(ctx, 1, fmt.as_str(), [0.25, 0.25], [0.4, 0.4], 
                            |ui| {
                                let mut fmt = String::new();
                                <String as core::fmt::Write>::write_fmt(&mut fmt, format_args!("Hue: {}°", (self.tag_color.hue * 180.0 / core::f32::consts::PI).round())).unwrap();
                                ui.text("Sample text", true, |builder| {
                                    builder
                                        .with_text(None, |builder| {
                                            builder
                                                .with_segment("This text be copied to ", None);
                                        })
                                        .color(ColorSRGBA::white(1.0))
                                        .with_text(Some("Ctrl+V"), |builder| {
                                            builder
                                                .with_segment("clipboard", None);
                                        })
                                        .default_color()
                                        .with_text(None, |builder| {
                                            builder
                                                .with_segment(" and it can have ", None);
                                        })
                                        .color(self.tag_color)
                                        .with_text(Some(&fmt), |builder| {
                                            builder
                                                .with_segment("tooltips and color", None);
                                        });
                                });
                        })?;
                    }
                    */
                    example.tag_color.hue = (example.tag_color.hue + core::f32::consts::PI * win.delta_time_secs_f32()) % core::f32::consts::TAU;
                    workspace.end(win, &mut gpu)?;
                    Ok(())
                },
                Event::Render { frame_graph, pending_transfers: _ } => {
                    let example = example
                        .borrow_mut(token)
                        .unwrap();
                    let workspace = workspace
                        .borrow_mut(token)
                        .unwrap();
                    let frame_buffer_size = frame_graph.gpu().frame_buffer_size();
                    example.aspect_ratio = frame_buffer_size.width as f32 / frame_buffer_size.height as f32;
                    let output = frame_graph.add_image(example.output_image)?;
                    let output_resolve = frame_graph.add_image(example.output_resolve_image)?;
                    workspace.render(
                        frame_graph,
                        (output, None),
                        Some(gpu::WriteResolveInfo::new(output_resolve, gpu::ResolveMode::Average, None)),
                        gpu::AttachmentLoadOp::Clear,
                        Default::default(),
                    )?;
                    //frame_graph.set_render_image(output_resolve, None)?;
                    Ok(())
                },
                Event::CleanUp { gpu } => {
                    let example = example
                        .borrow_mut(token)
                        .unwrap();
                    let mut file = File::create(&example.cache_dir).context("io error")?;
                    let data = gpu.retrieve_pipeline_cache_data(example.pipeline_cache)?;
                    file.write(&data).context("io error")?;
                    println!("cache written");
                    unsafe {
                        example.device_alloc.clean_up();
                        workspace
                            .borrow_mut(token)
                            .unwrap()
                            .clean_up(gpu)?;
                    }
                    Ok(())
                },
                _ => { Ok(()) }
            }
        },
        InitSettings::new(
            "gui example",
            Default::default(),
            [540, 540],
            true,
            false,
        ),
        &mut Default::default(),
    ).run();
    */
}
