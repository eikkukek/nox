use rustc_hash::FxHashMap;

use crate::gpu::*;

use crate::mem::{Allocator, vec_types::ArrayVec};

use crate::dev::error::{Result, Context};

const VERTEX_SHADER: &'static str = "
#version 450

vec3 positions[] = {

    vec3(-1.0f, -1.0f, 0.0f),
    vec3(-1.0f, 1.0f, 0.0f),
    vec3(1.0f, -1.0f, 0.0f),

    vec3(1.0f, -1.0f, 0.0f),
    vec3(-1.0f, 1.0f, 0.0f),
    vec3(1.0f, 1.0f, 0.0f)
};

vec2 uvs[] = {

    vec2(0.0f, 0.0f),
    vec2(0.0f, 1.0f),
    vec2(1.0f, 0.0f),

    vec2(1.0f, 0.0f),
    vec2(0.0f, 1.0f),
    vec2(1.0f, 1.0f)
};

layout(location = 0) out vec2 out_uv;

void main() {
    out_uv = uvs[gl_VertexIndex];
    gl_Position = vec4(positions[gl_VertexIndex], 1.0f);
}
";

const FRAGMENT_SHADER: &'static str = "
#version 450

layout(set = 0, binding = 0) uniform sampler2D image;

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = texture(image, in_uv);
}
";

pub struct FullScreenPass {
    pipeline_layout: PipelineLayoutId,
    shader_resource: ShaderResourceId,
    pipelines: FxHashMap<ImageFormat, GraphicsPipelineId>,
    current_pipeline: Option<GraphicsPipelineId>,
    prev_images: ArrayVec<(ImageId, Option<ImageRangeInfo>), {MAX_BUFFERED_FRAMES as usize}>,
    pass_id: PassId,
}

impl FullScreenPass {

    pub fn init(
        gpu: &mut GpuContext,
    ) -> Result<Self>
    {
        let vertex_shader = gpu.create_shader(ShaderSource::glsl(
            VERTEX_SHADER,
            "full screen pass vertex shader",
            ShaderStage::Vertex,
        )).context("failed to create shaders")?;
        let fragment_shader = gpu.create_shader(ShaderSource::glsl(
            FRAGMENT_SHADER,
            "full screen pass fragment shader",
            ShaderStage::Fragment,
        )).context("failed to create shaders")?;
        let pipeline_layout = gpu.create_pipeline_layout([
            vertex_shader, fragment_shader,
        ]).context("failed to create pipeline layout")?;
        let mut shader_resource = Default::default();
        gpu.allocate_shader_resources(
            &[ShaderResourceInfo::new(
                pipeline_layout,
                0,
            )],
            |_, resource| shader_resource = resource,
            &GlobalAlloc,
        ).context("failed to allocate shader resources")?;
        Ok(Self {
            pipeline_layout,
            shader_resource,
            pipelines: FxHashMap::default(),
            current_pipeline: None,
            prev_images: ArrayVec
                ::with_len(Default::default(), gpu.buffered_frames() as usize)
                .context("vec error")?,
            pass_id: Default::default(),
        })
    }

    pub fn frame_buffer_created(
        &mut self,
        gpu: &mut GpuContext,
        new_format: ImageFormat,
        cache: impl Into<Option<PipelineCacheId>>,
    ) -> Result<()>
    {
        if let Some(pipeline) = self.pipelines.get(&new_format) {
            self.current_pipeline = Some(*pipeline);
            Ok(())
        } else {
            let mut info = GraphicsPipelineInfo::new(self.pipeline_layout);
            info.with_color_output(new_format, WriteMask::all(), None);
            let mut pipeline = Default::default();
            gpu.create_graphics_pipelines(
                &[info],
                cache.into(),
                &GlobalAlloc,
                |_, p| pipeline = p,
            ).context_with(|| format_compact!(
                "failed to create full screen pass pipeline with format {:?}", new_format)
            )?;
            self.current_pipeline = Some(*self.pipelines.entry(new_format).or_insert(pipeline));
            Ok(())
        }
    }

    pub fn render(
        &mut self,
        frame_graph: &mut FrameGraph,
        image: ResourceId,
        range: Option<ImageRangeInfo>,
        sampler: SamplerId,
        load_op: AttachmentLoadOp,
        wait_semaphores: &[(TimelineSemaphoreId, u64, PipelineStage)],
        signal_semaphores: &[(TimelineSemaphoreId, u64)],
        alloc: &impl Allocator,
    ) -> Result<PassId> {
        let image_id = image.image_id();
        let prev_image = &mut self.prev_images[frame_graph.frame_index() as usize];
        if prev_image.0 != image_id || prev_image.1 != range {
            frame_graph.gpu_mut().update_shader_resources(
                &[ShaderResourceImageUpdate {
                    resource: self.shader_resource,
                    binding: 0,
                    starting_index: 0,
                    infos: &[ShaderResourceImageInfo {
                        sampler,
                        image_source: (image_id, range),
                        storage_image: false,
                    }]
                }],
                &[], &[], alloc,
            ).context("failed to update shader resources")?;
            *prev_image = (image_id, range);
        }
        let swapchain_image = frame_graph.swapchain_image();
        self.pass_id = frame_graph.add_pass(PassInfo {
            max_reads: 1, max_color_writes: 1,
            max_wait_semaphores: wait_semaphores.len() as u32,
            max_signal_semaphores: signal_semaphores.len() as u32,
            ..Default::default()
        }, |pass| {
            pass.with_read(ReadInfo::new(image, range))?
                .with_write(WriteInfo::new(swapchain_image)
                    .with_load_op(load_op)
                )?;
            for &(id, value, stage) in wait_semaphores {
                pass.with_wait_semaphore(id, value, stage)?;
            }
            for &(id, value) in signal_semaphores {
                pass.with_signal_semaphore(id, value)?;
            }
            Ok(())
        }).context("failed to add full screen pass")?;
        Ok(self.pass_id)
    }

    pub fn render_work(
        &self,
        commands: &mut RenderCommands,
        pass_id: PassId,
    ) -> Result<()>
    {
        if pass_id == self.pass_id && let Some(pipeline) = self.current_pipeline {
            commands.bind_pipeline(pipeline)?;
            commands.bind_shader_resources(|_| {
                self.shader_resource
            })?;
            commands.draw_bufferless(6, 1)?;
        }
        Ok(())
    }
}
