use std::sync::Arc;

use ash::vk;

use spirv_cross2::{
    reflect::{self, DecorationValue, ResolveSize, ResourceType, TypeInner}, spirv::{self}, targets, Compiler, Module
};

use nox_mem::{
    vec_types::GlobalVec,
    AsRaw,
};

use super::{
    Handle,
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
pub struct SpecializationConstant<T> {
    pub value: T,
    pub id: u32,
}

#[derive(Clone)]
pub struct Uniform {
    pub stage: ShaderStage,
    pub set: u32,
    pub binding: u32,
    pub count: u32,
    pub count_specialization: GlobalVec<SpecializationConstant<u32>>,
    pub ty: vk::DescriptorType,
}

impl Uniform {

    pub fn to_vk(
        &self,
        count_specialization: &[SpecializationConstant<u32>]
    ) -> vk::DescriptorSetLayoutBinding<'static> {
        let mut count = self.count;
        for spec in &self.count_specialization {
            if let Some(value) = count_specialization
                .iter()
                .find(|v| v.id == spec.id)
            {
                count *= value.value;
            }
            else {
                count *= spec.value;
            }
        }
        vk::DescriptorSetLayoutBinding {
            stage_flags: self.stage.into(),
            binding: self.binding,
            descriptor_type: self.ty,
            descriptor_count: count,
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
    push_constants: GlobalVec<PushConstant>,
    stage: ShaderStage,
}

impl Shader {

    pub fn new(
        device: Arc<ash::Device>,
        spirv: &[u32],
        stage: ShaderStage,
    ) -> Result<Self, Error>
    {
        let compiler = Compiler::<targets::None>::new(Module::from_words(spirv))?;
        let mut uniforms = GlobalVec::new();
        let mut push_constants = GlobalVec::new();
        let resources = compiler.shader_resources()?;
        let mut parse_uniform = |mut ty: vk::DescriptorType, resource_ty: ResourceType| -> Result<(), Error> {
            for resource in resources.resources_for_type(resource_ty)? {
                let mut set = 0;
                if let Some(DecorationValue::Literal(dec)) = compiler.decoration(resource.id, spirv::Decoration::DescriptorSet)? {
                    set = dec;
                }
                let mut binding = 0;
                if let Some(DecorationValue::Literal(dec)) = compiler.decoration(resource.id, spirv::Decoration::Binding)? {
                    binding = dec;
                }
                let mut count = 1;
                let mut count_specialization = GlobalVec::new();
                let mut desc = compiler.type_description(resource.base_type_id)?;
                while let TypeInner::Array { base, storage: _, dimensions, stride: _ } = desc.inner {
                    for dim in dimensions {
                        match dim {
                            reflect::ArrayDimension::Literal(n) => count *= n,
                            reflect::ArrayDimension::Constant(spec) => {
                                count_specialization.push(SpecializationConstant {
                                    value: compiler.specialization_constant_value(spec)?,
                                    id: spec.id(),
                                });
                            },
                        }
                    }
                    desc = compiler.type_description(base)?;
                }
                if let TypeInner::Image(img) = &desc.inner {
                    match img.class {
                        reflect::ImageClass::Sampled { depth: _, multisampled: _, arrayed: _ } => {
                            ty = vk::DescriptorType::COMBINED_IMAGE_SAMPLER;
                        },
                        reflect::ImageClass::Texture { multisampled: _, arrayed: _ } => {
                            ty = vk::DescriptorType::SAMPLED_IMAGE;
                        },
                        reflect::ImageClass::Storage { format: _ } => {
                            ty = vk::DescriptorType::STORAGE_IMAGE;
                        },
                    };
                };
                if desc.inner == TypeInner::Sampler {
                    ty = vk::DescriptorType::SAMPLER;
                }
                uniforms.push(Uniform {
                    stage,
                    set,
                    binding,
                    count,
                    count_specialization,
                    ty,
                });
            }
            Ok(())
        };
        parse_uniform(vk::DescriptorType::UNIFORM_BUFFER, ResourceType::UniformBuffer)?;
        parse_uniform(vk::DescriptorType::STORAGE_BUFFER, ResourceType::StorageBuffer)?;
        parse_uniform(vk::DescriptorType::SAMPLED_IMAGE, ResourceType::SampledImage)?;
        parse_uniform(vk::DescriptorType::STORAGE_IMAGE, ResourceType::StorageImage)?;
        for resource in resources.resources_for_type(ResourceType::PushConstant)? {
            let desc = compiler.type_description(resource.base_type_id)?;
            let size = match desc.size_hint {
                reflect::TypeSizeHint::Static(hint) => hint as u32,
                reflect::TypeSizeHint::RuntimeArray(hole) => hole.declared() as u32,
                reflect::TypeSizeHint::Matrix(hole) => hole.declared() as u32,
                reflect::TypeSizeHint::UnknownArrayStride(hole) => hole.declared() as u32,
            };
            let mut offset = 0;
            if let Some(DecorationValue::Literal(dec))
                = compiler.member_decoration_by_handle(resource.base_type_id, 0, spirv::Decoration::Offset)?
            {
                offset = dec;
            }
            push_constants.push(PushConstant {
                stage,
                size,
                offset,
            });
        }
        let module = create_shader_module(&device, spirv)?;
        Ok(Self {
            device,
            module,
            uniforms,
            push_constants,
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
    pub fn push_constants(&self) -> &[PushConstant] {
        &self.push_constants
    }
}

impl Drop for Shader {
    
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.module, None);
        }
    }
}
