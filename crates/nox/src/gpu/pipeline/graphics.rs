mod definitions;
mod create_info;

pub use definitions::*;
pub use create_info::*;

use nox_ash::vk;

use compact_str::format_compact;

use nox_mem::{
    alloc::LocalAlloc,
    slice::{self, AllocSlice},
    vec::{FixedVec32, Vec32, Vector},
    dynamic::Dyn,
    vec32,
};

use crate::{
    gpu::prelude::{dev_error::*, *},
    sync::Arc,
};

#[derive(Clone)]
pub struct GraphicsPipeline {
    handle: PipelineHandle,
    samples: MsaaSamples,
    vertex_input_bindings: Vec32<VertexInputBinding>,
    viewports: Arc<[Viewport]>,
    scissors: Arc<[Scissor]>,
    color_outputs_and_dynamic_states: Arc<[i32]>,
    n_color_output_formats: u32,
    depth_output_format: vk::Format,
    stencil_output_format: vk::Format,
}

impl GraphicsPipeline {

    #[inline(always)]
    pub(super) unsafe fn new(
        vk: Arc<Vulkan>,
        handle: vk::Pipeline,
        shader_set: Arc<ShaderSetInner>,
        create_info: &GraphicsPipelineCreateInfo,
    ) -> Self {
        let n_color_output_formats = create_info.color_output_formats.len() as usize;
        let color_outputs_and_dynamic_states =
        unsafe {
            let data: Arc<[i32]> = Arc::uninit_slice(
                n_color_output_formats + create_info.dynamic_states.len() as usize
            );
            let data = Arc::into_raw(data);
            let ptr = Dyn::raw_parts(data).data.cast_mut();
            slice::cast::<_, i32>(&create_info.color_output_formats).unwrap().as_ptr()
                .copy_to_nonoverlapping(
                    ptr,
                    n_color_output_formats,
                );
            slice::cast::<_, i32>(&create_info.dynamic_states).unwrap().as_ptr()
                .copy_to_nonoverlapping(
                    ptr.add(n_color_output_formats),
                    create_info.dynamic_states.len() as usize
                );
            Arc::from_raw(data)
        };
        Self {
            handle: unsafe {
                PipelineHandle::new(vk, handle, shader_set)
            },
            samples: create_info.sample_shading_info
                .map(|info| info.samples)
                .unwrap_or(MsaaSamples::X1),
            viewports: create_info.viewports.clone(),
            scissors: create_info.scissors.clone(),
            vertex_input_bindings: create_info.vertex_input_bindings.clone(),
            color_outputs_and_dynamic_states,
            n_color_output_formats: n_color_output_formats as u32,
            depth_output_format: create_info.depth_output_format,
            stencil_output_format: create_info.stencil_output_format,
        }
    }

    #[inline(always)]
    pub(crate) fn handle(&self) -> &PipelineHandle {
        &self.handle
    }

    #[inline(always)]
    pub fn samples(&self) -> MsaaSamples {
        self.samples
    }

    #[inline(always)]
    pub fn vertex_input_bindings(&self) -> &[VertexInputBinding] {
        &self.vertex_input_bindings
    }

    #[inline(always)]
    pub fn color_output_formats(&self) -> &[VkFormat] {
        unsafe {
            slice::cast(
                &self.color_outputs_and_dynamic_states[0..self.n_color_output_formats as usize]
            ).unwrap()
        }
    }

    #[inline(always)]
    pub fn viewports(&self) -> &[Viewport] {
        &self.viewports
    }

    #[inline(always)]
    pub fn scissors(&self) -> &[Scissor] {
        &self.scissors
    }

    #[inline(always)]
    pub fn depth_output_format(&self) -> VkFormat {
        self.depth_output_format
    }

    #[inline(always)]
    pub fn stencil_output_format(&self) -> VkFormat {
        self.stencil_output_format
    }

    #[inline(always)]
    pub fn dynamic_states(&self) -> &[DynamicState] {
        unsafe {
            slice::cast(
                &self.color_outputs_and_dynamic_states[self.n_color_output_formats as usize..]
            ).unwrap()
        }
    }

    #[inline(always)]
    pub fn has_dynamic_state(&self, dynamic_state: DynamicState) -> bool {
        self.dynamic_states().iter().any(|&state| state == dynamic_state)
    }
}
