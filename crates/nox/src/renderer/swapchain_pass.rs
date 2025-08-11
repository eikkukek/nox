use std::sync::{Arc, RwLock};

use ash::vk;

use nox_mem::{vec_types::{Vector, ArrayVec, GlobalVec}};

use nox_alloc::arena_alloc::*;

use crate::renderer::image::ImageRangeInfo;

use super::{
    *,
    pipeline::*,
    shader::ShaderStage,
};

pub(crate) struct SwapchainPassPipelineData {
    global_resources: Arc<RwLock<GlobalResources>>,
    layout_id: PipelineLayoutID,
    shader_resources: ArrayVec<ShaderResourceID, {MAX_BUFFERED_FRAMES as usize}>,
    sampler: SamplerID,
    pipelines: GlobalVec<(GraphicsPipelineID, vk::Format)>,
    last_pipeline: Option<(GraphicsPipelineID, vk::Format)>,
    shaders: [ShaderID; 2],
}

impl SwapchainPassPipelineData {

    pub fn new(
        global_resources: Arc<RwLock<GlobalResources>>,
        buffered_frame_count: u32,
        tmp_alloc: &ArenaAlloc,
    ) -> Result<Self, Error>
    {
        assert!(buffered_frame_count <= MAX_BUFFERED_FRAMES);

        let mut g = global_resources.write().unwrap();

        let shaders = [
            g.create_shader(Self::vertex_shader_input(), "swapchain_pass_vertex", ShaderStage::Vertex)?,
            g.create_shader(Self::fragment_shader_input(), "swapchain_pass_fragment", ShaderStage::Fragment)?,
        ];

        let layout_id = g.create_pipeline_layout(shaders)?;

        let sampler = g.create_sampler(|_| {})?;

        let stack_guard = ArenaGuard::new(&tmp_alloc);

        let resource_infos = ArrayVec::<ShaderResourceInfo, {MAX_BUFFERED_FRAMES as usize}>
            ::with_len(ShaderResourceInfo { layout_id, set: 0 }, buffered_frame_count as usize)
            .unwrap();

        let mut shader_resources = ArrayVec::new();

        g.allocate_shader_resources(
            &resource_infos,
            |_, v| { shader_resources.push(v).unwrap(); },
            &stack_guard,
        )?;

        Ok(Self {
            global_resources: global_resources.clone(),
            layout_id,
            shader_resources,
            sampler,
            pipelines: GlobalVec::new(),
            last_pipeline: Default::default(),
            shaders,
        })
    }

    pub fn _update_buffered_frame_count(
        &mut self,
        buffered_frame_count: u32,
        tmp_alloc: &ArenaAlloc,
    ) -> Result<(), Error>
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
    ) -> Result<vk::Pipeline, Error>
    {
        if let Some(pipeline) = self.last_pipeline {
            if format == pipeline.1 {
                return Ok(self.global_resources.read().unwrap().get_pipeline(pipeline.0)?.handle)
            }
        }
        if let Some(pipeline) = self.pipelines.iter().find(|p| p.1 == format) {
            self.last_pipeline = Some(*pipeline);
            return Ok(self.global_resources.read().unwrap().get_pipeline(pipeline.0)?.handle)
        }
        let mut info = GraphicsPipelineInfo::new(self.layout_id);
        info.with_color_output_vk(format, WriteMask::all(), None);
        let mut pipeline = None;
        let stack_guard = ArenaGuard::new(&tmp_alloc);
        self.global_resources
            .write()
            .unwrap()
            .create_graphics_pipelines(&[info], |_, v| { pipeline = Some(v) }, &stack_guard)?;
        let pipeline = self.pipelines.push((pipeline.unwrap(), format)).unwrap();
        Ok(self.global_resources.read().unwrap().get_pipeline(pipeline.0)?.handle)
    }

    pub fn get_pipeline_layout(&mut self) -> Result<vk::PipelineLayout, Error>
    {
        Ok(self.global_resources
            .read()
            .unwrap()
            .get_pipeline_layout(self.layout_id)?
            .handle()
        )
    }

    pub fn get_descriptor_set(
        &mut self,
        image: ImageID,
        range_info: Option<ImageRangeInfo>,
        frame_index: u32,
        tmp_alloc: &ArenaAlloc,
    ) -> Result<vk::DescriptorSet, Error>
    {
        let stack_guard = ArenaGuard::new(&tmp_alloc);
        let mut g = self.global_resources.write().unwrap();
        let resource_id = self.shader_resources[frame_index as usize];
        let update = ShaderResourceImageUpdate {
            resource: resource_id,
            binding: 0,
            starting_index: 0,
            infos: &[
                ShaderResourceImageInfo {
                    sampler: self.sampler,
                    image_source: (image, range_info),
            }],
        };
        g.update_shader_resources(&[update], &[], &[], &stack_guard)?;
        g.get_descriptor_set(resource_id).map_err(Into::into)
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
            g.destroy_shader(shader).ok();
        }
        for pipeline in &self.pipelines {
            g.destroy_pipeline(pipeline.0).ok();
        }
        g.destroy_sampler(self.sampler).ok();
    }
}
