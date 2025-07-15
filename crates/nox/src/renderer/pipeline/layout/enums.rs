use ash::vk;

use nox_mem::AsRaw;

#[repr(i32)]
#[derive(Clone, Copy, AsRaw)]
pub enum DescriptorType {
    Sampler = vk::DescriptorType::SAMPLER.as_raw(),
    CombinedImageSampler = vk::DescriptorType::COMBINED_IMAGE_SAMPLER.as_raw(),
    SampledImage = vk::DescriptorType::SAMPLED_IMAGE.as_raw(),
    StorageImage = vk::DescriptorType::STORAGE_IMAGE.as_raw(),
    UniformTexelBuffer = vk::DescriptorType::UNIFORM_TEXEL_BUFFER.as_raw(),
    StorageTexelBuffer = vk::DescriptorType::STORAGE_TEXEL_BUFFER.as_raw(),
    UniformBuffer = vk::DescriptorType::UNIFORM_BUFFER.as_raw(),
    StorageBuffer = vk::DescriptorType::STORAGE_BUFFER.as_raw(),
    UniformBufferDynamic = vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC.as_raw(),
    StorageBufferDynamic = vk::DescriptorType::STORAGE_BUFFER_DYNAMIC.as_raw(),
}

#[repr(u32)]
#[derive(Clone, Copy, AsRaw)]
pub enum ShaderStage {
    Vertex = vk::ShaderStageFlags::VERTEX.as_raw(),
    Geometry = vk::ShaderStageFlags::GEOMETRY.as_raw(),
    Fragment = vk::ShaderStageFlags::FRAGMENT.as_raw(),
    Compute = vk::ShaderStageFlags::COMPUTE.as_raw(),
}
