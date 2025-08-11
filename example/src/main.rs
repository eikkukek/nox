use std::{ffi::CString, path::{Path, PathBuf}, ptr::NonNull};

use core::f32::consts::PI;

use glam::{f32::*, Vec4Swizzles};

use nox::{
    interface::Interface,
    mem::{size_of, slice_as_bytes, vec_types::ArrayVec, GLOBAL_ALLOC},
    renderer::{
        frame_graph::*, image::*, pipeline::*, *
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

#[repr(C)]
#[derive(VertexInput)]
struct VertexOff {
    off: f32,
}

impl VertexOff {

    const fn new(off: f32) -> Self {
        Self {
            off,
        }
    }
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

const CUBE_OFF: &[VertexOff] = &[
    VertexOff::new(0.0),
    VertexOff::new(PI / 4.0),
    VertexOff::new(PI / 2.0),
    VertexOff::new(3.0 * PI / 4.0),
    VertexOff::new(PI),
    VertexOff::new(5.0 * PI / 4.0),
    VertexOff::new(3.0 * PI / 2.0),
    VertexOff::new(7.0 * PI / 4.0),
];

const CUBE_INDICES: &[u32] = &[
    0, 1, 2, 0, 2, 3,
    4, 5,  6, 4, 6, 7,
    8, 9, 10, 8, 10, 11,
    12, 13, 14, 12, 14, 15,
    16, 17, 18, 16, 18, 19,
    20, 21, 22, 20, 22, 23,
];

#[repr(C)]
struct Matrices {
    model: glam::Mat4,
    projection: glam::Mat4,
    view: glam::Mat4,
}

#[repr(C)]
struct LightInfo {
    pos: Vec3,
}

#[derive(Default)]
struct ImageAsset {
    ptr: Option<NonNull<u8>>,
    dim: Dimensions,
    ch: u32,
}

impl ImageAsset {

    fn new(filepath: &Path, channels: u32) -> Result<Self, nox::Error> {
        let mut x = 0;
        let mut y = 0;
        let mut ch = 0;
        let c_string = CString::new(filepath.to_str().unwrap()).unwrap();
        let ptr = unsafe {
            NonNull::new(stb_image::stb_image::stbi_load(
                c_string.as_ptr(),
                &mut x,
                &mut y,
                &mut ch,
                channels as i32,
            ))
        };
        let dim = Dimensions::new(x as u32, y as u32, 1);
        if ptr.is_none() {
            unsafe {
                let err = stb_image::stb_image::stbi_failure_reason();
                return Err(nox::Error::UserError(
                    std::ffi::CStr::from_ptr(err).to_str().unwrap().into()
                ))
            }
        }
        Ok(Self {
            ptr,
            dim,
            ch: channels,
        })
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.ptr.unwrap().as_ptr(),
                (self.dim.width * self.dim.height * self.ch) as usize
            )
        }
    }
}

impl Drop for ImageAsset {

    fn drop(&mut self) {
        if let Some(ptr) = self.ptr {
            unsafe {
                stb_image::stb_image::stbi_image_free(ptr.as_ptr() as _);
            }
        } 
    }
}

struct App {
    assets: [ImageAsset; 8],
    color_format: ColorFormat,
    depth_stencil_format: DepthStencilFormat,
    depth_format: DepthStencilFormat,
    image: ImageID,
    sampler: SamplerID,
    vertex_shader: ShaderID,
    fragment_shader: ShaderID,
    outline_vertex: ShaderID,
    outline_fragment: ShaderID,
    pipeline_layouts: [PipelineLayoutID; 2],
    pipelines: [GraphicsPipelineID; 2],
    outline_layout: PipelineLayoutID,
    vertex_buffer: BufferID,
    vertex_instance_buffer: BufferID,
    index_buffer: BufferID,
    matrices_buffer: BufferID,
    matrices_map: NonNull<Matrices>,
    light_info_buffer: BufferID,
    light_info_map: NonNull<LightInfo>,
    shader_resources: [ShaderResourceID; 2],
    first_pass: PassID,
    second_pass: PassID,
    rot: f32,
}

unsafe impl Send for App {}
unsafe impl Sync for App {}

impl App {

    fn new() -> Self {
        Self {
            assets: Default::default(),
            color_format: ColorFormat::SrgbRGBA8,
            depth_stencil_format: DepthStencilFormat::D32S8,
            depth_format: DepthStencilFormat::D32,
            image: Default::default(),
            sampler: Default::default(),
            vertex_shader: Default::default(),
            fragment_shader: Default::default(),
            outline_vertex: Default::default(),
            outline_fragment: Default::default(),
            outline_layout: Default::default(),
            pipeline_layouts: Default::default(),
            pipelines: Default::default(),
            vertex_buffer: Default::default(),
            vertex_instance_buffer: Default::default(),
            index_buffer: Default::default(),
            matrices_buffer: Default::default(),
            matrices_map: NonNull::dangling(),
            light_info_buffer: Default::default(),
            light_info_map: NonNull::dangling(),
            shader_resources: Default::default(),
            first_pass: Default::default(),
            second_pass: Default::default(),
            rot: 0.0,
        }
    }
}

impl Interface for App {

    fn init_settings(&self) -> InitSettings {
        InitSettings::new("Test", Version::default(), [540, 540], true)
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
        path.push("../..");
        let mut paths = ArrayVec::<PathBuf, 8>::with_len(path, 8).unwrap();
        paths[0].push("rock_035.jpg");
        paths[1].push("rock_051.jpg");
        paths[2].push("rock_058.jpg");
        paths[3].push("onyx.jpg");
        paths[4].push("diamond_plate.jpg");
        paths[5].push("ground.jpg");
        paths[6].push("marble.jpg");
        paths[7].push("wood_floor.jpg");
        for (i, path) in paths.iter().enumerate() {
            self.assets[i] = ImageAsset
                ::new(path, 4)
                .map_err(|e| nox::Error::UserError(format!("failed to open {:?} ( {:?} )", path, e)))?;
        }
        renderer_context.edit_resources(|r| {
            self.vertex_shader = r.create_shader(
                "#version 450

                layout(location = 0) in vec3 in_pos;
                layout(location = 1) in vec3 in_normal;
                layout(location = 2) in vec2 in_uv;

                layout(location = 3) in float in_off;

                layout(location = 0) out vec3 out_normal;
                layout(location = 1) out vec2 out_uv;
                layout(location = 2) out vec3 out_pos;
                layout(location = 3) out flat uint instance_index;

                layout(set = 0, binding = 0) uniform Matrices {
                    mat4 model;
                    mat4 projection;
                    mat4 view;
                } matrices;

                void main() {
                    mat3 normal_matrix = transpose(inverse(mat3(matrices.model)));
                    out_normal = normal_matrix * in_normal;
                    mat4 model = matrices.model;
                    vec4 off = vec4(
                        cos(in_off) * 3.0,
                        sin(in_off) * 3.0,
                        0.0,
                        0.0
                    );
                    model[3] += off;
                    gl_Position = matrices.projection * matrices.view * model * vec4(in_pos, 1.0);
                    out_uv = in_uv;
                    vec4 pos = model * vec4(in_pos, 1.0);
                    out_pos = pos.xyz;
                    instance_index = gl_InstanceIndex;
                }
                ",
                "vertex shader",
                ShaderStage::Vertex,
            )?;
            self.fragment_shader = r.create_shader(
                "#version 450

                layout(location = 0) in vec3 in_normal;
                layout(location = 1) in vec2 in_uv;
                layout(location = 2) in vec3 in_pos;
                layout(location = 3) in flat uint instance_index;

                layout(location = 0) out vec4 out_color;

                layout(set = 1, binding = 0) uniform sampler2DArray tex;
                layout(set = 1, binding = 1) uniform LightInfo {
                    vec3 pos;
                } light_info;
                
                void main() {

                    vec3 light_color = vec3(0.4, 0.4, 0.4);
                    vec3 light_dir = normalize(light_info.pos - in_pos);
                    const float diff = max(dot(normalize(in_normal), light_dir), 0.0);
                    vec3 diffuse = diff * light_color;
                    vec4 color = texture(tex, vec3(in_uv, instance_index));
                    out_color = vec4(color.xyz * 0.8f + (diffuse * light_color), 1.0);
                }
                ",
                "fragment shader",
                ShaderStage::Fragment,
            )?;
            self.outline_vertex = r.create_shader(
                "#version 450

                layout(location = 0) in vec3 in_pos;
                layout(location = 1) in vec3 in_normal;
                layout(location = 2) in vec2 in_uv;

                layout(location = 3) in float in_off;

                layout(set = 0, binding = 0) uniform Matrices {
                    mat4 model;
                    mat4 projection;
                    mat4 view;
                } matrices;

                void main() {
                    mat4 model = matrices.model;
                    vec4 off = vec4(
                        cos(in_off) * 3.0,
                        sin(in_off) * 3.0,
                        0.0,
                        0.0
                    );
                    model[0] *= 1.08;
                    model[1] *= 1.08;
                    model[2] *= 1.08;
                    model[3] += off;

                    gl_Position = matrices.projection * matrices.view * model * vec4(in_pos, 1.0);
                }
                ",
                "outline vertex",
                ShaderStage::Vertex
            )?;
            self.outline_fragment = r.create_shader(
                "#version 450
                layout(location = 0) out vec4 out_color;

                void main() {
                    out_color = vec4(1.0, 1.0, 1.0, 1.0);
                }
                ",
                "outline fragment",
                ShaderStage::Fragment,
            )?;
            self.pipeline_layouts = [
                    r.create_pipeline_layout(
                        [self.vertex_shader, self.fragment_shader],
                    )?,
                    r.create_pipeline_layout(
                        [self.outline_vertex, self.outline_fragment],
                    )?,
            ];
            self.outline_layout = r.create_pipeline_layout(
                [self.outline_vertex, self.outline_fragment],
            )?;
            self.color_format = r.supported_image_format(
                &[ColorFormat::SrgbRGBA8, ColorFormat::UnormRGBA8],
                FormatFeature::SampledImage | FormatFeature::ColorAttachment,
            ).unwrap();
            self.depth_stencil_format = r.supported_image_format(
                DepthStencilFormat::all_depth_stencil(),
                FormatFeature::DepthStencilAttachment,
            ).unwrap();
            self.depth_format = r.supported_image_format(
                DepthStencilFormat::all_depth(),
                FormatFeature::SampledImage | FormatFeature::DepthStencilAttachment,
            ).unwrap();
            self.image = r.create_image(
                &mut r.default_binder(),
                |builder| {
                    builder
                        .with_usage(ImageUsage::TransferDst)
                        .with_usage(ImageUsage::Sampled)
                        .with_dimensions(self.assets[0].dim)
                        .with_format(self.color_format, false)
                        .with_array_layers(8);
            })?;
            self.sampler = r.create_sampler(
                |_| {},
            )?;
            let mut graphics_pipeline_info = GraphicsPipelineInfo::new(self.pipeline_layouts[0]);
            graphics_pipeline_info
                .with_sample_shading(SampleShadingInfo::new(MSAA::X4, 0.2, false, false))
                .with_color_output(
                    self.color_format,
                    WriteMask::all(),
                    None,
                )
                .with_depth_output(self.depth_stencil_format)
                .with_stencil_output(self.depth_stencil_format)
                .with_vertex_input_binding(VertexInputBinding::new::<0, Vertex>(0, VertexInputRate::Vertex))
                .with_vertex_input_binding(VertexInputBinding::new::<3, VertexOff>(1, VertexInputRate::Instance))
                .with_depth_stencil(DepthStencilInfo {
                    compare_op: CompareOp::Less,
                    depth_bounds: Some(DepthBounds::new(0.0, 1.0)),
                    stencil_test_info: Some(StencilTestInfo {
                        front: Default::default(),
                        back: StencilOpState {
                            fail_op: StencilOp::Keep,
                            pass_op: StencilOp::Replace,
                            depth_fail_op: StencilOp::Keep,
                            compare_op: CompareOp::Always,
                            compare_mask: 0x0,
                            write_mask: 0x1,
                            reference: 0x1,
                        },
                    }),
                    write_enable: true,
                });
            let mut outline_pipeline_info = GraphicsPipelineInfo::new(self.pipeline_layouts[1]);
            outline_pipeline_info
                .with_sample_shading(SampleShadingInfo::new(MSAA::X4, 0.2, false, false))
                .with_color_output(
                    self.color_format,
                    WriteMask::all(),
                    None,
                )
                .with_depth_output(self.depth_stencil_format)
                .with_stencil_output(self.depth_stencil_format)
                .with_vertex_input_binding(VertexInputBinding::new::<0, Vertex>(0, VertexInputRate::Vertex))
                .with_vertex_input_binding(VertexInputBinding::new::<3, VertexOff>(1, VertexInputRate::Instance))
                .with_depth_stencil(DepthStencilInfo {
                    compare_op: CompareOp::Less,
                    depth_bounds: Some(DepthBounds::new(0.0, 1.0)),
                    stencil_test_info: Some(StencilTestInfo {
                        front: Default::default(),
                        back: StencilOpState {
                            fail_op: StencilOp::Keep,
                            pass_op: StencilOp::Keep,
                            depth_fail_op: StencilOp::Keep,
                            compare_op: CompareOp::NotEqual,
                            compare_mask: 0x1,
                            write_mask: 0x0,
                            reference: 0x1,
                        },
                    }),
                    write_enable: true,
                });
            r.create_graphics_pipelines(
                &[graphics_pipeline_info, outline_pipeline_info],
                |i, id| { self.pipelines[i] = id; },
                &GLOBAL_ALLOC
            )?;
            self.vertex_buffer = r.create_buffer(
                (CUBE_VERTICES.len() * size_of!(Vertex)) as u64,
                &[BufferUsage::VertexBuffer, BufferUsage::TransferDst],
                &mut r.default_binder(),
            )?;
            self.vertex_instance_buffer = r.create_buffer(
                (CUBE_OFF.len() * size_of!(VertexOff)) as u64,
                &[BufferUsage::VertexBuffer, BufferUsage::TransferDst],
                &mut r.default_binder(),
            )?;
            self.index_buffer = r.create_buffer(
                (CUBE_INDICES.len() * size_of!(u32)) as u64,
                &[BufferUsage::IndexBuffer , BufferUsage::TransferDst],
                &mut r.default_binder(),
            )?;
            self.matrices_buffer = r.create_buffer(
                size_of!(Matrices) as u64,
                &[BufferUsage::UniformBuffer],
                &mut r.default_binder_mappable(),
            )?;
            self.matrices_map = unsafe {
                r.map_buffer(self.matrices_buffer).unwrap().cast::<Matrices>()
            };
            self.light_info_buffer = r.create_buffer(
                size_of!(LightInfo) as u64,
                &[BufferUsage::UniformBuffer],
                &mut r.default_binder_mappable(),
            )?;
            self.light_info_map = unsafe {
                r.map_buffer(self.light_info_buffer).unwrap().cast::<LightInfo>()
            };
            r.allocate_shader_resources(
                &[
                    ShaderResourceInfo {
                        layout_id: self.pipeline_layouts[0],
                        set: 0,
                    },
                    ShaderResourceInfo {
                        layout_id: self.pipeline_layouts[0],
                        set: 1,
                    },
                ],
                |i, v| self.shader_resources[i] = v,
                &GLOBAL_ALLOC,
            )?;
            r.update_shader_resources(
                &[
                    ShaderResourceImageUpdate {
                        resource: self.shader_resources[1],
                        binding: 0,
                        starting_index: 0,
                        infos: &[ShaderResourceImageInfo {
                            sampler: self.sampler,
                            image_source: (self.image, None),
                        }]
                    },
                ],
                &[
                    ShaderResourceBufferUpdate {
                        resource: self.shader_resources[0],
                        binding: 0,
                        starting_index: 0,
                        infos: &[ShaderResourceBufferInfo {
                            buffer: self.matrices_buffer,
                            offset: 0,
                            size: size_of!(Matrices) as u64,
                        }],
                    },
                    ShaderResourceBufferUpdate {
                        resource: self.shader_resources[1],
                        binding: 1,
                        starting_index: 0,
                        infos: &[ShaderResourceBufferInfo {
                            buffer: self.light_info_buffer,
                            offset: 0,
                            size: size_of!(LightInfo) as u64,
                        }],
                    },
                ],
                &[],
                &GLOBAL_ALLOC
            )?;
            Ok(())
        })?;
        renderer_context
            .command_requests()
            .add_transfer_request(TransferRequest::new(1));
        Ok(())
    }

    fn update(
        &mut self,
        nox: &mut Nox<Self>,
        _renderer: &mut RendererContext,
        frame_buffer_size: (u32, u32),
    )
    {
        let (cursor_x, cursor_y) = nox.cursor_position();
        let relative_cursor = glam::vec3(
            1.0 - 2.0 * cursor_x as f32 / frame_buffer_size.0 as f32,
            2.0 * cursor_y as f32 / frame_buffer_size.1 as f32 - 1.0,
            0.0,
        );
        let proj = Mat4::perspective_lh(
            PI / 1.6,
            frame_buffer_size.0 as f32 / frame_buffer_size.1 as f32,
            0.01,
            100.0
        );
        //let proj = fix * Mat4::orthographic_lh(-5.0, 5.0, -5.0, 5.0, 0.1, 100.0);
        let view = Mat4::look_at_lh(vec3(0.0, 1.0, -2.0), vec3(0.0, 0.0, 3.0), vec3(0.0, 1.0, 0.0));
        let inv_proj = proj.inverse();
        let near_world = inv_proj * glam::vec4(relative_cursor.x, relative_cursor.y, 0.0, 1.0);
        let far_world = inv_proj * glam::vec4(relative_cursor.x, relative_cursor.y, 1.0, 1.0);
        let near_world = near_world.xyz() / near_world.w;
        let far_world = far_world.xyz() / far_world.w;
        let ray = (far_world - near_world).normalize();
        let plane_normal = vec3(0.0, 0.0, -1.0);
        let denom = plane_normal.dot(ray);
        let t = (vec3(0.0, 0.0, 3.0) - near_world).dot(plane_normal) / denom;
        let light_pos = near_world + ray * t;
        let mut tf = glam::Mat4::from_axis_angle(vec3(0.23, 1.0, 0.41).normalize(), self.rot);
        tf.col_mut(3).z = 3.0;
        unsafe {
            self.matrices_map.write(
                Matrices {
                    model: tf,
                    projection: proj,
                    view,
                }
            );
            self.light_info_map.write(
                LightInfo { pos: light_pos }
            );
        }
        self.rot += 0.001;
    }

    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn FrameGraphInit,
        pending_transfers: &[nox::renderer::CommandRequestID],
    ) -> Result<(), nox::renderer::Error>
    {
        if !pending_transfers.is_empty() {
            return Ok(())
        }
        let frame_graph = frame_graph.init(2)?;
        let frame_buffer_size = frame_graph.frame_buffer_size();
        let depth_stencil_output = frame_graph.add_transient_image(
            &mut |builder| {
                builder
                    .with_samples(MSAA::X4)
                    .with_usage(ImageUsage::DepthStencilAttachment)
                    .with_format(self.depth_stencil_format, false)
                    .with_dimensions(frame_buffer_size);
            }
        )?;
        let color_output = frame_graph.add_transient_image(
            &mut |builder| {
                builder
                    .with_samples(MSAA::X4)
                    .with_usage(ImageUsage::ColorAttachment)
                    .with_format(self.color_format, false)
                    .with_dimensions(frame_buffer_size);
            }
        )?;
        let color_output_resolve = frame_graph.add_transient_image(
            &mut |builder| {
                builder
                    .with_usage(ImageUsage::ColorAttachment)
                    .with_usage(ImageUsage::Sampled)
                    .with_format(self.color_format, false)
                    .with_dimensions(frame_buffer_size);
            }
        )?;
        let texture = frame_graph.add_image(self.image.into())?;
        frame_graph.set_render_image(color_output_resolve, None)?;
        self.first_pass = frame_graph.add_pass(
            PassInfo { max_color_writes: 1, max_reads: 2, msaa_samples: MSAA::X4 },
            &mut |builder| {
                builder
                    .with_read(ReadInfo { resource_id: texture, range_info: None })
                    .with_write(WriteInfo::new(
                        color_output,
                        None,
                        None,
                        None,
                        AttachmentLoadOp::Clear,
                        AttachmentStoreOp::Store,
                        Default::default())
                    )
                    .with_depth_stencil_write(WriteInfo::new(
                        depth_stencil_output,
                        None,
                        None,
                        None,
                        AttachmentLoadOp::Clear,
                        AttachmentStoreOp::Store,
                        ClearValue::DepthStencil{ depth: 1.0, stencil: 0 },
                    ));
            }
        )?;
        self.second_pass = frame_graph.add_pass(
            PassInfo { max_color_writes: 1, max_reads: 2, msaa_samples: MSAA::X4 },
            &mut |builder| {
                builder
                    .with_write(WriteInfo::new(
                        color_output,
                        None,
                        Some((color_output_resolve, ResolveMode::Average)),
                        None,
                        AttachmentLoadOp::Load,
                        AttachmentStoreOp::Store,
                        Default::default()
                    ))
                    .with_depth_stencil_write(WriteInfo::new(
                        depth_stencil_output,
                        None,
                        None,
                        None,
                        AttachmentLoadOp::Load,
                        AttachmentStoreOp::Store,
                        Default::default(),
                    ));
            }
        )?;
        //let id = frame_graph.add_image(self.image.into())?;
        //frame_graph.set_render_image(id);
        Ok(())
    }

    fn render_commands(
        &mut self,
        pass: PassID,
        commands: &mut RenderCommands,
    ) -> Result<(), nox::renderer::Error>
    {
        match pass {
            x if x == self.first_pass => {
                commands.bind_pipeline(self.pipelines[0])?;
                commands.bind_shader_resources(|i| {
                    self.shader_resources[i as usize]
                })?;
                commands.draw_indexed(
                    DrawInfo {
                        index_count: CUBE_INDICES.len() as u32,
                        instance_count: 8,
                        ..Default::default()
                    },
                    [
                        DrawBufferInfo::new(self.vertex_buffer, 0),
                        DrawBufferInfo::new(self.vertex_instance_buffer, 0),
                    ].into(),
                    DrawBufferInfo::new(self.index_buffer, 0),
                )?;
            },
            x if x == self.second_pass => {
                commands.bind_pipeline(self.pipelines[1])?;
                commands.bind_shader_resources(|_| {
                    self.shader_resources[0]
                })?;
                commands.draw_indexed(
                    DrawInfo {
                        index_count: CUBE_INDICES.len() as u32,
                        instance_count: 8,
                        ..Default::default()
                    },
                    [
                        DrawBufferInfo::new(self.vertex_buffer, 0),
                        DrawBufferInfo::new(self.vertex_instance_buffer, 0),
                    ].into(),
                    DrawBufferInfo::new(self.index_buffer, 0),
                )?
            }
            _ => {}
        }
        Ok(())
    }

    fn transfer_commands(
        &mut self,
        _id: nox::renderer::CommandRequestID,
        command_buffer: &mut nox::renderer::TransferCommandbuffer,
    ) -> Result<(), Error>
    {
        for (i, asset) in self.assets.iter().enumerate() {
            command_buffer.copy_data_to_image(
                self.image,
                asset.as_bytes(),
                ImageSubresourceLayers::new(ImageAspect::Color, 0, i as u32, 1),
                None,
                None,
            ).unwrap();
        }
        let vertices = unsafe { slice_as_bytes(CUBE_VERTICES) }.unwrap();
        command_buffer.copy_data_to_buffer(
            self.vertex_buffer,
            vertices, 0, vertices.len() as u64,
        ).unwrap();
        let vertices_instance = unsafe { slice_as_bytes(CUBE_OFF) }.unwrap();
        command_buffer.copy_data_to_buffer(
            self.vertex_instance_buffer,
            vertices_instance, 0, vertices_instance.len() as u64
        ).unwrap();
        let indices = unsafe { slice_as_bytes(CUBE_INDICES) }.unwrap();
        command_buffer.copy_data_to_buffer(
            self.index_buffer,
            indices,
            0,
            indices.len() as u64,
        ).unwrap();
        Ok(())
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
