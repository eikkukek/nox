use nox::{
    mem::{
        size_of, value_as_bytes,
        vec_types::{GlobalVec, Vector, Pointer}
    },
    *
};

use nox_geom::{
    earcut::{earcut, earcut_hole},
    shapes::*,
};

#[repr(C)]
#[derive(VertexInput)]
struct Vertex {
    pos: [f32; 2],
}

#[derive(Default)]
pub struct Example {
    vertices: GlobalVec<[f32; 2]>,
    indices: GlobalVec<u32>,
    points: GlobalVec<[f32; 2]>,
    hole: GlobalVec<[f32; 2]>,
    vertex_buffer: BufferId,
    vertex_buffer_map: *mut Vertex,
    index_buffer: BufferId,
    index_buffer_map: *mut u32,
    vertex_shader: ShaderId,
    fragment_shader: ShaderId,
    pipeline: GraphicsPipelineId,
    pipeline_layout: PipelineLayoutId,
    output_format: ColorFormat,
    output: ImageId,
    output_resolve: ImageId,
    frame_buffer_size: Dimensions,
    rects: [Rect; 5],
    target_rect: usize,
    current_rect: Rect,
}

impl Interface for Example {

    fn init_settings(&self) -> InitSettings {
        InitSettings {
            app_name: array_string!("shapes"),
            app_version: Default::default(),
            window_size: [540, 540],
            enable_vulkan_validation: true,
        }
    }

    fn init_callback(
        &mut self,
        _: &mut Nox<Self>,
        renderer: &mut RendererContext,
    ) -> Result<(), Error>
    {
        self.rects[0] = rect([-0.25, -0.25], [0.25, 0.25], 0.1);
        self.rects[1] = self.rects[0].translate(0.0, -0.4).widen(0.125);
        self.rects[2] = self.rects[0].translate(0.0, 0.4).widen(0.125);
        self.rects[3] = self.rects[0].translate(-0.4, 0.0).heighten(0.125);
        self.rects[4] = self.rects[0].translate(0.4, 0.0).heighten(0.125);
        self.target_rect = 0;
        self.current_rect = self.rects[0];
        self.rects[0].to_points_cw(&mut |p| { self.hole.push(p.into()); });
        self.points = GlobalVec::with_capacity(self.hole.len());
        outline_points(&self.hole, 0.075, true, &mut |p| { self.points.push(p.into()); });
        let (vertices, indices) = earcut(&self.points, &[earcut_hole(&self.hole, true)], false).unwrap();
        self.vertices = vertices;
        self.indices.append_map(&indices, |&i| i as u32);
        renderer
            .transfer_requests()
            .add_request(1 << 16);
        renderer.edit_resources(|r| {
            self.vertex_buffer = r.create_buffer(
                (self.vertices.len() * size_of!(Vertex)) as u64 * 4,
                &[BufferUsage::TransferDst, BufferUsage::VertexBuffer],
                &mut r.default_binder_mappable(),
            )?;
            self.vertex_buffer_map = unsafe {
                r.map_buffer(self.vertex_buffer).unwrap().cast::<Vertex>().as_ptr()
            };
            self.index_buffer = r.create_buffer(
                (self.indices.len() * size_of!(u32)) as u64 * 4,
                &[BufferUsage::TransferDst, BufferUsage::IndexBuffer],
                &mut r.default_binder_mappable(),
            )?;
            self.index_buffer_map = unsafe {
                r.map_buffer(self.index_buffer).unwrap().cast::<u32>().as_ptr()
            };
            unsafe {
                Pointer::new(self.vertices.as_mut_ptr()).unwrap().cast::<Vertex>()
                    .copy_to_nonoverlapping(*Pointer::new(self.vertex_buffer_map).unwrap(), self.vertices.len());
                Pointer::new(self.indices.as_mut_ptr()).unwrap().cast::<u32>()
                    .copy_to_nonoverlapping(*Pointer::new(self.index_buffer_map).unwrap(), self.indices.len());
            }
            self.vertex_shader = r.create_shader(
                "#version 450

                    layout(location = 0) in vec2 in_pos;

                    layout(push_constant) uniform PushConstant {
                        float aspect_ratio;
                    } pc;

                    void main() {
                        vec2 pos = in_pos;
                        pos.x *= 1.0 / pc.aspect_ratio;
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
                        out_color = vec4(1.0);
                    }
                ",
                "fragment shader",
                ShaderStage::Fragment,
            )?;
            self.pipeline_layout = r.create_pipeline_layout(
                [self.vertex_shader, self.fragment_shader],
            )?;
            self.output_format = r.supported_image_format(
                &[ColorFormat::SrgbRGBA8, ColorFormat::UnormRGBA8],
                FormatFeature::SampledImage | FormatFeature::ColorAttachment,
            ).unwrap();
            let mut pipeline_info = GraphicsPipelineInfo::new(self.pipeline_layout);
            pipeline_info
                .with_vertex_input_binding(VertexInputBinding::new::<0, Vertex>(0, VertexInputRate::Vertex))
                .with_color_output(self.output_format, Default::default(), None)
                .with_sample_shading(SampleShadingInfo::new(MSAA::X4, 0.2, false, false));
            r.create_graphics_pipelines(&[pipeline_info], None, &GLOBAL_ALLOC, |_, p| self.pipeline = p)?;
            Ok(())
        })?;
        Ok(())
    }

    fn frame_buffer_size_callback(
        &mut self,
        renderer: &mut RendererContext
    ) -> Result<(), Error> {
        self.frame_buffer_size = renderer.frame_buffer_size();
        renderer.edit_resources(|v| {
            v.destroy_image(self.output);
            v.destroy_image(self.output_resolve);
            self.output = v.create_image(
                &mut v.default_binder(),
                |builder| {
                    builder
                        .with_dimensions(self.frame_buffer_size)
                        .with_format(self.output_format, false)
                        .with_samples(MSAA::X4)
                        .with_usage(ImageUsage::ColorAttachment);
                },
            )?;
            self.output_resolve = v.create_image(
                &mut v.default_binder(),
                |builder| {
                    builder
                        .with_dimensions(self.frame_buffer_size)
                        .with_format(self.output_format, false)
                        .with_usage(ImageUsage::Sampled)
                        .with_usage(ImageUsage::ColorAttachment);
                },
            )?;
            Ok(())
        })?;
        Ok(())
    }

    fn update(
        &mut self,
        nox: &mut Nox<Self>,
        renderer: &mut RendererContext,
    ) -> Result<(), Error> {
        if nox.is_key_pressed(KeyCode::Space) {
            self.target_rect = 0;
        } else if nox.is_key_pressed(KeyCode::KeyW) {
            self.target_rect = 1
        } else if nox.is_key_pressed(KeyCode::KeyS) {
            self.target_rect = 2;
        } else if nox.is_key_pressed(KeyCode::KeyA) {
            self.target_rect = 3;
        } else if nox.is_key_pressed(KeyCode::KeyD) {
            self.target_rect = 4;
        } 
        let target_rect = self.rects[self.target_rect];
        if !self.current_rect.eq_epsilon(&target_rect, 1.0e-6) {

            self.points.clear();
            self.vertices.clear();
            self.indices.clear();
            self.hole.clear();

            self.current_rect = self.current_rect.lerp(target_rect, 10.0 * nox.delta_time().as_secs_f32());

            self.current_rect.to_points_cw(&mut |p| { self.hole.push(p.into()); });
            self.points = GlobalVec::with_capacity(self.hole.len());
            outline_points(&self.hole, 0.075, true, &mut |p| { self.points.push(p.into()); });
            let (vertices, indices) = earcut(&self.points, &[earcut_hole(&self.hole, true)], false).unwrap();
            self.vertices = vertices;
            self.indices.append_map(&indices, |&i| i as u32);

            if renderer.buffer_size(self.vertex_buffer).unwrap() <
                (self.vertices.len() * size_of!(Vertex)) as u64
            {
                return Err(Error::UserError("vertex buffer capacity exceeded".into()))
            }
            if renderer.buffer_size(self.index_buffer).unwrap() <
                (self.indices.len() * size_of!(u32)) as u64
            {
                return Err(Error::UserError("index buffer capacity exceeded".into()))
            }
            unsafe {
                Pointer::new(self.vertices.as_mut_ptr()).unwrap().cast::<Vertex>()
                    .copy_to_nonoverlapping(*Pointer::new(self.vertex_buffer_map).unwrap(), self.vertices.len());
                Pointer::new(self.indices.as_mut_ptr()).unwrap().cast::<u32>()
                    .copy_to_nonoverlapping(*Pointer::new(self.index_buffer_map).unwrap(), self.indices.len());
            }
        }
        Ok(())
    }

    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn frame_graph::FrameGraphInit,
        pending_transfers: &[CommandRequestId],
    ) -> Result<(), Error>
    {
        if !pending_transfers.is_empty() {
            return Ok(())
        }
        let frame_graph = frame_graph.init(1)?;
        self.frame_buffer_size = frame_graph.frame_buffer_size();
        let output = frame_graph.add_image(self.output)?;
        let output_resolve = frame_graph.add_image(self.output_resolve)?;
        frame_graph.add_pass(
            PassInfo { max_color_writes: 1, msaa_samples: MSAA::X4, ..Default::default() },
            &mut |builder| {
                builder
                    .with_write(WriteInfo {
                        main_id: output,
                        resolve: Some((output_resolve, ResolveMode::Average)),
                        load_op: AttachmentLoadOp::Clear,
                        store_op: AttachmentStoreOp::Store,
                        ..Default::default()
                    });
            }
        )?;
        frame_graph.set_render_image(output_resolve, None)?;
        Ok(())
    }

    fn transfer_commands(
        &mut self,
        _: CommandRequestId,
        _: &mut TransferCommands,
    ) -> Result<Option<std::thread::JoinHandle<()>>, Error>
    {
        Ok(None)
    }

    fn render_commands(
        &mut self,
        _: PassId,
        commands: &mut RenderCommands,
    ) -> Result<(), Error>
    {
        let aspect_ratio = self.frame_buffer_size.width as f32 / self.frame_buffer_size.height as f32;
        commands.bind_pipeline(self.pipeline)?;
        commands.push_constants(|_| unsafe { value_as_bytes(&aspect_ratio).unwrap() })?;
        commands.draw_indexed(
            DrawInfo {
                index_count: self.indices.len() as u32,
                index_type: IndexType::U32,
                instance_count: 1,
                ..Default::default()
            },
            [DrawBufferInfo::new(self.vertex_buffer, 0)],
            DrawBufferInfo::new(self.index_buffer, 0),
        )?;
        Ok(())
    }
}

fn main() {
    let example = Example::default();
    Nox::new(example, &mut Default::default()).run();
}
