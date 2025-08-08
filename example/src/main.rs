use std::{path::PathBuf, ptr::NonNull};

use nox::{
    interface::Interface,
    mem::{GLOBAL_ALLOC, size_of, slice_as_bytes},
    renderer::{
        image::*,
        pipeline::*,
        frame_graph::*,
        *
    },
    InitSettings,
    Memory,
    Nox,
    Version,
};

#[repr(C)]
#[derive(VertexInput)]
struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {

    const fn new(
        pos: [f32; 3],
        normal: [f32; 3],
        uv: [f32; 2],
    ) -> Self
    {
        Self {
            pos,
            normal,
            uv,
        }
    }
}

const CUBE_VERTICES: &[Vertex] = &[

    Vertex::new([0.5, -0.5, -0.5], [1.0, 0.0, 0.0], [0.0, 0.0]),
    Vertex::new([0.5, -0.5, 0.5], [1.0, 0.0, 0.0], [1.0, 0.0]),
    Vertex::new([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], [1.0, 1.0]),
    Vertex::new([0.5, 0.5, -0.5], [1.0, 0.0, 0.0], [0.0, 1.0]),

    Vertex::new([-0.5, -0.5, 0.5], [-1.0, 0.0, 0.0], [0.0, 0.0]),
    Vertex::new([-0.5, -0.5, -0.5], [-1.0, 0.0, 0.0], [1.0, 0.0]),
    Vertex::new([-0.5, 0.5, -0.5], [-1.0, 0.0, 0.0], [1.0, 1.0]),
    Vertex::new([-0.5, 0.5, 0.5], [-1.0, 0.0, 0.0], [0.0, 1.0]),

    Vertex::new([-0.5, 0.5, -0.5], [0.0, 1.0, 0.0], [0.0, 0.0]),
    Vertex::new([0.5, 0.5, -0.5], [0.0, 1.0, 0.0], [1.0, 0.0]),
    Vertex::new([0.5, 0.5, 0.5], [0.0, 1.0, 0.0], [1.0, 1.0]),
    Vertex::new([-0.5, 0.5, 0.5], [0.0, 1.0, 0.0], [0.0, 1.0]),

    Vertex::new([-0.5, -0.5, 0.5], [0.0, -1.0, 0.0], [0.0, 0.0]),
    Vertex::new([0.5, -0.5, 0.5], [0.0, -1.0, 0.0], [1.0, 0.0]),
    Vertex::new([0.5, -0.5, -0.5], [0.0, -1.0, 0.0], [1.0, 1.0]),
    Vertex::new([-0.5, -0.5, -0.5], [0.0, -1.0, 0.0], [0.0, 1.0]),

    Vertex::new([0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 0.0]),
    Vertex::new([-0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 0.0]),
    Vertex::new([-0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 1.0]),
    Vertex::new([0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 1.0]),

    Vertex::new([-0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 0.0]),
    Vertex::new([0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 0.0]),
    Vertex::new([0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 1.0]),
    Vertex::new([-0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 1.0]),
];

const CUBE_INDICES: &[u32] = &[
    0, 1, 2, 0, 2, 3,
    4, 5,  6, 4, 6, 7,
    8, 9, 10, 8, 10, 11,
    12, 13, 14, 12, 14, 15,
    16, 17, 18, 16, 18, 19,
    20, 21, 22, 20, 22, 23,
];

#[allow(dead_code)]
struct Matrices {
    model: [f32; 16],
    projection: [f32; 16],
}

struct Quat {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Quat {

    fn angle_axis(angle: f32, axis: [f32; 3]) -> Self {
        let half = angle / 2.0;
        let sin = half.sin();
        let mag = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        Self {
            x: axis[0] / mag * sin,
            y: axis[1] / mag * sin,
            z: axis[2] / mag * sin,
            w: half.cos(),
        }
    }

    fn to_matrix(self) -> [f32; 16] {
        let num1 = self.x * self.x;
        let num2 = self.y * self.y;
        let num3 = self.z * self.z;
        return [
            1.0 - 2.0 * (num2 + num3), 2.0 * (self.x * self.y + self.z * self.w), 2.0 * (self.x * self.z - self.y * self.w), 0.0,
            2.0 * (self.x * self.y - self.z * self.w), 1.0 - 2.0 * (num1 + num3), 2.0 * (self.y * self.z + self.x * self.w), 0.0,
            2.0 * (self.x * self.z + self.y * self.w), 2.0 * (self.y * self.z - self.x * self.w), 1.0 - 2.0 * (num1 + num2), 0.0,
            0.0, 0.0, 1.0, 1.0,
        ];
    }
}

fn mat4_project(fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> [f32; 16] {
    let half_tan = (fov / 2.0).tan();
    [
        1.0 / (aspect_ratio * half_tan), 0.0, 0.0 ,0.0,
        0.0, 1.0 / half_tan, 0.0, 0.0,
        0.0, 0.0, (z_far - z_near) / (z_far + z_near), 1.0,
        0.0, 0.0, (-2.0 * z_far * z_near) / (z_far + z_near), 1.0,
    ]
}

struct App {
    image: Option<NonNull<u8>>,
    color_format: ColorFormat,
    depth_format: DepthFormat,
    image_id: ImageID,
    sampler_id: SamplerID,
    vertex_shader_id: ShaderID,
    fragment_shader_id: ShaderID,
    pipeline_layout_id: PipelineLayoutID,
    pipeline_id: GraphicsPipelineID,
    vertex_buffer_id: BufferID,
    index_buffer_id: BufferID,
    uniform_buffer_id: BufferID,
    shader_resource_id: ShaderResourceID,
    matrices_map: NonNull<Matrices>,
    rot: f32,
}

unsafe impl Send for App {}
unsafe impl Sync for App {}

impl App {

    fn new() -> Self {
        Self {
            image: None,
            color_format: ColorFormat::SrgbRGBA8,
            depth_format: DepthFormat::D32,
            image_id: Default::default(),
            sampler_id: Default::default(),
            vertex_shader_id: Default::default(),
            fragment_shader_id: Default::default(),
            pipeline_layout_id: Default::default(),
            pipeline_id: Default::default(),
            vertex_buffer_id: Default::default(),
            index_buffer_id: Default::default(),
            uniform_buffer_id: Default::default(),
            shader_resource_id: Default::default(),
            matrices_map: NonNull::dangling(),
            rot: 0.0,
        }
    }
}

impl Interface for App {

    fn init_settings(&self) -> InitSettings {
        InitSettings::new("Test", Version::default(), [540, 540], false)
    }

    fn init_callback(
        &mut self,
        nox: &mut Nox<Self>,
        renderer_context: &mut nox::renderer::RendererContext
    ) -> Result<(), nox::Error>
    {
        println!("GPU: {}", nox.gpu_name());
        let mut path = match std::env::current_exe() {
            Ok(path) => path,
            Err(_) => PathBuf::new(),
        };
        path.pop();
        path.push("../../nightreign.jpg");
        self.image = Some(ImageBuffer::new(path.to_str().unwrap(), 4).unwrap());
        renderer_context.edit_resources(|r| {
            self.vertex_shader_id = r.create_shader(
                "#version 450

                layout(location = 0) in vec3 in_pos;
                layout(location = 1) in vec3 in_normal;
                layout(location = 2) in vec2 in_uv;

                layout(location = 0) out vec3 out_normal;
                layout(location = 1) out vec2 out_uv;

                layout(set = 0, binding = 0) uniform Matrices {
                    mat4 model;
                    mat4 projection;
                } matrices;

                void main() {
                    mat3 normal_matrix = transpose(inverse(mat3(matrices.model)));
                    out_normal = normal_matrix * in_normal;
                    gl_Position = matrices.projection * matrices.model * vec4(in_pos, 1.0);
                    out_uv = in_uv;
                }
                ",
                "vertex shader",
                ShaderStage::Vertex,
            )?;
            self.fragment_shader_id = r.create_shader(
                "#version 450

                layout(location = 0) out vec4 out_color;

                layout(location = 0) in vec3 in_normal;
                layout(location = 1) in vec2 in_uv;

                layout(set = 0, binding = 1) uniform sampler2D tex;
                
                void main() {
                    /*
                    const vec3 light_color = vec3(0.5, 0.5, 0.5);
                    const vec3 light_dir = normalize(vec3(-5.0, -2.0, -5.0));
                    const float diff = max(dot(normalize(in_normal), light_dir), 0.0);
                    vec3 diffuse = diff * light_color;
                    vec4 color = texture(tex, in_uv);
                    out_color = vec4(color.xyz * 0.5f + (diffuse * light_color), 1.0);
                    */
                    out_color = texture(tex, in_uv);
                }
                ",
                "fragment shader",
                ShaderStage::Fragment,
            )?;
            self.pipeline_layout_id = r.create_pipeline_layout(
                [self.vertex_shader_id, self.fragment_shader_id]
            )?;
            self.color_format = r.supported_image_format(
                &[ColorFormat::SrgbRGBA8, ColorFormat::UnormRGBA8],
                FormatFeature::SampledImage | FormatFeature::ColorAttachment,
            ).unwrap();
            self.depth_format = r.supported_image_format(
                DepthFormat::all_depth(),
                FormatFeature::DepthStencilAttachment,
            ).unwrap();
            self.image_id = r.create_image(
                |builder| {
                    builder
                        .with_usage(ImageUsage::TransferDst)
                        .with_usage(ImageUsage::Sampled)
                        .with_dimensions(self.image.as_ref().unwrap().dimensions())
                        .with_format(self.color_format, false);
                },
                &mut r.default_binder())?;
            self.sampler_id = r.create_sampler(
                |_| {},
            )?;
            let mut graphics_pipeline_info = GraphicsPipelineInfo::new(self.pipeline_layout_id);
            graphics_pipeline_info
                .with_color_output(
                    self.color_format,
                    WriteMask::all(),
                    None,
                )
                .with_depth_output(self.depth_format)
                .with_vertex_input_binding(VertexInputBinding::new::<0, Vertex>(0, VertexInputRate::Vertex))
                .with_depth_stencil(DepthStencilInfo {
                    compare_op: CompareOp::Less,
                    depth_bounds: Some(DepthBounds::new(0.0, 1.0)),
                    stencil_test_info: None,
                    write_enable: true,
                    stencil_test_enable: false,
                });
            r.create_graphics_pipelines(
                &[graphics_pipeline_info],
                |_, id| { self.pipeline_id = id; },
                &GLOBAL_ALLOC
            )?;
            self.vertex_buffer_id = r.create_buffer(
                (CUBE_VERTICES.len() * size_of!(Vertex)) as u64,
                &[BufferUsage::VertexBuffer, BufferUsage::TransferDst],
                &mut r.default_binder(),
            )?;
            self.index_buffer_id = r.create_buffer(
                (CUBE_INDICES.len() * size_of!(u32)) as u64,
                &[BufferUsage::IndexBuffer , BufferUsage::TransferDst],
                &mut r.default_binder(),
            )?;
            self.uniform_buffer_id = r.create_buffer(
                size_of!(Matrices) as u64,
                &[BufferUsage::UniformBuffer],
                &mut r.default_binder_mappable(),
            )?;
            self.matrices_map = unsafe {
                r.map_buffer(self.uniform_buffer_id).unwrap().cast::<Matrices>()
            };
            r.allocate_shader_resources(
                &[ShaderResourceInfo {
                    layout_id: self.pipeline_layout_id,
                    set: 0,
                }],
                |_, v| self.shader_resource_id = v,
                &GLOBAL_ALLOC,
            )?;
            r.update_shader_resources(
                &[
                    ShaderResourceImageUpdate {
                        resource: self.shader_resource_id,
                        binding: 1,
                        starting_index: 0,
                        infos: &[ShaderResourceImageInfo {
                            sampler: self.sampler_id,
                            image_source: self.image_id.into(),
                        }]
                    }
                ],
                &[
                    ShaderResourceBufferUpdate {
                        resource: self.shader_resource_id,
                        binding: 0,
                        starting_index: 0,
                        infos: &[ShaderResourceBufferInfo {
                            buffer: self.uniform_buffer_id,
                            offset: 0,
                            size: size_of!(Matrices) as u64,
                        }],
                    }
                ],
                &[],
                &GLOBAL_ALLOC)?;
            Ok(())
        })?;
        renderer_context
            .command_requests()
            .add_transfer_request(TransferRequest::new(1));
        Ok(())
    }

    fn update(&mut self, _nox: &mut Nox<Self>, _renderer_contexts: &mut nox::renderer::RendererContext) {
    }

    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn FrameGraphInit,
        pending_transfers: &[nox::renderer::CommandRequestID]
    ) -> Result<(), nox::renderer::Error>
    {
        if !pending_transfers.is_empty() {
            return Ok(())
        }
        let frame_graph = frame_graph.init(1)?;
        let frame_buffer_size = frame_graph.frame_buffer_size();
        unsafe {
            self.matrices_map.write(
                Matrices {
                    model: Quat::angle_axis(self.rot, [-0.52, 1.0, 0.3]).to_matrix(),
                    projection: mat4_project(
                        90.0,
                        frame_buffer_size.width as f32 / frame_buffer_size.height as f32,
                        0.1,
                        100.0
                    ),
                }
            );
        }
        self.rot += 0.001;
        let color_output = frame_graph.add_transient_image(
            &mut |builder| {
                builder
                    .with_usage(ImageUsage::ColorAttachment)
                    .with_usage(ImageUsage::Sampled)
                    .with_format(self.color_format, false)
                    .with_dimensions(frame_buffer_size);
            }
        )?;
        let texture = frame_graph.add_image(self.image_id.into())?;
        frame_graph.set_render_image(color_output);
        let depth_output = frame_graph.add_transient_image(
            &mut |builder| {
                builder
                    .with_usage(ImageUsage::DepthStencilAttachment)
                    .with_format(self.depth_format, false)
                    .with_dimensions(frame_buffer_size);
            }
        )?;
        frame_graph.add_pass(
            PassInfo { max_color_writes: 1, max_reads: 1, ..Default::default() },
            &mut |builder| {
                builder
                    .with_read(ReadInfo { resource_id: texture, range_info: None })
                    .with_write(WriteInfo::new(
                        color_output,
                        None,
                        AttachmentLoadOp::Clear,
                        AttachmentStoreOp::Store,
                        Default::default())
                    )
                    .with_depth_write(WriteInfo::new(
                        depth_output,
                        None,
                        AttachmentLoadOp::Clear,
                        AttachmentStoreOp::Store,
                        ClearValue::DepthStencil(ClearDepthStencilValue { depth: 1.0, stencil: 0 }),
                    ));
            }
        )?;
        //let id = frame_graph.add_image(self.image_id.into())?;
        //frame_graph.set_render_image(id);
        Ok(())
    }

    fn render_commands(
        &mut self,
        _pass: PassID,
        commands: &mut RenderCommands,
    ) -> Result<(), nox::renderer::Error>
    {
        commands.bind_pipeline(self.pipeline_id)?;
        commands.bind_shader_resources(|_| {
            self.shader_resource_id
        })?;
        commands.draw_indexed(DrawInfo {
                index_count: CUBE_INDICES.len() as u32,
                ..Default::default()
            },
            [DrawBufferInfo::new(self.vertex_buffer_id, 0)].into(),
            DrawBufferInfo::new(self.index_buffer_id, 0),
        )?;
        Ok(())
    }

    fn transfer_commands(
        &mut self,
        _id: nox::renderer::CommandRequestID,
        command_buffer: &mut nox::renderer::TransferCommandbuffer,
    )
    {
        command_buffer.copy_data_to_image(
            self.image_id,
            self.image.as_ref().unwrap(),
            None,
            None,
            None,
        ).unwrap();
        let vertices = unsafe { slice_as_bytes(CUBE_VERTICES) }.unwrap();
        command_buffer.copy_data_to_buffer(
            self.vertex_buffer_id,
            vertices,
            0,
            vertices.len() as u64,
        ).unwrap();
        let indices = unsafe { slice_as_bytes(CUBE_INDICES) }.unwrap();
        command_buffer.copy_data_to_buffer(
            self.index_buffer_id,
            indices,
            0,
            indices.len() as u64,
        ).unwrap();
    }
}

fn main() {
    let app = App::new();
    let mut memory = match Memory::default() {
        Some(r) => r,
        None => {
            eprintln!("failed to create memory");
            return
        }
    };
    if let Some(nox) = Nox::new(app, &mut memory) {
        nox.run();
    }
}
