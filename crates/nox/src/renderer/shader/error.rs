use ash::vk;

#[derive(Debug)]
pub enum ShaderError {
    VulkanError(vk::Result),
    Shaderc(shaderc::Error),
    SpirvCross(spirv_cross2::SpirvCrossError),
    InvalidSpirv,
}

impl From<vk::Result> for ShaderError {

    fn from(value: vk::Result) -> Self {
        Self::VulkanError(value)
    }
}

impl From<shaderc::Error> for ShaderError {

    fn from(value: shaderc::Error) -> Self {
        Self::Shaderc(value)
    }
}

impl From<spirv_cross2::SpirvCrossError> for ShaderError {

    fn from(value: spirv_cross2::SpirvCrossError) -> Self {
        Self::SpirvCross(value)
    }
}

impl core::fmt::Display for ShaderError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VulkanError(err) => write!(f, "{err}"),
            Self::Shaderc(err) => write!(f, "{err}"),
            Self::SpirvCross(err) => write!(f, "{err}"),
            Self::InvalidSpirv => write!(f, "invalid spirv, spirv binary size must be a multiple of 4"),
        }
    }
}

impl core::error::Error for ShaderError {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::VulkanError(err) => Some(err),
            Self::Shaderc(err) => Some(err),
            Self::SpirvCross(err) => Some(err),
            Self::InvalidSpirv => None,
        }
    }
}
