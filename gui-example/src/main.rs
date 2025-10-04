use nox::*;
use nox_gui::{
    *
};

struct Example<'a> {
    workspace: Workspace<'a, &'static str>,
    output_format: ColorFormat,
    aspect_ratio: f32,
    slider_value: f32,
}

impl<'a> Example<'a> {

    pub fn new(workspace: Workspace<'a, &'static str>) -> Self {
        Self {
            workspace,
            output_format: Default::default(),
            aspect_ratio: 1.0,
            slider_value: 0.0,
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
            Ok(())
        })?;
        self.workspace
            .create_graphics_pipelines(renderer, MSAA::X8, self.output_format, None, &GlobalAlloc)?;
        Ok(())
    }

    fn update(
        &mut self,
        nox: &mut Nox<Self>,
        _renderer: &mut RendererContext,
    ) -> Result<(), Error> {
        self.workspace.update_window(0, "Widgets", [0.5, 0.5], [0.0, 0.0],
            |mut win| {
                win.update_slider(0, "Slider 1", &mut self.slider_value, 0.0, 100.0);
                win.update_slider(1, "Slider 2", &mut self.slider_value, 0.0, 200.0);
                Ok(())
            }
        )?;
        self.workspace.end(nox);
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
        let output = frame_graph.add_transient_image(&mut |builder| {
            builder
                .with_dimensions(frame_buffer_size)
                .with_format(self.output_format, false)
                .with_samples(MSAA::X8)
                .with_usage(ImageUsage::ColorAttachment);
        })?;
        let output_resolve = frame_graph.add_transient_image(&mut |builder| {
            builder
                .with_dimensions(frame_buffer_size)
                .with_format(self.output_format, false)
                .with_usage(ImageUsage::ColorAttachment)
                .with_usage(ImageUsage::Sampled);
        })?;
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
                        Default::default()
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
}

fn main() {
    let font = unsafe {
        memmap2::
            Mmap::map(
                &std::fs::File::open("HackNerdFontMono-Regular.ttf").unwrap()
            )
            .unwrap()
    };
    let example = Example::new(Workspace::new([("regular", font::Face::parse(&font, 0).unwrap())], "regular", 0.01));
    Nox::new(example, &mut Default::default()).run();
}
