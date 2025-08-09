use super::*;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ShaderID(pub(super) SlotIndex<Shader>);

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineLayoutID(pub(super) SlotIndex<pipeline::PipelineLayout>);

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct BufferID(pub(super) SlotIndex<Buffer>);

#[derive(Default, Clone, Copy)]
pub struct ShaderResourceInfo {
    pub layout_id: PipelineLayoutID,
    pub set: u32,
}

#[derive(Clone, Copy)]
pub(super) struct ShaderResource {
    pub descriptor_set: vk::DescriptorSet,
    pub layout_id: PipelineLayoutID,
    pub set: u32,
    pub binding_count: u32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct ShaderResourceID(pub(super) SlotIndex<ShaderResource>);

#[derive(Default, Clone, Copy)]
pub struct ShaderResourceBufferInfo {
    pub buffer: BufferID,
    pub offset: u64,
    pub size: u64,
}

#[derive(Clone, Copy)]
pub struct ShaderResourceImageInfo {
    pub sampler: SamplerID,
    pub image_source: ImageSourceID,
}

#[derive(Clone, Copy)]
pub struct ShaderResourceImageUpdate<'a> {
    pub resource: ShaderResourceID,
    pub binding: u32,
    pub starting_index: u32,
    pub infos: &'a [ShaderResourceImageInfo],
}

#[derive(Clone, Copy)]
pub struct ShaderResourceBufferUpdate<'a> {
    pub resource: ShaderResourceID,
    pub binding: u32,
    pub starting_index: u32,
    pub infos: &'a [ShaderResourceBufferInfo],
}

#[derive(Clone, Copy)]
pub struct ShaderResourceCopy {
    pub src_resource: ShaderResourceID,
    pub src_binding: u32,
    pub src_starting_index: u32,
    pub dst_resource: ShaderResourceID,
    pub dst_starting_index: u32,
    pub dst_binding: u32,
    pub array_count: u32,
}

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

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct GraphicsPipelineID(pub(super) SlotIndex<GraphicsPipeline>);

pub(crate) struct ImageResource {
    pub subresources: GlobalSlotMap<ImageSubresourceRange>,
    pub image: Arc<Image>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ImageID(pub(super) SlotIndex<ImageResource>);

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ImageSubresourceID(pub(super) SlotIndex<ImageResource>, pub(super) SlotIndex<ImageSubresourceRange>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImageSourceID {
    ImageID(ImageID),
    SubresourceID(ImageSubresourceID),
}

impl Default for ImageSourceID {

    fn default() -> Self {
        Self::ImageID(Default::default())
    }
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

#[derive(Clone, Copy)]
pub(super) struct Sampler {
    pub handle: vk::Sampler,
    pub _builder: SamplerBuilder,
}

#[derive(Default, Clone, Copy)]
pub struct SamplerID(pub(super) SlotIndex<Sampler>);
