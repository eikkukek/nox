use std::{
    fs::{self, File},
    path::PathBuf,
    io::Write,
};

use memmap2::Mmap;
use nox::*;

use nox_gui::*;

struct Example<'a> {
    workspace: Workspace<
        'a,
        Self,
        &'static str,
        DefaultStyle<&'static str>,
        DefaultHoverStyle<&'static str>,
    >,
    output_format: ColorFormat,
    aspect_ratio: f32,
    slider_value: f32,
    slider_value_int: u32,
    drag_value_int: i8,
    input_text: String,
    color: ColorSRGBA,
    pipeline_cache: PipelineCacheId,
    cache_dir: PathBuf,
    checkbox_checked: bool,
    show_other_window: bool,
    output_image: ImageId,
    output_resolve_image: ImageId,
}

impl<'a> Example<'a> {

    pub fn new(
        workspace: Workspace<
            'a,
            Self,
            &'static str,
            DefaultStyle<&'static str>,
            DefaultHoverStyle<&'static str>,
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
            drag_value_int: 0,
            input_text: Default::default(),
            color: Default::default(),
            pipeline_cache: Default::default(),
            cache_dir,
            checkbox_checked: false,
            show_other_window: false,
            output_image: Default::default(),
            output_resolve_image: Default::default(),
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
    ) -> Result<(), Error> {

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
            Ok(())
        })?;
        self.workspace
            .create_graphics_pipelines(renderer, MSAA::X8, self.output_format, None, &GlobalAlloc)?;
        Ok(())
    }

    fn frame_buffer_size_callback(
        &mut self,
        renderer: &mut RendererContext
    ) -> Result<(), Error>
    {
        let frame_buffer_size = renderer.frame_buffer_size();
        renderer.edit_resources(|r| {
            r.destroy_image(self.output_image);
            r.destroy_image(self.output_resolve_image);
            self.output_image = r
                .create_image(&mut r.default_binder(), |builder| {
                    builder
                        .with_dimensions(frame_buffer_size)
                        .with_format(self.output_format, false)
                        .with_samples(MSAA::X8)
                        .with_usage(ImageUsage::ColorAttachment);
            })?;
            self.output_resolve_image = r
                .create_image(&mut r.default_binder(), |builder| {
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
        self.workspace.begin()?;
        self.workspace.update_window(0, "Widgets", [0.0, 0.0], [0.5, 0.5],
            |mut win| {
                if win.update_checkbox(0, "Show sliders", &mut self.checkbox_checked) {
                    win.update_slider(0, "Slider 1", &mut self.slider_value, 0.0, 100.0)?;
                    win.update_slider(1, "Slider 2", &mut self.slider_value, 0.0, 200.0)?;
                    win.update_slider(2, "Slider 3", &mut self.slider_value_int, 0, 10)?;
                }
                win.update_checkbox(1, "Show other window", &mut self.show_other_window);
                win.update_color_picker(0, "Color picker", &mut self.color);
                if win.update_button(0, "Print \"hello\"") {
                    println!("hello");
                }
                win.update_input_text(0,
                    "Input text", &mut self.input_text,
                    "Input text here",
                    None,
                );
                win.update_drag_value(
                    0,
                    "Input int",
                    &mut self.drag_value_int,
                    i8::MIN,
                    i8::MAX,
                    Some(500.0),
                    0.1,
                    false,
                    None,
                );
                Ok(())
            }
        )?;
        if self.show_other_window {
            let mut fmt = String::new();
            <String as core::fmt::Write>::write_fmt(&mut fmt, format_args!("fps: {:.0}", 1.0 / nox.delta_time_secs_f32())).unwrap();
            self.workspace.update_window(1, fmt.as_str(), [0.25, 0.25], [0.0, 0.0], 
                |mut _win| {
                    Ok(())
            })?;
        }
        self.workspace.end(nox)?;
        Ok(())
    }

    fn render<'b>(
        &mut self,
        frame_graph: &'b mut dyn frame_graph::FrameGraphInit,
        _pending_transfers: &[CommandRequestId],
    ) -> Result<(), Error> {
        let frame_graph = frame_graph.init(1)?;
        let frame_buffer_size = frame_graph.frame_buffer_size();
        self.aspect_ratio = frame_buffer_size.width as f32 / frame_buffer_size.height as f32;
        let output = frame_graph.add_image(self.output_image)?;
        let output_resolve = frame_graph.add_image(self.output_resolve_image)?;
        frame_graph.add_pass(
            PassInfo {
                max_color_writes: 1,
                msaa_samples: MSAA::X8,
                ..Default::default()
            },
            &mut |builder| {
                builder
                    .with_write(WriteInfo::new(
                        output,
                        None,
                        Some((output_resolve, ResolveMode::Average)),
                        None,
                        AttachmentLoadOp::Clear,
                        AttachmentStoreOp::Store,
                        ClearValue::Color([0.05, 0.01, 0.01, 1.0].into()),
                    ));
            })?;
        frame_graph.set_render_image(output_resolve, None)?;
        Ok(())
    }

    fn render_commands(
        &mut self,
        _pass_id: frame_graph::PassId,
        commands:&mut RenderCommands,
    ) -> Result<(), Error> {
        self.workspace.render_commands(
            commands,
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
            DefaultHoverStyle::new("regular"),
            0.01,
        ));
    Nox::new(example, &mut Default::default()).run();
}
