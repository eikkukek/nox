mod shader_fn;
mod enums;

use core::{
    ffi::CStr,
    hash::{self, Hash},
    ptr::NonNull,
};

use nox_ash::vk;

use spirv_cross2::{
    reflect::{self, DecorationValue, ResolveSize, ResourceType, TypeInner},
    spirv::{self}, targets, Compiler, Module,
};

use compact_str::format_compact;

use nox_mem::{
    alloc::{LocalAllocExt, StdAlloc}, vec::Vec32
};
use nox_threads::futures::{
    future::RemoteHandle,
};

use shader_fn::glsl_to_spirv;

use crate::{
    Version,
    error::Context,
    error::*,
    sync::*,
    gpu::*,
};

pub use enums::*;

#[derive(Clone, Copy)]
pub enum ShaderSource<'a> {
    Spirv(&'a [u32]),
    Glsl(&'a str),
}

#[derive(Clone)]
pub enum ShaderSourceOwned {
    Spirv(Box<[u32]>),
    Glsl(Box<str>),
}

impl<'a> ShaderSource<'a> {

    pub fn compile(
        self,
        api_version: Version,
        stage: ShaderStage,
        name: &str,
    ) -> Result<ShaderSourceCompiled>
    {
        match self {
            Self::Spirv(bin) => unsafe {
                let len = bin.len();
                assert!(len <= u32::MAX as usize);
                let spirv = StdAlloc
                    .alloc_uninit(len)
                    .expect("global alloc failed");
                bin.as_ptr()
                    .copy_to_nonoverlapping(spirv.as_ptr(), len);
                Ok(ShaderSourceCompiled {
                    spirv,
                    spirv_len: len as u32,
                    stage,
                })
            },
            Self::Glsl(input) => unsafe {
                let comp = glsl_to_spirv(input, name, stage.into(), api_version)
                .context("failed to compile glsl to spirv")?;
                let bin = comp.as_binary();
                let len = bin.len();
                assert!(len <= u32::MAX as usize);
                let spirv = StdAlloc
                    .alloc_uninit(len)
                    .expect("global alloc failed");
                bin.as_ptr()
                    .copy_to_nonoverlapping(spirv.as_ptr(), len);
                Ok(ShaderSourceCompiled {
                    spirv,
                    spirv_len: len as u32,
                    stage,
                })
            },
        }
    }

    #[inline(always)]
    pub fn to_owned(&self) -> ShaderSourceOwned {
        match *self {
            Self::Spirv(bin) => ShaderSourceOwned::Spirv(Box::from(bin)),
            Self::Glsl(glsl) => ShaderSourceOwned::Glsl(Box::from(glsl)),
        }
    }
}

impl ShaderSourceOwned {

    #[inline(always)]
    pub fn borrow(&self) -> ShaderSource<'_> {
        match self {
            ShaderSourceOwned::Spirv(bin) => ShaderSource::Spirv(bin),
            ShaderSourceOwned::Glsl(glsl) => ShaderSource::Glsl(glsl),
        }
    }
}

pub struct ShaderSourceCompiled {
    spirv: NonNull<u32>,
    spirv_len: u32,
    stage: ShaderStage,
}

unsafe impl Send for ShaderSourceCompiled {}
unsafe impl Sync for ShaderSourceCompiled {}

impl ShaderSourceCompiled {

    #[inline(always)]
    pub fn stage(&self) -> ShaderStage {
        self.stage
    }
    
    #[inline(always)]
    pub fn spirv(&self) -> &[u32] {
        unsafe {
            core::slice::from_raw_parts(
                self.spirv.as_ptr(),
                self.spirv_len as usize,
            )
        }
    }
}

impl Clone for ShaderSourceCompiled {

    fn clone(&self) -> Self {
        unsafe {
            let count = self.spirv_len as usize;
            let spirv = StdAlloc
                .alloc_uninit(count)
                .expect("global alloc failed");
            self.spirv
                .copy_to_nonoverlapping(spirv, count);
            ShaderSourceCompiled {
                spirv,
                spirv_len: self.spirv_len,
                stage: self.stage,
            }
        }
    }
}

impl Drop for ShaderSourceCompiled {

    fn drop(&mut self) {
        unsafe {
           StdAlloc 
                .free_uninit(self.spirv, self.spirv_len as usize);
        }
    }
}

pub struct ShaderAttributes<'a> {
    source: Option<ShaderSource<'a>>,
    stage: ShaderStage,
    name: &'a str,
    entry_point: &'a CStr,
}

pub struct ShaderAttributesOwned {
    source: Option<ShaderSourceOwned>,
    stage: ShaderStage,
    name: Arc<str>,
    entry_point: Arc<CStr>,
}

impl<'a> ShaderAttributes<'a> {

    /// Sets the source of the shader to Glsl with the given `input`. The source needs to be valid
    /// Glsl for Vulkan.
    ///
    /// The default source is `None`.
    ///
    /// Note that for the shader to compile, a source needs to be explicitly specified.
    #[inline(always)]
    pub fn with_glsl(mut self, input: &'a str) -> Self {
        self.source = Some(ShaderSource::Glsl(input));
        self
    }
    /// Sets the source of the shader to Spir-V with the given binary. The source needs to be valid
    /// Spir-V for Vulkan.
    ///
    /// The default source is `None`.
    ///
    /// Note that for the shader to compile, a source needs to be explicitly specified.
    #[inline(always)]
    pub fn with_spirv(mut self, bin: &'a [u32]) -> Self {
        self.source = Some(ShaderSource::Spirv(bin));
        self
    }

    /// Sets the `stage` of the shader.
    ///
    /// The default stage is [`ShaderStage::Unknown`].
    ///
    /// Note that for the shader to compile, the shader stage needs to be explicitly specified as
    /// something other than [`ShaderStage::Unknown`].
    #[inline(always)]
    pub fn with_stage(mut self, stage: ShaderStage) -> Self {
        self.stage = stage;
        self
    }

    /// Sets the `name` for the shader. The default name is an empty string.
    /// 
    /// The name is used mainly for debugging.
    #[inline(always)]
    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    /// Sets the entry point for the shader. A shader can have multiple entry points, so you need
    /// to specify which entry point to use for pipelines and shader reflection. The default entry
    /// point is `c"main"`.
    #[inline(always)]
    pub fn with_entry_point(mut self, name: &'a CStr) -> Self {
        self.entry_point = name;
        self
    }

    #[inline(always)]
    pub fn to_owned(self) -> ShaderAttributesOwned {
        ShaderAttributesOwned {
            source: self.source.map(|s| s.to_owned()),
            stage: self.stage,
            name: self.name.into(),
            entry_point: self.entry_point.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpecializationConstant<T> {
    pub value: T,
    pub constant_id: u32,
}

#[derive(Clone)]
pub struct Uniform {
    pub stage: ShaderStage,
    pub input_attachment_index: Option<u32>,
    pub set: u32,
    pub binding: u32,
    pub ty: DescriptorType,
    pub name: Arc<str>,
    pub count: u32,
    pub count_specialization: Vec32<SpecializationConstant<u32>>,
    pub struct_size: Option<u32>,
}

impl Uniform {

    pub(crate) fn as_layout_binding(
        &self,
        count_specialization: &[SpecializationConstant<u32>],
        inline_uniform_block: bool,
    ) -> Result<DescriptorSetLayoutBinding> {
        let mut count = self.count;
        for spec in &self.count_specialization {
            if let Some(spec) = count_specialization
                .iter()
                .find(|v| v.constant_id == spec.constant_id)
            {
                count *= spec.value;
            }
            else {
                count *= spec.value;
            }
        }
        let mut ty = self.ty;
        if inline_uniform_block {
            if !matches!(ty, DescriptorType::UniformBuffer) {
                return Err(Error::just_context(format_compact!(
                    "invalid cast of descriptor type {} to inline uniform block", ty,
                )))
            }
            ty = DescriptorType::InlineUniformBlock;
            count *= self.struct_size.ok_or_else(|| Error::just_context(format_compact!(
                "struct size was none"
            )))?;
        }
        Ok(DescriptorSetLayoutBinding {
            binding: self.binding,
            descriptor_type: ty,
            name: self.name.clone(),
            descriptor_count: count,
            stage_flags: self.stage.into(),
            struct_size: self.struct_size,
        })
    }
}

#[derive(Clone)]
pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
    pub name: Arc<str>,
    pub struct_size: Option<u32>,
}

impl Hash for DescriptorSetLayoutBinding {

    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.binding.hash(state);
        self.descriptor_type.hash(state);
        self.descriptor_count.hash(state);
        self.stage_flags.hash(state);
    }
}

impl PartialEq for DescriptorSetLayoutBinding {

    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.binding == other.binding &&
        self.descriptor_type == other.descriptor_type &&
        self.descriptor_count == other.descriptor_count &&
        self.stage_flags == other.stage_flags
    }
}

impl Eq for DescriptorSetLayoutBinding {}

impl From<DescriptorSetLayoutBinding> for vk::DescriptorSetLayoutBinding<'_> {

    #[inline(always)]
    fn from(value: DescriptorSetLayoutBinding) -> Self {
        Self {
            binding: value.binding,
            descriptor_type: value.descriptor_type.into(),
            descriptor_count: value.descriptor_count,
            stage_flags: value.stage_flags.into(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct PushConstantRange {
    pub stage: ShaderStage,
    pub offset: u32,
    pub size: u32,
}

impl From<PushConstantRange> for vk::PushConstantRange {

    fn from(value: PushConstantRange) -> Self {
        Self {
            stage_flags: value.stage.into(),
            offset: value.offset,
            size: value.size,
        }
    }
}

#[derive(Clone)]
pub struct ShaderInner {
    compiled: ShaderSourceCompiled,
    entry_point: Arc<CStr>,
    uniforms: Vec32<Uniform>,
    push_constant_ranges: Vec32<PushConstantRange>,
    stage: ShaderStage,
}

impl ShaderInner {

    #[inline(always)]
    pub fn source(&self) -> &ShaderSourceCompiled {
        &self.compiled
    }

    #[inline(always)]
    pub fn entry_point(&self) -> &CStr {
        &self.entry_point
    }

    #[inline(always)]
    pub fn stage(&self) -> ShaderStage {
        self.stage
    }

    #[inline(always)]
    pub fn uniforms(&self) -> &[Uniform] {
        &self.uniforms
    }

    #[inline(always)]
    pub fn push_constant_ranges(&self) -> &[PushConstantRange] {
        &self.push_constant_ranges
    }
}

#[derive(Clone)]
pub struct Shader {
    inner: Arc<FutureLock<ShaderInner, RemoteHandle<Result<ShaderInner>>>>
}

/// Creates default [`ShaderAttributes`].
///
/// For the attributes to be valid, a [`ShaderStage`] and [`ShaderSource`] need to be specified.
///
/// If the shader has a different entry point from `c"main"`, that also needs to be specified.
#[inline]
pub fn default_shader_attributes<'a>() -> ShaderAttributes<'a> {
    ShaderAttributes {
        source: None,
        stage: ShaderStage::Vertex,
        name: "",
        entry_point: c"main",
    }
}

impl Shader {

    pub fn new(
        gpu: &Gpu,
        attributes: ShaderAttributes<'_>,
    ) -> Result<Self>
    {
        let attributes = attributes.to_owned();
        let g = gpu.clone();
        let fut = async move {
            let Some(source) = attributes.source else {
                return Err(Error::just_context("no source given for shader"))
            };
            let execution_model = attributes.stage.execution_model();
            let compiled = source
                .borrow()
                .compile(g.device().api_version(), attributes.stage, &attributes.name)
                .context("failed to compile shader")?;
            let spirv = compiled.spirv();
            let stage = attributes.stage;
            let mut compiler = Compiler::<targets::None>::new(Module::from_words(spirv))
                .context("failed to create compiler for shader reflection")?;
            compiler.set_entry_point(attributes.entry_point.as_ref(), execution_model,
            ).context_with(|| format_compact!(
                "failed to set entry point {:?} for shader", attributes.entry_point,
            ))?;
            let mut uniforms = Vec32::with_capacity(4);
            let mut push_constant_ranges = Vec32::with_capacity(4);
            let resources = compiler
                .shader_resources()
                .context("failed to reflect")?;
            let mut parse_uniform = |
                mut ty: Option<DescriptorType>,
                resource_ty: ResourceType
            | -> core::result::Result<(), spirv_cross2::SpirvCrossError> {
                for resource in resources.resources_for_type(resource_ty)? {
                    let mut set = 0;
                    if let Some(DecorationValue::Literal(dec)) =
                        compiler.decoration(resource.id, spirv::Decoration::DescriptorSet)?
                    {
                        set = dec;
                    }
                    let mut binding = 0;
                    if let Some(DecorationValue::Literal(dec)) =
                        compiler.decoration(resource.id, spirv::Decoration::Binding)?
                    {
                        binding = dec;
                    }
                    let mut input_attachment_index = None;
                    if let Some(DecorationValue::Literal(dec)) =
                        compiler.decoration(resource.id, spirv::Decoration::InputAttachmentIndex)?
                    {
                        input_attachment_index = Some(dec);
                    }
                    let mut count = 1;
                    let mut count_specialization = Vec32::new();
                    let mut desc = compiler.type_description(resource.base_type_id)?;
                    while let TypeInner::Array { base, storage: _, dimensions, stride: _ } = desc.inner {
                        for dim in dimensions {
                            match dim {
                                reflect::ArrayDimension::Literal(n) => count *= n,
                                reflect::ArrayDimension::Constant(spec) => {
                                    count_specialization.push(SpecializationConstant {
                                        value: compiler.specialization_constant_value(spec)?,
                                        constant_id: spec.id(),
                                    });
                                },
                            }
                        }
                        desc = compiler.type_description(base)?;
                    }
                    let struct_size =
                    match &desc.inner {
                        TypeInner::Struct(s) => { Some(s.size as u32) },
                        TypeInner::Scalar(s) => Some(s.size.byte_size() as u32),
                        TypeInner::Vector { width, scalar } => Some(width * scalar.size.byte_size() as u32),
                        TypeInner::Matrix { columns, rows, scalar } =>
                            Some(columns * rows * scalar.size.byte_size() as u32),
                        _ => None,
                    };
                    if ty.is_none() &&
                        let TypeInner::Image(img) = &desc.inner
                    {
                        if resource_ty == ResourceType::SampledImage {
                            if img.dimension == spirv::Dim::DimBuffer {
                                ty = Some(DescriptorType::UniformTexelBuffer);
                            }
                        } else if resource_ty == ResourceType::SeparateImage {
                            ty =
                                if img.dimension == spirv::Dim::DimBuffer {
                                    Some(DescriptorType::UniformTexelBuffer)
                                } else {
                                    Some(DescriptorType::SampledImage)
                                };
                        } else if resource_ty == ResourceType::StorageImage {
                            ty =
                                if img.dimension == spirv::Dim::DimBuffer {
                                    Some(DescriptorType::StorageTexelBuffer)
                                } else {
                                    Some(DescriptorType::StorageImage)
                                };
                        }
                    };
                    if desc.inner == TypeInner::Sampler {
                        ty = Some(DescriptorType::Sampler);
                    }
                    uniforms.push(Uniform {
                        stage,
                        input_attachment_index,
                        set,
                        binding,
                        ty: ty.unwrap_or(DescriptorType::UniformBuffer),
                        name: compiler.name(resource.id)?
                            .map(|name| name.as_ref().into())
                            .unwrap_or(Default::default()),
                        count,
                        count_specialization,
                        struct_size,
                    });
                }
                Ok(())
            };
            parse_uniform(Some(DescriptorType::UniformBuffer), ResourceType::UniformBuffer)
                .context("failed to reflect uniform buffers")?;
            parse_uniform(Some(DescriptorType::StorageImage), ResourceType::StorageBuffer)
                .context("failed to reflect storage buffers")?;
            parse_uniform(Some(DescriptorType::CombinedImageSampler), ResourceType::SampledImage)
                .context("failed to reflect sampled images")?;
            parse_uniform(Some(DescriptorType::Sampler), ResourceType::SeparateSamplers)
                .context("failed to reflect separate images")?;
            parse_uniform(None, ResourceType::SeparateImage)
                .context("failed to reflect separate images")?;
            parse_uniform(None, ResourceType::StorageImage)
                .context("failed to reflect storage images")?;
            parse_uniform(Some(DescriptorType::InputAttachment), ResourceType::SubpassInput)
                .context("failed to reflect storage images")?;
            for resource in resources
                .resources_for_type(ResourceType::PushConstant)
                .context("failed to reflect push constants")?
            {
                let desc = compiler.type_description(resource.base_type_id)
                    .context("failed to reflect push constants")?;
                let size = match desc.size_hint {
                    reflect::TypeSizeHint::Static(hint) => hint as u32,
                    reflect::TypeSizeHint::RuntimeArray(hole) => hole.declared() as u32,
                    reflect::TypeSizeHint::Matrix(hole) => hole.declared() as u32,
                    reflect::TypeSizeHint::UnknownArrayStride(hole) => hole.declared() as u32,
                };
                let mut offset = 0;
                if let Some(DecorationValue::Literal(dec)) =
                    compiler.member_decoration_by_handle(resource.base_type_id, 0, spirv::Decoration::Offset)
                    .context("failed to reflect push constants")?
                {
                    offset = dec;
                }
                push_constant_ranges.push(PushConstantRange {
                    stage,
                    size,
                    offset,
                });
            }
            Ok(ShaderInner {
                compiled,
                entry_point: attributes.entry_point,
                uniforms,
                push_constant_ranges,
                stage,
            })
        };
        Ok(Self {
            inner: Arc::new(FutureLock::new(
                gpu.thread_pool()
                    .spawn_with_handle(fut)
                    .context("spawn error")?,
            ))
        })
    }

    #[inline(always)]
    pub async fn inner(&self) -> Result<&ShaderInner> {
        self.inner.load().await
    }
}
