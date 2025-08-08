use std::sync::Arc;

use ash::vk;

use rspirv_reflect::Reflection;

use nox_mem::{vec_types::{Vector, GlobalVec}, AsRaw};

use crate::Version;

use super::{
    Handle,
    shader_fn::glsl_to_spirv,
    pipeline::create_shader_module,
    Error,
};

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum ShaderStage {
    Unknown = 0,
    Vertex = 1,
    Fragment = 2,
    Compute = 4,
}

impl From<vk::ShaderStageFlags> for ShaderStage {

    fn from(value: vk::ShaderStageFlags) -> Self {
        match value {
            vk::ShaderStageFlags::VERTEX => Self::Vertex,
            vk::ShaderStageFlags::FRAGMENT => Self::Fragment,
            vk::ShaderStageFlags::COMPUTE => Self::Compute,
            _ => Self::Unknown,
        }
    }
}

impl From<ShaderStage> for vk::ShaderStageFlags {

    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Unknown => Self::empty(),
            ShaderStage::Vertex => Self::VERTEX,
            ShaderStage::Fragment => Self::FRAGMENT,
            ShaderStage::Compute => Self::COMPUTE,
        }
    }
}

impl From<ShaderStage> for shaderc::ShaderKind {

    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Unknown => Self::AnyHit,
            ShaderStage::Vertex => Self::Vertex,
            ShaderStage::Fragment => Self::Fragment,
            ShaderStage::Compute => Self::Compute,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Uniform {
    pub stage: ShaderStage,
    pub set: u32,
    pub binding: u32,
    pub count: u32,
    pub ty: vk::DescriptorType,
}

impl From<Uniform> for vk::DescriptorSetLayoutBinding<'static> {

    fn from(value: Uniform) -> Self {
        Self {
            stage_flags: value.stage.into(),
            binding: value.binding,
            descriptor_type: value.ty,
            descriptor_count: value.count,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct PushConstant {
    pub stage: ShaderStage,
    pub offset: u32,
    pub size: u32,
}

impl From<PushConstant> for vk::PushConstantRange {

    fn from(value: PushConstant) -> Self {
        Self {
            stage_flags: value.stage.into(),
            offset: value.offset,
            size: value.size,
        }
    }
}

impl From<vk::PushConstantRange> for PushConstant {

    fn from(value: vk::PushConstantRange) -> Self {
        Self {
            stage: value.stage_flags.into(),
            offset: value.offset,
            size: value.size,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Shader {
    device: Arc<ash::Device>,
    module: vk::ShaderModule,
    uniforms: GlobalVec<Uniform>,
    push_constant: Option<PushConstant>,
    stage: ShaderStage,
}

impl Shader {

    pub fn new(
        device: Arc<ash::Device>,
        input: &str,
        name: &str,
        stage: ShaderStage,
        vulkan_version: Version,
    ) -> Result<Self, Error>
    {
        let spirv = glsl_to_spirv(
            input,
            name,
            stage.into(),
            vulkan_version,
        )?;
        let reflect = Reflection::new_from_spirv(spirv.as_binary_u8())?;
        let sets = reflect.get_descriptor_sets()?;
        let mut uniforms = GlobalVec::with_capacity(sets.len()).unwrap();
        for (set, info) in sets {
            for (binding, info) in info {
                let count = match info.binding_count {
                    rspirv_reflect::BindingCount::One => 1,
                    rspirv_reflect::BindingCount::StaticSized(v) => v as u32,
                    rspirv_reflect::BindingCount::Unbounded => 1,
                };
                uniforms.push(Uniform {
                    stage,
                    set,
                    binding,
                    count,
                    ty: vk::DescriptorType::from_raw(info.ty.0 as i32),
                }).unwrap();
            }
        }
        let push_constant = reflect
            .get_push_constant_range()?
            .map(|v| {
                PushConstant {
                    stage,
                    offset: v.offset,
                    size: v.size,
                }
            });
        let module = create_shader_module(&device, spirv.as_binary())?;
        Ok(Self {
            device,
            module,
            uniforms,
            push_constant,
            stage,
        })
    }

    #[inline(always)]
    pub fn stage(&self) -> ShaderStage {
        self.stage
    }

    #[inline(always)]
    pub fn shader_module(&self) -> Handle<'_, vk::ShaderModule> {
        Handle::new(self.module)
    }

    #[inline(always)]
    pub fn uniforms(&self) -> &[Uniform] {
        &self.uniforms
    }

    #[inline(always)]
    pub fn push_constant(&self) -> Option<PushConstant> {
        self.push_constant
    }
}

impl Drop for Shader {
    
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.module, None);
        }
    }
}
