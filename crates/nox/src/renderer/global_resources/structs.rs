use crate::renderer::image::ImageRangeInfo;

use super::*;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ShaderId(pub(super) SlotIndex<Shader>);

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineLayoutId(pub(super) SlotIndex<pipeline::PipelineLayout>);

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct BufferId(pub(super) SlotIndex<Buffer>);

#[derive(Default, Clone, Copy)]
pub struct ShaderResourceInfo {
    pub layout_id: PipelineLayoutId,
    pub set: u32,
}

#[derive(Clone)]
pub(super) struct ShaderResource {
    pub descriptor_set: vk::DescriptorSet,
    pub layout_id: PipelineLayoutId,
    pub set: u32,
    pub binding_count: u32,
    pub image_views: GlobalVec<(ImageId, SlotIndex<vk::ImageView>)>,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct ShaderResourceId(pub(super) SlotIndex<ShaderResource>);

#[derive(Default, Clone, Copy)]
pub struct ShaderResourceBufferInfo {
    pub buffer: BufferId,
    pub offset: u64,
    pub size: u64,
}

#[derive(Clone, Copy)]
pub struct ShaderResourceImageInfo {
    pub sampler: SamplerId,
    pub image_source: (ImageId, Option<ImageRangeInfo>),
    pub storage_image: bool,
}

#[derive(Clone, Copy)]
pub struct ShaderResourceImageUpdate<'a> {
    pub resource: ShaderResourceId,
    pub binding: u32,
    pub starting_index: u32,
    pub infos: &'a [ShaderResourceImageInfo],
}

#[derive(Default, Clone, Copy)]
pub struct ShaderResourceBufferUpdate<'a> {
    pub resource: ShaderResourceId,
    pub binding: u32,
    pub starting_index: u32,
    pub infos: &'a [ShaderResourceBufferInfo],
}

#[derive(Clone, Copy)]
pub struct ShaderResourceCopy {
    pub src_resource: ShaderResourceId,
    pub src_binding: u32,
    pub src_starting_index: u32,
    pub dst_resource: ShaderResourceId,
    pub dst_starting_index: u32,
    pub dst_binding: u32,
    pub array_count: u32,
}

#[derive(Clone)]
pub(crate) struct GraphicsPipeline {
    pub device: Arc<ash::Device>,
    pub handle: vk::Pipeline,
    pub _color_formats: GlobalVec<vk::Format>,
    pub _dynamic_states: GlobalVec<vk::DynamicState>,
    pub layout_id: PipelineLayoutId,
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
pub struct GraphicsPipelineId(pub(super) SlotIndex<GraphicsPipeline>);

pub(crate) struct ComputePipeline {
    pub device: Arc<ash::Device>,
    pub handle: vk::Pipeline,
    pub layout_id: PipelineLayoutId,
}

impl Drop for ComputePipeline {

    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.handle, None);
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ComputePipelineId(pub(super) SlotIndex<ComputePipeline>);

pub(crate) struct PipelineCache {
    pub device: Arc<ash::Device>,
    pub handle: vk::PipelineCache,
}

impl Drop for PipelineCache {
    
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline_cache(self.handle, None);
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct PipelineCacheId(pub(super) SlotIndex<PipelineCache>);

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ImageId(pub(super) SlotIndex<Arc<Image>>);

#[derive(Clone)]
pub(super) struct Sampler {
    pub device: Arc<ash::Device>,
    pub handle: vk::Sampler,
    pub _builder: SamplerBuilder,
}

impl Drop for Sampler {

    fn drop(&mut self) {
        unsafe {
            self.device.destroy_sampler(self.handle, None);
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct SamplerId(pub(super) SlotIndex<Sampler>);
