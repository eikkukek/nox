use nox::*;
use nox_gui::Workspace;

struct Example<'a> {
    workspace: Workspace<'a, &'static str>,
    output_format: ColorFormat,
}

impl<'a> Example<'a> {

    pub fn new(workspace: Workspace<'a, &'static str>) -> Self {
        Self {
            workspace,
            output_format: Default::default(),
        }
    }
}

impl<'a> Interface for Example<'a> {

    fn init_settings(&self) -> InitSettings {
        InitSettings::new(
            "example",
            Default::default(),
            [540, 540],
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
            Ok(())
        })?;
        self.workspace
            .create_graphics_pipelines(renderer, MSAA::X1, self.output_format, None, &GLOBAL_ALLOC)?;
        self.workspace.add_widget([0.25, 0.25], [0.0, 0.0]);
        Ok(())
    }

    fn render<'b>(
        &mut self,
        frame_graph: &'b mut dyn frame_graph::FrameGraphInit,
        _pending_transfers: &[CommandRequestId],
    ) -> Result<(), Error> {
        let frame_graph = frame_graph.init(1)?;
        let frame_buffer_size = frame_graph.frame_buffer_size();
        let output = frame_graph.add_transient_image(&mut |builder| {
            builder
                .with_dimensions(frame_buffer_size)
                .with_format(self.output_format, false)
                .with_usage(ImageUsage::ColorAttachment)
                .with_usage(ImageUsage::Sampled);
        })?;
        frame_graph.add_pass(
            PassInfo {
                max_color_writes: 1, ..Default::default()
            },
            &mut |builder| {
                builder
                    .with_write(WriteInfo::new(
                        output, None, None, None,
                        AttachmentLoadOp::Clear,
                        AttachmentStoreOp::Store,
                        Default::default()
                    ));
            })?;
        frame_graph.set_render_image(output, None)?;
        Ok(())
    }

    fn render_commands(
        &mut self,
        _pass_id: frame_graph::PassId,
        commands:&mut RenderCommands,
    ) -> Result<(), Error> {
        self.workspace.render(commands)?;
        Ok(())
    }
}

fn main() {
    let example = Example::new(Workspace::new([], 0.01));
    Nox::new(example, &mut Default::default()).run();
}
