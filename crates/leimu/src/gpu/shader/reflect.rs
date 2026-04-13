use core::ffi::CStr;

use nox_mem::{
    vec::Vec32,
    vec32,
};

use crate::gpu::prelude::*;

use nox_spirv::op;

impl<'a> ShaderReflector<'a> for nox_spirv::Reflector<'a> {

    type Error = nox_spirv::ReflectError;

    fn new(spirv: &'a [u32], entry_point: &CStr, stage: ShaderStage) -> Result<Self, Self::Error> {
        let mut reflector = Self::new(spirv)?;
        let execution_model = match stage {
            ShaderStage::Vertex => op::ExecutionModel::VERTEX,
            ShaderStage::TesellationControl => op::ExecutionModel::TESELLATION_CONTROL,
            ShaderStage::TesellationEvaluation => op::ExecutionModel::TESELLATION_EVAULATION,
            ShaderStage::Geometry => op::ExecutionModel::GEOMETRY,
            ShaderStage::Fragment => op::ExecutionModel::FRAGMENT,
            ShaderStage::Compute => op::ExecutionModel::GL_COMPUTE,
        };
        reflector.set_entry_point(entry_point, execution_model)?;
        Ok(reflector)
    }

    fn parse_uniforms(
        &mut self,
        resource_type: ShaderResourceType,
    ) -> Result<Vec32<Uniform>, Self::Error> {
        let mut uniforms = vec32![];
        for resource in self.resources_for_type(resource_type.into()) {
            let mut set = 0;
            let mut binding = 0;
            let mut input_attachment_index = None;
            for decoration in self.get_decorations(resource.variable_id.to_any()) {
                if let nox_spirv::Decoration::DesrciptorSet(x) = decoration {
                    set = x;
                } else if let nox_spirv::Decoration::Binding(x) = decoration {
                    binding = x;
                } else if let nox_spirv::Decoration::InputAttachmentIndex(x) = decoration {
                    input_attachment_index = Some(x);
                };
            }
            let mut count_specialization = vec32![];
            let mut desc = self.get_type_description(resource.base_type_id)?;
            if let nox_spirv::TypeInner::Array { element_type, length } = desc.inner {
                if let Some(ty) = self.get_constant(id)
            };
            while let S = desc.inner
            {
                for dim in dimensions {
                    match dim {
                        ArrayDimension::Literal(n) => count *= n,
                        ArrayDimension::Constant(spec) => {
                            count_specialization.push(crate::gpu::SpecializationConstant {
                                value: self.compiler.specialization_constant_value(spec)?,
                                constant_id: spec.id(),
                            });
                        },
                    }
                }
                desc = self.compiler.type_description(base)?;
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
            let mut ty = resource_type.descriptor_type();
            if ty.is_none() &&
                let TypeInner::Image(img) = &desc.inner
            {
                if resource_type == ShaderResourceType::CombinedImageSampler {
                    if img.dimension == spirv::Dim::DimBuffer {
                        ty = Some(DescriptorType::UniformTexelBuffer);
                    }
                } else if resource_type == ShaderResourceType::Image {
                    ty =
                        if img.dimension == spirv::Dim::DimBuffer {
                            Some(DescriptorType::UniformTexelBuffer)
                        } else {
                            Some(DescriptorType::SampledImage)
                        };
                } else if resource_type == ShaderResourceType::StorageImage {
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
                name: self.compiler.name(resource.id)?
                    .map(|name| name.as_ref().into())
                    .unwrap_or(Default::default()),
                count,
                count_specialization,
                struct_size,
            });
        }
        Ok(uniforms)
    }
}
