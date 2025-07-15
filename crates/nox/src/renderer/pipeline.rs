pub mod graphics;
pub mod layout;
mod pipeline_cache;

pub use pipeline_cache::{PipelineID, PipelineCache};

use ash::vk;

use nox_mem::{Allocator, Vector, FixedVec};

use crate::renderer::Error;

pub fn create_transient_graphics_pipelines<'alloc, Alloc: Allocator>(
    device: ash::Device,
    pipeline_infos: &[(graphics::CreateInfos, u32, u32, u32)],
    vertex_input_state_infos: &[vk::PipelineVertexInputStateCreateInfo],
    stages: &[&[vk::PipelineShaderStageCreateInfo]],
    layouts: &[vk::PipelineLayout],
    alloc: &'alloc Alloc,
) -> Result<FixedVec<'alloc, vk::Pipeline, Alloc>, Error>
{
    let pipeline_count = pipeline_infos.len();
    let mut create_infos = FixedVec::with_capacity(pipeline_count, alloc)?;
    for info in pipeline_infos {
        let create_info = &info.0;
        let vertex_input_state = &vertex_input_state_infos[info.1 as usize];
        let stages = stages[info.2 as usize];
        let layout = layouts[info.3 as usize];
        let vk_create_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: (&create_info.rendering_info as *const _) as _,
            stage_count: stages.len() as u32,
            p_stages: stages.as_ptr(),
            p_vertex_input_state: vertex_input_state,
            p_input_assembly_state: &create_info.input_assembly_state,
            p_tessellation_state: &create_info.tesellation_state,
            p_viewport_state: core::ptr::null(),
            p_rasterization_state: &create_info.rasterization_state,
            p_multisample_state: &create_info.multisample_state,
            p_depth_stencil_state: &create_info.depth_stencil_state,
            p_color_blend_state: &create_info.color_blend_state,
            p_dynamic_state: &create_info.dynamic_state,
            layout,
            ..Default::default()
        };
        create_infos.push(vk_create_info).unwrap();
    }
    let mut pipelines = FixedVec::with_capacity(create_infos.len(), alloc)?;
    unsafe {
        let create_graphics_pipelines = device.fp_v1_0().create_graphics_pipelines;
        let result = create_graphics_pipelines(
            device.handle(),
            vk::PipelineCache::null(),
            create_infos.len() as u32,
            create_infos.as_ptr(),
            core::ptr::null(),
            pipelines.as_mut_ptr(),
        );
        if result != vk::Result::SUCCESS {
            return Err(result.into())
        }
    }
    Ok(pipelines)
}
