use std::fs::File;

use memmap2::Mmap;

use nox::{
    mem::{size_of, slice_as_bytes, value_as_bytes, vec_types::GlobalVec},
    *
};

use nox_font::*;

#[derive(Default)]
pub struct App {
    color_format: ColorFormat,
    vertex_buffers: GlobalVec<BufferId>,
    vertex_offset_buffers: GlobalVec<BufferId>,
    index_buffers: GlobalVec<BufferId>,
    vertex_shader: ShaderId,
    fragment_shader: ShaderId,
    pipeline_layout: PipelineLayoutId,
    pipeline: GraphicsPipelineId,
    rendered_text: RenderedText,
    frame_buffer_size: Dimensions,
}

impl Interface for App {

    fn init_settings(&self) -> InitSettings {
        InitSettings::new(
            "font test",
            Version::new(0, 1, 0),
            [540, 540],
            false,
        )
    }

    fn init_callback(
        &mut self,
        _nox: &mut Nox<Self>,
        renderer: &mut renderer::RendererContext,
    ) -> Result<(), Error> {
        let regular = File::open("adobe-garamond/AGaramondPro-Regular.otf")?;
        let regular = unsafe {
            Mmap::map(&regular)?
        };
        let regular = Face::parse(&regular, 0).unwrap();
        let italic = File::open("adobe-garamond/AGaramondPro-Italic.otf")?;
        let italic = unsafe {
            Mmap::map(&italic)?
        };
        let italic = Face::parse(&italic, 0).unwrap();
        let bold = File::open("adobe-garamond/AGaramondPro-Bold.otf")?;
        let bold = unsafe {
            Mmap::map(&bold)?
        };
        let bold = Face::parse(&bold, 0).unwrap();
        let mut text = VertexTextRenderer::new([("regular", regular), ("italic", italic), ("bold", bold)], 0.07);
        self.rendered_text = text.render(
            &[
                text_segment("To AV moi @ 2 gå ", "italic"),
                text_segment("this is bold ", "bold"),
                text_segment("this is regular", "regular"),
            ],
            true,
            5.0
        ).unwrap();
        renderer.edit_resources(|r| {
            self.vertex_shader = r.create_shader(
                "#version 450

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
                }
                ",
                "vertex shader",
                ShaderStage::Vertex,
            )?;
            self.fragment_shader = r.create_shader(
                "#version 450

                layout(location = 0) out vec4 out_color;

                void main() {
                    float alpha = 1.0;
                    out_color = vec4(1.0, 1.0, 1.0, alpha);
                }
                ",
                "fragment shader",
                ShaderStage::Fragment,
            )?;
            self.pipeline_layout = r.create_pipeline_layout(
                [self.vertex_shader, self.fragment_shader],
            )?;
            self.color_format = r.supported_image_format(
                &[ColorFormat::SrgbRGBA8, ColorFormat::UnormRGBA8],
                FormatFeature::ColorAttachment | FormatFeature::SampledImage,
            ).unwrap();
            let mut pipeline_info = GraphicsPipelineInfo::new(self.pipeline_layout);
            pipeline_info
                .with_vertex_input_binding(VertexInputBinding::new::<0, Vertex>(0, VertexInputRate::Vertex))
                .with_vertex_input_binding(VertexInputBinding::new::<1, VertexOffset>(1, VertexInputRate::Instance))
                .with_sample_shading(SampleShadingInfo::new(MSAA::X8, 0.2, false, false))
                .with_color_output(
                    self.color_format,
                    WriteMask::all(),
                    Some(ColorOutputBlendState {
                        src_color_blend_factor: BlendFactor::SrcAlpha,
                        dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
                        color_blend_op: BlendOp::Add,
                        src_alpha_blend_factor: BlendFactor::One,
                        dst_alpha_blend_factor: BlendFactor::Zero,
                        alpha_blend_op: BlendOp::Add,
                    }),
                );
            r.create_graphics_pipelines(
                &[pipeline_info],
                None,
                &nox::mem::GLOBAL_ALLOC,
                |_, p| { self.pipeline = p }
            )?;
            for text in &self.rendered_text {
                self.vertex_buffers.push(r.create_buffer(
                    (text.trigs.vertices.len() * size_of!(Vertex)) as u64, 
                    &[BufferUsage::VertexBuffer, BufferUsage::TransferDst],
                    &mut r.default_binder(),
                )?);
                self.vertex_offset_buffers.push(r.create_buffer(
                    (text.offsets.len() * size_of!(VertexOffset)) as u64, 
                    &[BufferUsage::VertexBuffer, BufferUsage::TransferDst],
                    &mut r.default_binder(),
                )?);
                self.index_buffers.push(r.create_buffer(
                    (text.trigs.indices.len() * size_of!(u32)) as u64,
                    &[BufferUsage::IndexBuffer, BufferUsage::TransferDst],
                    &mut r.default_binder(),
                )?);
            }
            Ok(())
        })?;
        renderer
            .transfer_requests()
            .add_request(1 << 16);
        Ok(())
    }

    fn transfer_commands(
        &mut self,
        _id: renderer::CommandRequestId,
        commands: &mut renderer::TransferCommands,
    ) -> Result<Option<std::thread::JoinHandle<()>>, Error> {
        for (i, text) in self.rendered_text.iter().enumerate() {
            let vertices = unsafe { slice_as_bytes(&text.trigs.vertices) }.unwrap();
            commands.copy_data_to_buffer(
                self.vertex_buffers[i],
                vertices, 0, vertices.len() as u64,
            )?;
            let offsets = unsafe { slice_as_bytes(&text.offsets) }.unwrap();
            commands.copy_data_to_buffer(
                self.vertex_offset_buffers[i],
                offsets, 0, offsets.len() as u64
            )?;
            let indices = unsafe { slice_as_bytes(&text.trigs.indices) }.unwrap();
            commands.copy_data_to_buffer(
                self.index_buffers[i],
                indices, 0, indices.len() as u64
            )?;
        }
        Ok(None)
    }

    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn frame_graph::FrameGraphInit,
        pending_transfers: &[renderer::CommandRequestId],
    ) -> Result<(), Error> {
        if !pending_transfers.is_empty() {
            return Ok(())
        }
        let frame_graph = frame_graph.init(1)?;
        self.frame_buffer_size = frame_graph.frame_buffer_size();
        let color_output = frame_graph.add_transient_image(&mut |builder| {
            builder
                .with_samples(MSAA::X8)
                .with_usage(ImageUsage::ColorAttachment)
                .with_format(self.color_format, false)
                .with_dimensions(self.frame_buffer_size);
        })?;
        let color_image_resolve = frame_graph.add_transient_image(&mut |builder| {
            builder
                .with_usage(ImageUsage::ColorAttachment)
                .with_usage(ImageUsage::Sampled)
                .with_format(self.color_format, false)
                .with_dimensions(self.frame_buffer_size);
        })?;
        frame_graph.set_render_image(color_image_resolve, None)?;
        frame_graph.add_pass(
            frame_graph::PassInfo {
                max_color_writes: 1,
                msaa_samples: MSAA::X8,
                ..Default::default()
            },
            &mut |builder| {
                builder
                    .with_write(WriteInfo::new(
                        color_output, None,
                        Some((color_image_resolve, ResolveMode::Average)), None,
                        AttachmentLoadOp::Clear,
                        AttachmentStoreOp::Store,
                        ClearValue::Color([0.0, 0.0, 0.0, 1.0].into()),
                    ));
            },
        )?;
        Ok(())
    }

    fn render_commands(
        &mut self,
        _pass_id: frame_graph::PassId,
        commands: &mut renderer::RenderCommands,
    ) -> Result<(), Error> {
        commands.bind_pipeline(self.pipeline)?;
        struct PC {
            _text_width: f32,
            _font_height: f32,
            _text_rows: u32,
            _aspect_ratio: f32,
        }
        let pc = PC {
            _text_width: self.rendered_text.text_width,
            _font_height: self.rendered_text.font_height,
            _text_rows: self.rendered_text.text_rows,
            _aspect_ratio: self.frame_buffer_size.width as f32 / self.frame_buffer_size.height as f32,
        };
        commands.push_constants(|_| {
            unsafe { value_as_bytes(&pc) }.unwrap()
        })?;
        for (i, text) in self.rendered_text.iter().enumerate() {
            commands.draw_indexed(
                DrawInfo {
                    index_count: text.trigs.indices.len() as u32,
                    instance_count: text.offsets.len() as u32,
                    ..Default::default()
                },
                [
                    DrawBufferInfo::new(self.vertex_buffers[i], 0),
                    DrawBufferInfo::new(self.vertex_offset_buffers[i], 0),
                ].into(),
                DrawBufferInfo::new(self.index_buffers[i], 0)
            )?;
        }
        Ok(())
    }
}

fn main() {
    let app = App::default();
    let mut memory = Default::default();
    Nox::new(app, &mut memory).run();
}
