pub mod vertex_input;
mod layout;
mod graphics;
mod compute;

pub(crate) use layout::*;

pub use graphics::*;
pub use compute::*;

use ash::vk;

#[inline(always)]
pub(crate) fn create_shader_module(device: &ash::Device, spirv: &[u32]) -> Result<vk::ShaderModule, vk::Result> {
    let create_info = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        code_size: spirv.len() * size_of::<u32>(),
        p_code: spirv.as_ptr(),
        ..Default::default()
    };
    unsafe {
        Ok(device.create_shader_module(&create_info, None)?)
    }
}
