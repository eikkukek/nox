use ash::vk;

use nox_error::Error;

#[derive(Error, Debug)]
pub enum ShaderError {

    #[display("{0}")]
    VulkanError(#[from] #[source] vk::Result),

    #[display("{0}")]
    Shaderc(#[from] #[source] shaderc::Error),

    #[display("{0}")]
    SpirvCross(#[from] #[source] spirv_cross2::SpirvCrossError),

    #[display("invalid spirv, spirv binary size must be a multiple of 4")]
    InvalidSpirv,
}
