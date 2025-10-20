use std::{
    ffi::CString, fs::{self, File}, io::Write, path::{Path, PathBuf}, ptr::NonNull
};

use core::f32::consts::PI;

use glam::{f32::*, Vec4Swizzles};

use memmap2::Mmap;

use nox::{
    mem::{size_of, slice_as_bytes, vec_types::ArrayVec, GlobalAlloc},
    *,
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
    fire_format: FloatFormat,
    image: ImageId,
    fire_images: [ImageId; 2],
    sampler: SamplerId,
    vertex_shader: ShaderId,
    fragment_shader: ShaderId,
    outline_vertex: ShaderId,
    outline_fragment: ShaderId,
    fire_effect_compute: ShaderId,
    fire_effect_vertex: ShaderId,
    fire_effect_fragment: ShaderId,
    pipeline_layouts: [PipelineLayoutId; 4],
    pipelines: [GraphicsPipelineId; 3],
    fire_pipeline: ComputePipelineId,
    vertex_buffer: BufferId,
    vertex_instance_buffer: BufferId,
    index_buffer: BufferId,
    matrices_buffer: BufferId,
    matrices_map: NonNull<Matrices>,
    light_info_buffer: BufferId,
    light_info_map: NonNull<LightInfo>,
    shader_resources: [ShaderResourceId; 4],
    fire_pass: PassId,
    cube_pass: PassId,
    outline_pass: PassId,
    frame_buffer_size: image::Dimensions,
    cache_dir: PathBuf,
    pipeline_cache: PipelineCacheId,
    heat_in: u32,
    fire_transfer_id: CommandRequestId,
    rot: f32,
}

impl App {

    fn new() -> Self {
        let mut cache_dir = std::env::current_exe().unwrap();
        cache_dir.pop();
        cache_dir.push("example.cache");
        Self {
            assets: Default::default(),
            color_format: ColorFormat::SrgbRGBA8,
            depth_stencil_format: DepthStencilFormat::D32S8,
            depth_format: DepthStencilFormat::D32,
            image: Default::default(),
            fire_format: FloatFormat::R32,
            fire_images: Default::default(),
            sampler: Default::default(),
            vertex_shader: Default::default(),
            fragment_shader: Default::default(),
            outline_vertex: Default::default(),
            outline_fragment: Default::default(),
            fire_effect_compute: Default::default(),
            fire_effect_vertex: Default::default(),
            fire_effect_fragment: Default::default(),
            pipeline_layouts: Default::default(),
            pipelines: Default::default(),
            fire_pipeline: Default::default(),
            vertex_buffer: Default::default(),
            vertex_instance_buffer: Default::default(),
            index_buffer: Default::default(),
            matrices_buffer: Default::default(),
            matrices_map: NonNull::dangling(),
            light_info_buffer: Default::default(),
            light_info_map: NonNull::dangling(),
            shader_resources: Default::default(),
            fire_pass: Default::default(),
            cube_pass: Default::default(),
            outline_pass: Default::default(),
            fire_transfer_id: Default::default(),
            frame_buffer_size: Default::default(),
            cache_dir,
            pipeline_cache: Default::default(),
            heat_in: 0,
            rot: 0.0,
        }
    }
}

impl Interface for App {

    fn init_settings(&self) -> InitSettings {
        InitSettings::new("Test", Version::default(), [540, 540], true, false)
    }

    fn init_callback(
        &mut self,
        nox: &mut Nox<Self>,
        renderer: &mut nox::renderer::RendererContext
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
        renderer.edit_resources(|r| {
            self.pipeline_cache =
                if fs::exists(&self.cache_dir)? {
                    let file = File::open(&self.cache_dir)?;
                    let map = unsafe {
                        Mmap::map(&file)?
                    };
                    r.create_pipeline_cache(Some(&map))?
                }
                else {
                    File::create_new(&self.cache_dir)?;
                    r.create_pipeline_cache(None)?
                };
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
            self.fire_effect_compute = r.create_shader(
                "#version 450

                layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

                layout(binding = 0, r32F) uniform image2D smoke_in;
                layout(binding = 1, r32F) uniform image2D smoke_out;

                layout(push_constant) uniform Params {
                    float density;
                    float noise_strength;
                    float time;
                } params;

                float rand(vec2 co) {
                    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
                }

                void main() {

                    ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
                    ivec2 size = imageSize(smoke_in);

                    float s = imageLoad(smoke_in, uv).r;

                    float down = imageLoad(smoke_in, uv + ivec2(0, 1)).r;
                    float right = imageLoad(smoke_in, uv + ivec2(1, 0)).r;
                    float left = imageLoad(smoke_in, uv - ivec2(1, 0)).r;

                    if (down > right && down > left) {
                        s = max(mix(s, down, 0.5), s);
                    }
                    if (right > left) {
                        s = mix(s, right, 0.00);
                    }
                    else {
                        s = mix(s, left, 0.00);
                    }

                    s *= params.density;

                    if (uv.y == size.y - 1 && abs(s) < 0.5) {
                        s = rand(uv * params.time);
                        //if (s < 0.7) s = 0.0;
                        //s = params.noise_strength * s;
                    }

                    imageStore(smoke_out, uv, vec4(s, 0.0, 0.0, 1.0));
                }
                ",
                "fire effect",
                ShaderStage::Compute,
            )?;
            self.fire_effect_vertex = r.create_shader(
                "#version 450

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
                    vec2(0.0, 1.0),
                    vec2(1.0, 1.0),
                    vec2(1.0, 0.0),
                    vec2(0.0, 0.0),
                    vec2(0.0, 1.0),
                    vec2(1.0, 0.0)
                );

                void main() {
                    int vertex_index = gl_VertexIndex;
                    out_uv = uvs[vertex_index];
                    gl_Position = vec4(positions[vertex_index], 0.0, 1.0);
                }
                ",
                "fire effect vertex",
                ShaderStage::Vertex,
            )?;
            self.fire_effect_fragment = r.create_shader(
                "#version 450

                layout(location = 0) in vec2 in_uv;

                layout(location = 0) out vec4 out_color;

                layout(set = 0, binding = 0) uniform sampler2D fire_tex;

                const float hot = 12.0 / 360.0;
                const float warm = 188.0 / 360.0;

                const float sat = 1.0;
                const float light = 0.63;

                float hue_to_rgb(float p, float q, float t) {
                    if (t < 0.0) t += 1.0;
                    if (t > 1.0) t -= 1.0;
                    if (t < 1.0 / 6.0) return p + (q - p) * 6.0 * t;
                    if (t < 0.5) return q;
                    if (t < 2.0 / 3.0) return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
                    return p;
                }

                vec3 hsl_to_rgb(float h, float s, float l) {
                    const float q = l < 0.5 ? l * (1.0 + s) : l + s - l * s;
                    const float p = 2.0 * l - q;
                    float r = hue_to_rgb(p, q, h + 1.0 / 3.0);
                    float g = hue_to_rgb(p, q, h);
                    float b = hue_to_rgb(p, q, h - 1.0 / 3.0);
                    return vec3(r, g, b);
                }

                void main() {
                    float r = texture(fire_tex, in_uv).r;
                    float a = r;
                    float hue = mix(warm, hot, r);
                    vec3 rgb = hsl_to_rgb(hue, sat, light);
                    out_color = vec4(rgb * a, 1.0);
                }
                ",
                "fire effect fragment",
                ShaderStage::Fragment,
            )?;
            self.pipeline_layouts = [
                    r.create_pipeline_layout(
                        [self.vertex_shader, self.fragment_shader],
                    )?,
                    r.create_pipeline_layout(
                        [self.outline_vertex, self.outline_fragment],
                    )?,
                    r.create_pipeline_layout(
                        [self.fire_effect_compute],
                    )?,
                    r.create_pipeline_layout(
                        [self.fire_effect_vertex, self.fire_effect_fragment],
                    )?,
            ];
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
            self.fire_format = r.supported_image_format(
                &[FloatFormat::R32],
                FormatFeature::SampledImage | FormatFeature::StorageImage
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
                }
            )?;
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
            let mut fire_effect_pipeline_info = GraphicsPipelineInfo::new(self.pipeline_layouts[3]);
            fire_effect_pipeline_info
                .with_sample_shading(SampleShadingInfo::new(MSAA::X4, 0.2, false, false))
                .with_color_output(
                    self.color_format,
                    WriteMask::all(),
                    None,
                );
            r.create_graphics_pipelines(
                &[graphics_pipeline_info, outline_pipeline_info, fire_effect_pipeline_info],
                Some(self.pipeline_cache),
                &GlobalAlloc,
                |i, id| { self.pipelines[i] = id; },
            )?;
            let fire_pipeline_info = ComputePipelineInfo::new(self.pipeline_layouts[2]);
            r.create_compute_pipelines(
                &[fire_pipeline_info],
                Some(self.pipeline_cache),
                &GlobalAlloc,
                |_, id| { self.fire_pipeline = id },
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
                    ShaderResourceInfo {
                        layout_id: self.pipeline_layouts[2],
                        set: 0,
                    },
                    ShaderResourceInfo {
                        layout_id: self.pipeline_layouts[3],
                        set: 0,
                    },
                ],
                |i, v| self.shader_resources[i] = v,
                &GlobalAlloc,
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
                            storage_image: false,
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
                &GlobalAlloc
            )?;
            Ok(())
        })?;
        renderer
            .transfer_requests()
            .add_request(1 << 28);
        self.frame_buffer_size = renderer.frame_buffer_size();
        Ok(())
    }

    fn frame_buffer_size_callback(
        &mut self,
        renderer: &mut RendererContext
    ) -> Result<(), nox::Error>
    {
        renderer.edit_resources(|r| {
            for image in self.fire_images.iter_mut() {
                if r.is_valid_image(*image) {
                    r.destroy_image(*image);
                }
                *image = r.create_image(
                    &mut r.default_binder(),
                    |builder| {
                        builder
                            .with_usage(ImageUsage::Storage)
                            .with_usage(ImageUsage::Sampled)
                            .with_usage(ImageUsage::TransferDst)
                            .with_dimensions(renderer.frame_buffer_size())
                            .with_format(self.fire_format, false);
                    }
                )?;
                self.heat_in = 0;
            }
            Ok(())
        })?;
        self.fire_transfer_id = renderer
            .transfer_requests()
            .add_request(1 << 28);
        self.frame_buffer_size = renderer.frame_buffer_size();
        Ok(())
    }

    fn update(
        &mut self,
        nox: &mut Nox<Self>,
        _renderer: &mut RendererContext,
    ) -> Result<(), Error>
    {
        let (cursor_x, cursor_y) = nox.cursor_position();
        let relative_cursor = glam::vec3(
            1.0 - 2.0 * cursor_x as f32 / self.frame_buffer_size.width as f32,
            2.0 * cursor_y as f32 / self.frame_buffer_size.height as f32 - 1.0,
            0.0,
        );
        let proj = Mat4::perspective_lh(
            PI / 1.6,
            self.frame_buffer_size.width as f32 / self.frame_buffer_size.height as f32,
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
        self.rot = (self.rot + PI * nox.delta_time().as_secs_f32()) % (PI * 2.0);
        Ok(())
    }

    fn compute(
        &mut self,
        commands: &mut ComputeCommands,
    ) -> Result<(), Error>
    {
        struct Params {
            _density: f32,
            _noise_strength: f32,
            _time: f32,
        }
        let params = &[Params {
            _density: 0.995,
            _noise_strength: 0.5,
            _time: self.rot,
        }];
        commands.edit_resources(|r| {
            r.update_shader_resources(
                &[
                    ShaderResourceImageUpdate {
                        resource: self.shader_resources[2],
                        binding: 0,
                        starting_index: 0,
                        infos: &[ShaderResourceImageInfo {
                            sampler: self.sampler,
                            image_source: (self.fire_images[self.heat_in as usize], None),
                            storage_image: true,
                        }]
                    },
                    ShaderResourceImageUpdate {
                        resource: self.shader_resources[2],
                        binding: 1,
                        starting_index: 0,
                        infos: &[ShaderResourceImageInfo {
                            sampler: self.sampler,
                            image_source: (self.fire_images[(self.heat_in + 1) as usize % 2], None),
                            storage_image: true,
                        }]
                    },
                    ShaderResourceImageUpdate {
                        resource: self.shader_resources[3],
                        binding: 0,
                        starting_index: 0,
                        infos: &[ShaderResourceImageInfo {
                            sampler: self.sampler,
                            image_source: (self.fire_images[self.heat_in as usize], None),
                            storage_image: false,
                        }]
                    }
                ],
                &[],
                &[],
                &GlobalAlloc
            )?;
            Ok(())
        })?;
        commands.prepare_storage_image(self.fire_images[0])?;
        commands.prepare_storage_image(self.fire_images[1])?;
        commands.bind_pipeline(self.fire_pipeline)?;
        commands.bind_shader_resources(|_| self.shader_resources[2])?;
        commands.push_constants(|_| unsafe { slice_as_bytes(params).unwrap() })?;
        commands.dispatch((self.frame_buffer_size.width + 7) / 8, (self.frame_buffer_size.height + 7) / 8, 1);
        self.heat_in = (self.heat_in + 1) % 2;
        Ok(())
    }

    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn FrameGraphInit,
        pending_transfers: &[nox::renderer::CommandRequestId],
    ) -> Result<(), Error>
    {
        if !pending_transfers.is_empty() {
            return Ok(())
        }
        let frame_graph = frame_graph.init(3)?;
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
        let fire_tex = frame_graph.add_image(self.fire_images[self.heat_in as usize])?;
        let texture = frame_graph.add_image(self.image)?;
        frame_graph.set_render_image(color_output_resolve, None)?;
        self.fire_pass = frame_graph.add_pass(
            PassInfo { max_color_writes: 1, max_reads: 1, msaa_samples: MSAA::X4 },
            &mut |builder| {
                builder
                    .with_read(ReadInfo::new(fire_tex, None))
                    .with_write(WriteInfo::new(
                        color_output,
                        None,
                        None,
                        None,
                        AttachmentLoadOp::Clear,
                        AttachmentStoreOp::Store,
                        Default::default(),
                    ));
            },
        )?;
        self.cube_pass = frame_graph.add_pass(
            PassInfo { max_color_writes: 1, max_reads: 2, msaa_samples: MSAA::X4 },
            &mut |builder| {
                builder
                    .with_read(ReadInfo { resource_id: texture, range_info: None })
                    .with_write(WriteInfo::new(
                        color_output,
                        None,
                        None,
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
                        AttachmentLoadOp::Clear,
                        AttachmentStoreOp::Store,
                        ClearValue::DepthStencil{ depth: 1.0, stencil: 0 },
                    ));
            }
        )?;
        self.outline_pass = frame_graph.add_pass(
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
        Ok(())
    }

    fn transfer_commands(
        &mut self,
        id: nox::renderer::CommandRequestId,
        commands: &mut nox::renderer::TransferCommands,
    ) -> Result<Option<std::thread::JoinHandle<()>>, Error>
    {
        if id == self.fire_transfer_id {
            for image in &self.fire_images {
                commands.clear_color_image(*image, [0.0, 0.0, 0.0, 0.0].into(), None, &GlobalAlloc)?;
            }
            return Ok(None)
        }
        for (i, asset) in self.assets.iter().enumerate() {
            commands.copy_data_to_image(
                self.image,
                asset.as_bytes(),
                ImageSubresourceLayers::new(ImageAspect::Color, 0, i as u32, 1),
                None,
                None,
            ).unwrap();
        }
        let vertices = unsafe { slice_as_bytes(CUBE_VERTICES) }.unwrap();
        commands.copy_data_to_buffer(
            self.vertex_buffer,
            vertices, 0, vertices.len() as u64,
        ).unwrap();
        let vertices_instance = unsafe { slice_as_bytes(CUBE_OFF) }.unwrap();
        commands.copy_data_to_buffer(
            self.vertex_instance_buffer,
            vertices_instance, 0, vertices_instance.len() as u64
        ).unwrap();
        let indices = unsafe { slice_as_bytes(CUBE_INDICES) }.unwrap();
        commands.copy_data_to_buffer(
            self.index_buffer,
            indices,
            0,
            indices.len() as u64,
        ).unwrap();
        Ok(None)
    }

    fn render_commands(
        &mut self,
        pass: PassId,
        commands: &mut RenderCommands,
    ) -> Result<(), Error>
    {
        match pass {
            x if x == self.fire_pass => {
                commands.bind_pipeline(self.pipelines[2])?;
                commands.bind_shader_resources(|_|
                    self.shader_resources[3]
                )?;
                commands.draw_bufferless(6, 1);
            },
            x if x == self.cube_pass => {
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
            x if x == self.outline_pass => {
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

    fn clean_up(
        &mut self,
        renderer: &mut RendererContext,
    )
    {
        renderer.edit_resources(|r| {
            let mut file = File::create(&self.cache_dir)?;
            let data = r.retrieve_pipeline_cache_data(self.pipeline_cache)?;
            file.write(&data)?;
            println!("cache written, len {}", data.len());
            Ok(())
        }).ok();
    }
}

fn main() {
    let app = App::new();
    let mut memory = Default::default();
    Nox::new(app, &mut memory).run();
}
