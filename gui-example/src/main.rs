use std::{
    fs::{self, File}, io::Write, path::PathBuf
};

use memmap2::Mmap;
use nox::{
    linear_device_alloc::LinearDeviceAlloc,
    *
};

use nox_gui::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MyEnum {
    First,
    Second,
    Third,
}

struct Example<'a> {
    workspace: Workspace<
        'a,
        Self,
        &'static str,
        DefaultStyle<&'static str>,
    >,
    output_format: ColorFormat,
    aspect_ratio: f32,
    slider_value: f32,
    slider_value_int: u32,
    radio_value: MyEnum,
    drag_value_int: i8,
    input_text: String,
    color: ColorSRGBA,
    pipeline_cache: PipelineCacheId,
    sampler: SamplerId,
    cache_dir: PathBuf,
    show_other_window: bool,
    output_image: ImageId,
    output_resolve_image: ImageId,
    tag_color: ColorHSVA,
    resizeable: bool,
    clamp_width: bool,
    clamp_height: bool,
    msaa: MSAA,
    device_alloc: Option<LinearDeviceAlloc>,
}

impl<'a> Example<'a> {

    pub fn new(
        workspace: Workspace<
            'a,
            Self,
            &'static str,
            DefaultStyle<&'static str>,
        >,
    ) -> Self
    {
        let mut cache_dir = std
            ::env::current_exe()
            .unwrap_or_default();
        cache_dir.pop();
        cache_dir.push("my_cache.cache");
        Self {
            workspace,
            output_format: Default::default(),
            aspect_ratio: 1.0,
            slider_value: 0.0,
            slider_value_int: 0,
            radio_value: MyEnum::First,
            drag_value_int: 0,
            input_text: Default::default(),
            color: Default::default(),
            pipeline_cache: Default::default(),
            sampler: Default::default(),
            cache_dir,
            show_other_window: false,
            output_image: Default::default(),
            output_resolve_image: Default::default(),
            tag_color: ColorHSVA::new(0.0, 0.53, 1.0, 0.9),
            resizeable: true,
            clamp_width: true,
            clamp_height: true,
            msaa: MSAA::X8,
            device_alloc: None,
        }
    }
}

impl<'a> Interface for Example<'a> {

    fn init_settings(&self) -> InitSettings {
        InitSettings::new(
            "example",
            Default::default(),
            [540, 540],
            true,
            false,
        )
    }

    fn init_callback(
        &mut self,
        _nox: &mut Nox<Self>,
        renderer: &mut RendererContext,
    ) -> Result<(), Error>
    {
        self.device_alloc = Some(LinearDeviceAlloc::default(1 << 28, &renderer)?);
        renderer.edit_resources(|r| {
            self.output_format = r
                .supported_image_format(
                    &[ColorFormat::SrgbRGBA8, ColorFormat::UnormRGBA8],
                    FormatFeature::ColorAttachment | FormatFeature::SampledImage,
                ).unwrap();
            self.pipeline_cache =
                if fs::exists(&self.cache_dir)? {
                    let file = File::open(&self.cache_dir)?;
                    let map = unsafe {
                        Mmap::map(&file)?
                    };
                    r.create_pipeline_cache(Some(&map))?
                } else {
                    File::create_new(&self.cache_dir)?;
                    r.create_pipeline_cache(None)?
                };
            self.sampler = r.create_sampler(|_| {})?;
            Ok(())
        })?;
        self.workspace
            .create_graphics_pipelines(renderer, self.msaa, self.output_format, None, &GlobalAlloc)?;
        Ok(())
    }

    fn frame_buffer_size_callback(
        &mut self,
        renderer: &mut RendererContext
    ) -> Result<(), Error>
    {
        let frame_buffer_size = renderer.frame_buffer_size();
        renderer.edit_resources(|r| {
            let device_alloc = self.device_alloc.as_mut().unwrap();
            unsafe {
               device_alloc.reset();
            }
            r.destroy_image(self.output_image);
            r.destroy_image(self.output_resolve_image);
            self.output_image = r
                .create_image(ResourceBinderImage::Owned(device_alloc, None), |builder| {
                    builder
                        .with_dimensions(frame_buffer_size)
                        .with_format(self.output_format, false)
                        .with_samples(self.msaa)
                        .with_usage(ImageUsage::ColorAttachment);
            })?;
            self.output_resolve_image = r
                .create_image(ResourceBinderImage::Owned(device_alloc, None), |builder| {
                    builder
                        .with_dimensions(frame_buffer_size)
                        .with_format(self.output_format, false)
                        .with_usage(ImageUsage::ColorAttachment)
                        .with_usage(ImageUsage::Sampled);
            })?;
            Ok(())
        })
    }

    fn update(
        &mut self,
        nox: &mut Nox<Self>,
        _renderer: &mut RendererContext,
    ) -> Result<(), Error> {
        self.workspace.begin(nox)?;
        self.workspace.window(0, "Widgets", [0.0, 0.0], [0.5, 0.5],
            |win| {
                win.checkbox(&mut self.resizeable, "Resizeable");
                win.checkbox(&mut self.clamp_width, "Clamp width");
                win.checkbox(&mut self.clamp_height, "Clamp height");
                win.resizeable(self.resizeable);
                win.clamp_height(self.clamp_height);
                win.clamp_width(self.clamp_width);
                win.collapsing("Show/hide widgets", |win| {

                    win.checkbox(&mut self.show_other_window, "Show other window");
                    win.end_row();

                    win.color_picker(&mut self.color);
                    win.tag("Color picker");
                    win.end_row();

                    if win.button("Print \"hello\"") {
                        println!("hello");
                    }
                    win.end_row();

                    win.radio_button(&mut self.radio_value, MyEnum::First, "First");
                    win.radio_button(&mut self.radio_value, MyEnum::Second, "Second");
                    win.radio_button(&mut self.radio_value, MyEnum::Third, "Third");

                    win.end_row();

                    win.selectable_tag(&mut self.radio_value, MyEnum::First, "First");
                    win.selectable_tag(&mut self.radio_value, MyEnum::Second, "Second");
                    win.selectable_tag(&mut self.radio_value, MyEnum::Third, "Third");

                    win.end_row();

                    win.combo_box("My Combo", |builder| {
                        builder.item(&mut self.radio_value, MyEnum::First, "First");
                        builder.item(&mut self.radio_value, MyEnum::Second, "Second");
                        builder.item(&mut self.radio_value, MyEnum::Third, "Third");
                    });

                    win.end_row();

                    win.input_text(
                        &mut self.input_text,
                        "Input text here",
                        None,
                    );

                    win.collapsing("Sliders", |win| {
                        win.collapsing("Float", |win| {
                            win.slider(&mut self.slider_value, 0.0, 100.0, 200.0);
                            win.tag("Float 1");
                            win.end_row();
                            win.slider(&mut self.slider_value, 0.0, 200.0, 400.0);
                            win.tag("Float 2");
                        });
                        win.collapsing("Int", |win| {
                            //win.tag("Int");
                            win.slider(&mut self.slider_value_int, 0, 10, 20.0);
                        });
                    });
                  
                    win.drag_value(
                        &mut self.drag_value_int,
                        i8::MIN,
                        i8::MAX,
                        500.0,
                        0.01,
                        None,
                    );
                    win.tag("Drag value");
                });
                //win.collapsing("test", |_| {});
            }
        )?;
        if self.show_other_window {
            let mut fmt = String::new();
            <String as core::fmt::Write>::write_fmt(&mut fmt, format_args!("fps: {:.0}", 1.0 / nox.delta_time_secs_f32())).unwrap();
            self.workspace.window(1, fmt.as_str(), [0.25, 0.25], [0.4, 0.4], 
                |win| {
                    let mut fmt = String::new();
                    <String as core::fmt::Write>::write_fmt(&mut fmt, format_args!("Hue: {}°", (self.tag_color.hue * 180.0 / core::f32::consts::PI).round())).unwrap();
                    win.text("Sample text", true, |builder| {
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
        self.tag_color.hue = (self.tag_color.hue + core::f32::consts::PI * nox.delta_time_secs_f32()) % core::f32::consts::TAU;
        self.workspace.end(nox)?;
        Ok(())
    }

    fn render<'b>(
        &mut self,
        frame_graph: &'b mut dyn frame_graph::FrameGraph,
        _pending_transfers: &[CommandRequestId],
    ) -> Result<(), Error> {
        let frame_buffer_size = frame_graph.frame_buffer_size();
        self.aspect_ratio = frame_buffer_size.width as f32 / frame_buffer_size.height as f32;
        let output = frame_graph.add_image(self.output_image)?;
        let output_resolve = frame_graph.add_image(self.output_resolve_image)?;
        self.workspace.render(
            frame_graph,
            (output, None), (Some((output_resolve, ResolveMode::Average)), None),
            AttachmentLoadOp::Clear,
            Default::default(),
            self.sampler,
        )?;
        frame_graph.set_render_image(output_resolve, None)?;
        Ok(())
    }

    fn render_commands(
        &mut self,
        pass_id: frame_graph::PassId,
        commands:&mut RenderCommands,
    ) -> Result<(), Error> {
        self.workspace.render_commands(
            commands,
            pass_id,
        )?;
        Ok(())
    }

    fn clean_up(
        &mut self,
        renderer: &mut RendererContext,
    )
    {
        renderer.edit_resources(|r| {
            let mut file = File::create(&self.cache_dir)?;
            let data = r.retrieve_pipeline_cache_data(self.pipeline_cache)?;
            file.write(&data)?;
            println!("cache written");
            Ok(())
        }).ok();
        unsafe {
            self.workspace.clean_up(&renderer);
            self.device_alloc.take().unwrap().clean_up();
        }
    }
}

fn main() {
    let font = unsafe {
        memmap2::
            Mmap::map(
                &std::fs::File::open("HackNerdFontMono-Regular.ttf").unwrap()
            )
            .unwrap()
    };
    let example = 
        Example::new(Workspace::new(
            [("regular", font::Face::parse(&font, 0).unwrap())], 
            DefaultStyle::new("regular"),
            0.1,
        ));
    Nox::new(example, &mut Default::default()).run();
}
