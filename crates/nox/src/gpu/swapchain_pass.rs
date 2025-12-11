use std::{
    fs::{self, File}, io::Write, path::PathBuf, sync::{Arc, RwLock}
};

use memmap2::Mmap;

use ash::vk;

use nox_mem::{vec_types::{Vector, ArrayVec, GlobalVec}};

use nox_alloc::arena_alloc::*;

use crate::dev::error::*;

use super::{
    *,
    pipeline::*,
    shader::ShaderStage,
};

pub(crate) struct SwapchainPassPipelineData {
    global_resources: Arc<RwLock<GlobalResources>>,
    layout_id: PipelineLayoutId,
    shader_resources: ArrayVec<ShaderResourceId, {MAX_BUFFERED_FRAMES as usize}>,
    sampler: SamplerId,
    pipelines: GlobalVec<(GraphicsPipelineId, vk::Format)>,
    last_pipeline: Option<(GraphicsPipelineId, vk::Format)>,
    shaders: [ShaderId; 2],
    pipeline_cache_id: PipelineCacheId,
    cache_dir: PathBuf,
}

impl SwapchainPassPipelineData {


    const CACHE_NAME: &str = "sc_pass_cache.nox";

    pub fn new(
        global_resources: Arc<RwLock<GlobalResources>>,
        buffered_frame_count: u32,
        tmp_alloc: &ArenaAlloc,
    ) -> Result<Self>
    {
        assert!(buffered_frame_count <= MAX_BUFFERED_FRAMES);

        let mut g = global_resources.write().unwrap();

        let mut cache_dir = std::env
            ::current_exe()
            .context("io error")?;
        cache_dir.pop();
        cache_dir.push(Self::CACHE_NAME);

        let pipeline_cache_id = 
            if fs::exists(&cache_dir).context("io error")? {
                let file = File
                    ::open(&cache_dir)
                    .context("failed to open pipeline cache file")?;
                let map = unsafe {
                    Mmap::map(&file)
                        .context("failed to map pipeline cache")?
                };
                g.create_pipeline_cache(Some(&map))
                    .context("failed to create pipeline cache with data")?
            }
            else {
                File::create_new(&cache_dir).context("failed to create pipeline cache file")?;
                g.create_pipeline_cache(None)
                    .context("failed to create pipeline cache")?
            };

        let shaders = [
            g.create_shader(Self::vertex_shader_input(), "swapchain_pass_vertex", ShaderStage::Vertex)
                .context("failed to create vertex shader")?,
            g.create_shader(Self::fragment_shader_input(), "swapchain_pass_fragment", ShaderStage::Fragment)
                .context("failed to create fragment shader")?,
        ];

        let layout_id = g.create_pipeline_layout(shaders)
            .context("failed to create pipeline layout")?;

        let sampler = g.create_sampler(|_| {})
            .context("failed to create sampler")?;

        let stack_guard = ArenaGuard::new(&tmp_alloc);

        let resource_infos = ArrayVec::<ShaderResourceInfo, {MAX_BUFFERED_FRAMES as usize}>
            ::with_len(ShaderResourceInfo { layout_id, set: 0 }, buffered_frame_count as usize)
            .unwrap();

        let mut shader_resources = ArrayVec::new();

        g.allocate_shader_resources(
            &resource_infos,
            |_, v| { shader_resources.push(v).unwrap(); },
            &stack_guard,
        ).context("failed to allocate shader resources")?;

        Ok(Self {
            global_resources: global_resources.clone(),
            layout_id,
            shader_resources,
            sampler,
            pipelines: GlobalVec::new(),
            last_pipeline: Default::default(),
            shaders,
            pipeline_cache_id,
            cache_dir,
        })
    }

    pub fn _update_buffered_frame_count(
        &mut self,
        buffered_frame_count: u32,
        tmp_alloc: &ArenaAlloc,
    ) -> Result<()>
    {
        let mut g = self.global_resources.write().unwrap();
        let stack_guard = ArenaGuard::new(&tmp_alloc);
        g.free_shader_resources(&self.shader_resources, &stack_guard)?;

        self.shader_resources.clear();

        let resource_infos = ArrayVec::<ShaderResourceInfo, {MAX_BUFFERED_FRAMES as usize}>
            ::with_len(ShaderResourceInfo { layout_id: self.layout_id, set: 0 }, buffered_frame_count as usize)
            .unwrap();

        g.allocate_shader_resources(
            &resource_infos,
            |_, v| { self.shader_resources.push(v).unwrap(); },
            &stack_guard,
        )?;

        Ok(())
    }

    pub fn get_pipeline(
        &mut self,
        format: vk::Format,
        tmp_alloc: &ArenaAlloc,
    ) -> Result<vk::Pipeline>
    {
        let mut g = self.global_resources.write().unwrap();
        if let Some(pipeline) = self.last_pipeline {
            if format == pipeline.1 {
                return Ok(g
                    .get_graphics_pipeline(pipeline.0)
                    .unwrap().handle
                )
            }
        }
        if let Some(pipeline) = self.pipelines.iter().find(|p| p.1 == format) {
            self.last_pipeline = Some(*pipeline);
            return Ok(g
                .get_graphics_pipeline(pipeline.0)
                .unwrap().handle
            )
        }
        let mut info = GraphicsPipelineInfo::new(self.layout_id);
        info.with_color_output_vk(format, WriteMask::all(), None);
        let mut pipeline = None;
        let stack_guard = ArenaGuard::new(&tmp_alloc);
        g
            .create_graphics_pipelines(
                &[info],
                Some(self.pipeline_cache_id),
                &stack_guard, |_, v| { pipeline = Some(v) },
            )?;
        let pipeline = self.pipelines.push((pipeline.unwrap(), format));
        Ok(g
            .get_graphics_pipeline(pipeline.0)
            .unwrap().handle
        )
    }

    pub fn get_pipeline_layout(&mut self, context: &mut GpuContext) -> Result<vk::PipelineLayout>
    {
        Ok(context
            .get_pipeline_layout(self.layout_id)
            .context("couldn't find swapchain pass pipeline")?
            .handle()
        )
    }

    pub fn get_descriptor_set(
        &mut self,
        context: &mut GpuContext,
        image: ImageId,
        range_info: Option<ImageRangeInfo>,
        frame_index: u32,
        tmp_alloc: &ArenaAlloc,
    ) -> Result<vk::DescriptorSet>
    {
        let stack_guard = ArenaGuard::new(&tmp_alloc);
        let resource_id = self.shader_resources[frame_index as usize];
        let update = ShaderResourceImageUpdate {
            resource: resource_id,
            binding: 0,
            starting_index: 0,
            infos: &[
                ShaderResourceImageInfo {
                    sampler: self.sampler,
                    image_source: (image, range_info),
                    storage_image: false,
            }],
        };
        context.update_shader_resources(&[update], &[], &[], &stack_guard)?;
        Ok(context.get_descriptor_set(resource_id).unwrap())
    }

    const fn vertex_shader_input() -> &'static str {
        "
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
        "
    }

    const fn fragment_shader_input() -> &'static str {
        "
            #version 450

            layout(location = 0) in vec2 in_uv;

            layout(location = 0) out vec4 out_color;

            layout(set = 0, binding = 0) uniform sampler2D render_image;

            void main() {
                out_color = texture(render_image, in_uv);
            }
        "
    }
}

impl Drop for SwapchainPassPipelineData {

    fn drop(&mut self) {
        let mut g = self.global_resources.write().unwrap();
        for shader in self.shaders {
            g.destroy_shader(shader);
        }
        for pipeline in &self.pipelines {
            g.destroy_graphics_pipeline(pipeline.0);
        }
        g.destroy_sampler(self.sampler);
        if let Ok(mut file) = File::create(&self.cache_dir) {
            if let Ok(data) = g.retrieve_pipeline_cache_data(self.pipeline_cache_id) {
                file.write(&data).ok();
            }
        }
    }
}
