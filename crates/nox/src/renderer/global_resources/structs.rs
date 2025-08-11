use crate::renderer::image::ImageRangeInfo;

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

#[derive(Clone)]
pub(super) struct ShaderResource {
    pub descriptor_set: vk::DescriptorSet,
    pub layout_id: PipelineLayoutID,
    pub set: u32,
    pub binding_count: u32,
    pub image_views: GlobalVec<(ImageID, SlotIndex<vk::ImageView>)>,
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
    pub image_source: (ImageID, Option<ImageRangeInfo>),
}

#[derive(Clone, Copy)]
pub struct ShaderResourceImageUpdate<'a> {
    pub resource: ShaderResourceID,
    pub binding: u32,
    pub starting_index: u32,
    pub infos: &'a [ShaderResourceImageInfo],
}

#[derive(Default, Clone, Copy)]
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
pub(crate) struct GraphicsPipeline {
    pub device: Arc<ash::Device>,
    pub _color_formats: GlobalVec<vk::Format>,
    pub _dynamic_states: GlobalVec<vk::DynamicState>,
    pub handle: vk::Pipeline,
    pub layout_id: PipelineLayoutID,
    pub _depth_format: vk::Format,
    pub _stencil_format: vk::Format,
    pub samples: MSAA,
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

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ImageID(pub(super) SlotIndex<Arc<Image>>);

#[derive(Clone, Copy)]
pub(super) struct Sampler {
    pub handle: vk::Sampler,
    pub _builder: SamplerBuilder,
}

#[derive(Default, Clone, Copy)]
pub struct SamplerID(pub(super) SlotIndex<Sampler>);
