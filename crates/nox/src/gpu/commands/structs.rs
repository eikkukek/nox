use nox_ash::vk;

use crate::gpu::prelude::ShaderResourceId;

use nox_mem::{
    AsRaw, impl_as_raw_bit_op,
    vec::NonNullVec32,
};

pub(super) struct CommandResult<'a> 
{
    pub primary_command_buffers: NonNullVec32<'a, vk::CommandBuffer>,
    pub timeline_value: u64,
    pub wait_scope: vk::PipelineStageFlags2,
    pub signal_scope: vk::PipelineStageFlags2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandOrdering {
    None,
    Strict,
}

#[repr(u8)]
#[derive(Default, Clone, Copy, PartialEq, Eq, AsRaw)]
pub enum ShaderAccess {
    #[default]
    None = 0x0,
    Read = vk::AccessFlags2::SHADER_READ.as_raw() as u8,
    Write = vk::AccessFlags2::SHADER_WRITE.as_raw() as u8,
    ReadWrite = vk::AccessFlags2::SHADER_READ.as_raw() as u8 | vk::AccessFlags2::SHADER_WRITE.as_raw() as u8,
}

impl_as_raw_bit_op!(ShaderAccess);

#[derive(Clone, Copy)]
pub struct BindingBarrierInfo {
    pub set: u32,
    pub binding: u32,
    pub ordering: CommandOrdering,
    pub access: ShaderAccess,
}

impl BindingBarrierInfo {

    #[inline(always)]
    pub fn new(
        set: u32,
        binding: u32,
        ordering: CommandOrdering,
        access: ShaderAccess,
    ) -> Self {
        Self {
            set,
            binding,
            ordering,
            access,
        }
    }
}

pub(super) struct ShaderResourceBindCall {
    pub sets: NonNullVec32<'static, Option<ShaderResourceId>>,
    pub barriers: NonNullVec32<'static, BindingBarrierInfo>,
}
