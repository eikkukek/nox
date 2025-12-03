use ash::vk;
use nox_mem::AsRaw;

use core::fmt::Display;

#[repr(u32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug)]
pub enum MSAA {
    None = 0,
    #[default]
    X1 = vk::SampleCountFlags::TYPE_1.as_raw(),
    X2 = vk::SampleCountFlags::TYPE_2.as_raw(),
    X4 = vk::SampleCountFlags::TYPE_4.as_raw(),
    X8 = vk::SampleCountFlags::TYPE_8.as_raw(),
    X16 = vk::SampleCountFlags::TYPE_16.as_raw(),
    X32 = vk::SampleCountFlags::TYPE_32.as_raw(),
    X64 = vk::SampleCountFlags::TYPE_64.as_raw(),
}

impl From<MSAA> for vk::SampleCountFlags {
    
    fn from(value: MSAA) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum CompareOp {
    Never = vk::CompareOp::NEVER.as_raw(),
    Less = vk::CompareOp::LESS.as_raw(),
    Equal = vk::CompareOp::EQUAL.as_raw(),
    LessOrEqual = vk::CompareOp::LESS_OR_EQUAL.as_raw(),
    Greater = vk::CompareOp::GREATER.as_raw(),
    NotEqual = vk::CompareOp::NOT_EQUAL.as_raw(),
    GreaterOrEqual = vk::CompareOp::GREATER_OR_EQUAL.as_raw(),
    Always = vk::CompareOp::ALWAYS.as_raw(),
}

impl From<CompareOp> for vk::CompareOp {

    fn from(value: CompareOp) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug)]
pub enum PipelineStage {
    ComputeShader = vk::PipelineStageFlags::COMPUTE_SHADER.as_raw(),
    Transfer = vk::PipelineStageFlags::TRANSFER.as_raw(),
    AllGraphics = vk::PipelineStageFlags::ALL_GRAPHICS.as_raw(),
    AllCommands = vk::PipelineStageFlags::ALL_COMMANDS.as_raw(),
}

impl From<PipelineStage> for vk::PipelineStageFlags {

    fn from(value: PipelineStage) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum QueueFamily {
    Graphics,
    Transfer,
    Compute,
}

impl Display for QueueFamily {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Graphics => {
                write!(f, "Graphics")
            },
            Self::Transfer => {
                write!(f, "Transfer")
            },
            Self::Compute => {
                write!(f, "Compute")
            }
        }
    }
}
