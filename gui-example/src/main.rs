use std::{
    fs::{self, File}, io::Write, path::PathBuf
};

use memmap2::Mmap;

use nox::*;
use nox::error::Context;

use nox_gui::{geom::{shapes::circle, *}, *};

pub fn my_widget_show(
    ui: &mut impl UiReact,
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
    center.x += lerp(0.0, size.x - radius * 2.0, t);
    let visuals = ui.style().interact_visuals(&reaction);
    ui.paint(move |painter, row| {
        let off = vec2(0.0, row.height_halved - size.y * 0.5);
        painter
            .rect(id, rect, offset + off,
                visuals.fill_col,
                visuals.bg_strokes.clone(),
                visuals.bg_stroke_idx
            )
            .circle(id, circle(Vec2::default(), radius * 0.75), 18,
                center + off,
                visuals.fill_col,
                visuals.fg_strokes.clone(),
                visuals.fg_stroke_idx,
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
    output_format: gpu::ColorFormat,
    aspect_ratio: f32,
    slider_value: f32,
    slider_value_uint: u8,
    radio_value: MyEnum,
    drag_value_int: i8,
    input_text: String,
    color: ColorSRGBA,
    pipeline_cache: gpu::PipelineCacheId,
    sampler: gpu::SamplerId,
    cache_dir: PathBuf,
    show_other_window: bool,
    output_image: gpu::ImageId,
    output_resolve_image: gpu::ImageId,
    tag_color: ColorHSVA,
    resizeable: bool,
    clamp_width: bool,
    clamp_height: bool,
    msaa: gpu::MSAA,
    device_alloc: gpu::linear_device_alloc::LinearDeviceAlloc,
}

impl Example {

    fn init(
        gpu: &mut gpu::GpuContext,
    ) -> Result<Self>
    {
        let mut cache_dir = std
            ::env::current_exe()
            .unwrap_or_default();
        cache_dir.pop();
        cache_dir.push("my_cache.cache");
        Ok(Self {
            output_format: gpu.supported_image_format(
                &[gpu::ColorFormat::SrgbRGBA8, gpu::ColorFormat::UnormRGBA8],
                &[gpu::FormatFeature::ColorAttachment, gpu::FormatFeature::SampledImage],
            ).ok_or(Error::just_context("failed to find suitable output format"))?,
            aspect_ratio: 1.0,
            slider_value: 0.0,
            slider_value_uint: 0,
            radio_value: MyEnum::First,
            drag_value_int: 0,
            input_text: Default::default(),
            color: Default::default(),
            pipeline_cache: if fs::exists(&cache_dir).context("io error")? {
                let file = File
                    ::open(&cache_dir)
                    .context("failed to open pipeline cache file")?;
                let map = unsafe {
                    Mmap::map(&file)
                        .context("failed to map pipeline cache file")?
                };
                gpu.create_pipeline_cache(Some(&map))?
            } else {
                File::create_new(&cache_dir)
                    .context("failed to create pipeline cache file")?;
                gpu.create_pipeline_cache(None)?
            },
            sampler: gpu.create_sampler(|builder| {
                builder
                    .with_mag_filter(gpu::Filter::Linear)
                    .with_min_filter(gpu::Filter::Linear)
                    .max_lod_clamp_none();
            })?,
            cache_dir,
            show_other_window: false,
            output_image: Default::default(),
            output_resolve_image: Default::default(),
            tag_color: ColorHSVA::new(0.0, 0.53, 1.0, 0.9),
            resizeable: true,
            clamp_width: true,
            clamp_height: true,
            msaa: gpu::MSAA::X8,
            device_alloc: gpu::linear_device_alloc::LinearDeviceAlloc
                ::default(1 << 29, &gpu)
                .context("failed to create device alloc")?,
        })
    }
}

singleton_cell_token!(Token);

fn main() {
    let font = unsafe {
        memmap2::
            Mmap::map(
                &std::fs::File::open("HackNerdFontMono-Regular.ttf").unwrap()
            )
            .unwrap()
    };
    let mut token = Token::new().unwrap();
    let workspace = InitCell::new(&mut token);
    let example = InitCell::new(&mut token);
    Nox::new(
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
}
