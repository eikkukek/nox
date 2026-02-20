use nox_error::Error;

#[derive(Error, Debug)]
pub enum ShaderError {

    #[display("Source was None")]
    NoSource,

    #[display("{0}")]
    Shaderc(#[from] shaderc::Error),

    #[display("{0}")]
    SpirvCross(#[from] spirv_cross2::SpirvCrossError),

    #[display("invalid spirv, spirv binary size must be a multiple of 4")]
    InvalidSpirv,
}

pub(super) type Result<T> = core::result::Result<T, ShaderError>;
