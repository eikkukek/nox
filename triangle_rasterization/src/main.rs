use nox::{
    mem::{slice_as_bytes, vec_types::{GlobalVec, Vector}},
    *,
};

use nox_geom::{
    *,
    fn_2d::*,
};

#[derive(Default)]
struct App {
    image_data: Vec<u32>,
    image: ImageId,
    image_dim: Dimensions,
    sampler: SamplerId,
    pipeline_layout: PipelineLayoutId,
    pipeline: GraphicsPipelineId,
    resource: ShaderResourceId,
    output_format: ColorFormat,
    read_format: ColorFormat,
}

fn plot_line(
    start: Vec2,
    end: Vec2,
    dim: (usize, usize),
    data: &mut [u32],
    color: u32,
) {
    let le = color.to_le_bytes();
    let mut plot = |x: f32, y: f32, c: f32| {
        let mut le = le;
        le[3] = (le[3] as f32 * c) as u8;
        let index = y as usize * dim.0 + x as usize;
        if index < data.len() {
            data[index] = u32::from_le_bytes(le);
        }
    };
    let mut x0 = start.x;
    let mut x1 = end.x;
    let mut y0 = start.y;
    let mut y1 = end.y;
    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    if steep {
        let tmp = x0;
        x0 = y0;
        y0 = tmp;
        let tmp = x1;
        x1 = y1;
        y1 = tmp;
    }
    if x0 > x1 {
        let tmp = x0;
        x0 = x1;
        x1 = tmp;
        let tmp = y0;
        y0 = y1;
        y1 = tmp;
    }

    let dx = x1 - x0;
    let dy = y1 - y0;

    let gradient = 
        if dx == 0.0 {
            1.0
        } else {
            dy / dx
        };

    let x_end = x0.floor();
    let y_end = y0 + gradient * (x_end - x0);
    let x_gap = 1.0 - (x0 - x_end);
    let num_1_x = x_end;
    let num_1_y = y_end.floor();
    if steep {
        plot(num_1_y, num_1_x, (1.0 - y_end.fract()) * x_gap);
        plot(num_1_y + 1.0, num_1_x, y_end.fract() * x_gap);
    } else {
        plot(num_1_x, num_1_y, (1.0 - y_end.fract()) * x_gap);
        plot(num_1_x, num_1_y + 1.0, y_end.fract() * x_gap);
    }
    let mut inter_y = y_end + gradient;

    let x_end = x1.ceil();
    let y_end = y1 + gradient * (x_end - x1);
    let x_gap = 1.0 - (x_end - x1);
    let num_2_x = x_end;
    let num_2_y = y_end.floor();
    if steep {
        plot(num_2_y, num_2_x, (1.0 - y_end.fract()) * x_gap);
        plot(num_2_y + 1.0, num_2_x, y_end.fract() * x_gap);
    } else {
        plot(num_2_x, num_2_y, (1.0 - y_end.fract()) * x_gap);
        plot(num_2_x, num_2_y + 1.0, y_end.fract() * x_gap);
    }

    if steep {
        let mut x = num_1_x;
        while x < num_2_x {
            let floor = inter_y.floor();
            let fract = inter_y.fract();
            plot(floor, x, 1.0 - fract);
            plot(floor + 1.0, x, fract);
            inter_y = inter_y + gradient;
            x += 1.0;
        }
    } else {
        let mut x = num_1_x;
        while x < num_2_x {
            let floor = inter_y.floor();
            let fract = inter_y.fract();
            plot(x, floor, 1.0 - fract);
            plot(x, floor + 1.0, fract);
            inter_y = inter_y + gradient;
            x += 1.0;
        }
    }
}

fn fill_triangle(
    a: Vec2,
    b: Vec2,
    c: Vec2,
    dim: (usize, usize),
    data: &mut [u32],
    color: u32,
    _stack: &mut GlobalVec<(f32, f32, f32, f32)>,
) {
    let mut stack = std::collections::vec_deque::VecDeque::new();
    let mut plot = |x: f32, y: f32| {
        let index = y as usize * dim.0 + x as usize;
        if index < data.len() {
            data[index] = color;
        }
    };
    let centroid = centroid(a, b, c);
    if !point_in_triangle(a, b, c, centroid) {
        panic!()
    }
    stack.push_front((centroid.x, centroid.x, centroid.y, 1.0));
    stack.push_front((centroid.x, centroid.x, centroid.y - 1.0, -1.0));
    while let Some((mut x1, x2, y, dy)) = stack.pop_back() {
        let mut x = x1;
        if point_in_triangle(a, b, c, vec2(x, y)) {
            while point_in_triangle(a, b, c, vec2(x - 1.0, y)) {
                plot(x - 1.0, y);
                x -= 1.0;
            }
            if x < x1 {
                stack.push_front((x, x1 - 1.0, y - dy, - dy));
            }
        }
        while x1 <= x2 {
            while point_in_triangle(a, b, c, vec2(x1, y)) {
                plot(x1, y);
                x1 = x1 + 1.0;
            }
            if x1 > x {
                stack.push_front((x, x1 - 1.0, y + dy, dy));
            }
            if x1 - 1.0 > x2 {
                stack.push_front((x2 + 1.0, x1 - 1.0, y - dy, -dy));
            }
            x1 = x1 + 1.0;
            while x1 < x2 && !point_in_triangle(a, b, c, vec2(x1, y)) {
                x1 = x1 + 1.0;
            }
            x = x1;
        }
    }
}

impl Interface for App {

    fn init_settings(&self) -> InitSettings {
        InitSettings::new(
            "Test",
            Default::default(),
            [540, 540],
            true,
            true,
        )
    }

    fn frame_buffer_size_callback(
        &mut self,
        renderer: &mut RendererContext
    ) -> Result<(), Error> {
        let dim = renderer.frame_buffer_size();
        self.image_dim = dim;
        renderer.edit_resources(|r| {
            r.free_shader_resources(&[self.resource], &GlobalAlloc)?;
            r.destroy_image(self.image);
            self.image = r.create_image(&mut r.default_binder(), |builder| {
                builder
                    .with_dimensions(dim)
                    .with_format(self.read_format, false)
                    .with_usage(ImageUsage::Sampled)
                    .with_usage(ImageUsage::TransferDst);
            })?;
            r.allocate_shader_resources(
                &[ShaderResourceInfo {
                    layout_id: self.pipeline_layout,
                    set: 0
                }], |_, r| self.resource = r,
                &GlobalAlloc
            )?;
            r.update_shader_resources(&[
                ShaderResourceImageUpdate {
                    resource: self.resource,
                    binding: 0,
                    starting_index: 0,
                    infos: &[
                        ShaderResourceImageInfo {
                            sampler: self.sampler,
                            image_source: (self.image, None),
                            storage_image: false,
                        }
                    ],
                },
            ], &[], &[], &GlobalAlloc)?;
            Ok(())
        })?;
        renderer
            .transfer_requests()
            .add_request(1 << 26);
        Ok(())
    }

    fn init_callback(
        &mut self,
        _nox: &mut Nox<Self>,
        renderer: &mut RendererContext,
    ) -> Result<(), Error> {
        renderer
            .edit_resources(|r| {
                self.output_format = r.supported_image_format(
                    &[ColorFormat::SrgbRGBA8],
                    FormatFeature::SampledImage | FormatFeature::ColorAttachment,
                ).unwrap();
                self.read_format = r.supported_image_format(
                    &[ColorFormat::SrgbRGBA8],
                    FormatFeature::SampledImage | FormatFeature::TransferDst
                ).unwrap();
                let vertex_shader = r.create_shader("
                        #version 450

                        layout(location = 0) out vec2 out_uv;

                        vec2 positions[6] = vec2[](
                            vec2(1.0, 1.0),
                            vec2(-1.0, 1.0),
                            vec2(-1.0, -1.0),
                            vec2(1.0, -1.0),
                            vec2(1.0, 1.0),
                            vec2(-1.0, -1.0)
                        );

                        vec2 uvs[6] = vec2[](
                            vec2(1.0, 1.0),
                            vec2(0.0, 1.0),
                            vec2(0.0, 0.0),
                            vec2(1.0, 0.0),
                            vec2(1.0, 1.0),
                            vec2(0.0, 0.0)
                        );

                        void main() {
                            int vertex_index = gl_VertexIndex;
                            out_uv = uvs[vertex_index];
                            gl_Position = vec4(positions[vertex_index], 0.0, 1.0);
                        }
                        ",
                        "vertex shader",
                        ShaderStage::Vertex
                )?;
                let fragment_shader = r.create_shader("
                    #version 450
                    layout(location = 0) in vec2 in_uv;
                    
                    layout(location = 0) out vec4 out_color;

                    layout(set = 0, binding = 0) uniform sampler2D render_image;

                    void main() {
                        out_color = texture(render_image, in_uv);
                    }
                    ",
                    "fragment shader",
                    ShaderStage::Fragment
                )?;
                self.pipeline_layout = r.create_pipeline_layout(
                    [
                        vertex_shader,
                        fragment_shader,
                    ]
                )?;
                self.sampler = r.create_sampler(|_| {})?;
                let mut pipeline_info = GraphicsPipelineInfo::new(self.pipeline_layout);
                pipeline_info
                    .with_color_output(self.read_format, Default::default(),
                        Some(ColorOutputBlendState {
                            src_color_blend_factor: BlendFactor::SrcAlpha,
                            dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
                            color_blend_op: BlendOp::Add,
                            src_alpha_blend_factor: BlendFactor::One,
                            dst_alpha_blend_factor: BlendFactor::OneMinusSrcAlpha,
                            alpha_blend_op: BlendOp::Add,
                        })
                    );
                r.create_graphics_pipelines(&[pipeline_info], None, &GlobalAlloc, |_, p| {
                    self.pipeline = p;
                })?;
                Ok(())
        })?;
        Ok(())
    }

    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn FrameGraphInit,
        pending_transfers: &[CommandRequestId],
    ) -> Result<(), Error> {
        if !pending_transfers.is_empty() {
            return Ok(())
        }
        let frame_graph = frame_graph.init(1)?;
        let size = frame_graph.frame_buffer_size();
        let output_image = frame_graph.add_transient_image(&mut |image| {
            image
                .with_usage(ImageUsage::Sampled)
                .with_usage(ImageUsage::ColorAttachment)
                .with_format(self.output_format, false)
                .with_dimensions(size);
        })?;
        let read_image = frame_graph.add_image(self.image)?;
        frame_graph.add_pass(
            PassInfo {
                max_reads: 1,
                max_color_writes: 1,
                msaa_samples: MSAA::X1,
            },
            &mut |pass| {
                pass
                    .with_read(
                        ReadInfo::new(read_image, None)
                    )
                    .with_write(WriteInfo::new(
                        output_image, None,
                        Default::default(), None,
                        AttachmentLoadOp::Clear,
                        AttachmentStoreOp::Store,
                        ClearValue::Color(Default::default())
                    ));
            }
        )?;
        frame_graph.set_render_image(output_image, None)?;
        Ok(())
    }

    fn render_commands(
        &mut self,
        _pass_id: PassId,
        commands: &mut RenderCommands,
    ) -> Result<(), Error> {
        commands.bind_pipeline(self.pipeline)?;
        commands.bind_shader_resources(|_set| {
            self.resource
        })?;
        commands.draw_bufferless(6, 1);
        Ok(())
    }

    fn transfer_commands(
        &mut self,
        _id: CommandRequestId,
        commands: &mut TransferCommands,
    ) -> Result<Option<std::thread::JoinHandle<()>>, Error> {
        self.image_data.clear();
        self.image_data.resize(self.image_dim.width as usize * self.image_dim.height as usize, 0);
        let a = vec2(self.image_dim.width as f32, 0.0);
        let b = vec2(0.0, 0.0);
        let c = vec2(0.0, self.image_dim.height as f32);
        plot_line(a, b, (self.image_dim.width as usize, self.image_dim.height as usize), &mut self.image_data, u32::MAX);
        plot_line(b, c, (self.image_dim.width as usize, self.image_dim.height as usize), &mut self.image_data, u32::MAX);
        plot_line(c, a, (self.image_dim.width as usize, self.image_dim.height as usize), &mut self.image_data, u32::MAX);
        fill_triangle(a, b, c, (self.image_dim.width as usize, self.image_dim.height as usize), &mut self.image_data, u32::MAX, &mut GlobalVec::new());
        commands.copy_data_to_image(
            self.image,
            unsafe { slice_as_bytes(&self.image_data).unwrap() },
            None,
            None,
            None,
        )?;
        Ok(None)
    }
}

fn main() {
    let app = App::default();
    Nox::new(app, &mut Default::default()).run();
}
