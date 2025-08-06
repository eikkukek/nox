use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ShaderID(pub(super) SlotIndex<Shader>);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineLayoutID(pub(super) SlotIndex<pipeline::PipelineLayout>);

#[derive(Clone, Copy)]
pub struct ShaderResourceInfo {
    pub layout_id: PipelineLayoutID,
    pub set: u32,
}

pub(super) struct ShaderResource {
    pub descriptor_set: vk::DescriptorSet,
    pub layout_id: PipelineLayoutID,
    pub set: u32,
}

pub enum ShaderResourceWrite {
    BufferWrite { buffer: BufferID, offset: u64, range: u64, },
    ImageWrite { sampler: SamplerID, image_source: ImageSourceID, },
}

pub struct ShaderResourceUpdate {
    pub resource: ShaderResourceID,
    pub write: ShaderResourceWrite,
}

pub struct ShaderResourceID(pub(super) SlotIndex<ShaderResource>);

#[derive(Clone)]
pub(super) struct GraphicsPipeline {
    pub device: Arc<ash::Device>,
    pub _color_formats: GlobalVec<vk::Format>,
    pub _dynamic_states: GlobalVec<vk::DynamicState>,
    pub handle: vk::Pipeline,
    pub layout_id: PipelineLayoutID,
    pub _depth_format: vk::Format,
    pub _stencil_format: vk::Format,
}

impl Drop for GraphicsPipeline {

    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.handle, None);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GraphicsPipelineID(pub(super) SlotIndex<GraphicsPipeline>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageID(pub(super) SlotIndex<ImageResource>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageSubresourceID(pub(super) SlotIndex<ImageResource>, pub(super) SlotIndex<ImageSubresourceRange>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImageSourceID {
    ImageID(ImageID),
    SubresourceID(ImageSubresourceID),
}

impl From<ImageID> for ImageSourceID {

    fn from(value: ImageID) -> Self {
        Self::ImageID(value)
    }
}

impl From<ImageSubresourceID> for ImageSourceID {

    fn from(value: ImageSubresourceID) -> Self {
        Self::SubresourceID(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BufferID(pub(super) SlotIndex<Buffer>);

pub(crate) struct ImageResource {
    pub subresources: GlobalSlotMap<ImageSubresourceRange>,
    pub image: Image,
}

#[derive(Clone, Copy)]
pub(super) struct Sampler {
    pub handle: vk::Sampler,
    pub builder: SamplerBuilder,
}

#[derive(Clone, Copy)]
pub struct SamplerID(pub(super) SlotIndex<Sampler>);
