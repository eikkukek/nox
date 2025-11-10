use crate::{memory_binder::DeviceMemory, renderer::image::ImageRangeInfo};

use super::*;

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ShaderId(pub(super) SlotIndex<Shader>);

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PipelineLayoutId(pub(super) SlotIndex<pipeline::PipelineLayout>);

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct BufferId(pub(super) SlotIndex<Buffer>);

#[must_use]
#[derive(Default, Clone, Copy)]
pub struct ShaderResourceInfo {
    pub layout_id: PipelineLayoutId,
    pub set: u32,
}

impl ShaderResourceInfo {
    
    #[inline(always)]
    pub fn new(layout_id: PipelineLayoutId, set: u32) -> Self {
        Self {
            layout_id,
            set,
        }
    }
}

#[derive(Clone)]
pub(super) struct ShaderResource {
    pub descriptor_set: vk::DescriptorSet,
    pub layout_id: PipelineLayoutId,
    pub set: u32,
    pub binding_count: u32,
    pub image_views: GlobalVec<(ImageId, SlotIndex<vk::ImageView>)>,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
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

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineCacheId(pub(super) SlotIndex<PipelineCache>);

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SamplerId(pub(super) SlotIndex<Sampler>);

pub(super) struct LinearDeviceAllocResource {
    pub alloc: LinearDeviceAlloc,
    pub semaphore_count: u32,
}

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LinearDeviceAllocId(pub(super) SlotIndex<Arc<RwLock<LinearDeviceAllocResource>>>);

pub struct LinearDeviceAllocLock {
    pub(super) alloc: Arc<RwLock<LinearDeviceAllocResource>>,
}

impl LinearDeviceAllocLock {

    pub unsafe fn reset(&mut self) {
        unsafe {
            self.alloc
                .write()
                .unwrap()
                .alloc
                .reset();
        }
    }
}

impl MemoryBinder for LinearDeviceAllocLock {

    fn bind_image_memory(
        &mut self,
        image: vk::Image,
        fall_back: Option<&mut dyn FnMut(vk::Image) -> Result<Box<dyn DeviceMemory>, Error>>,
    ) -> Result<Box<dyn DeviceMemory>, Error> {
        self.alloc
            .write()
            .unwrap()
            .alloc.bind_image_memory(image, fall_back)
    }

    fn bind_buffer_memory(
        &mut self,
        buffer: vk::Buffer,
        fall_back: Option<&mut dyn FnMut(vk::Buffer) -> Result<Box<dyn DeviceMemory>, Error>>,
    ) -> Result<Box<dyn DeviceMemory>, Error> {
        self.alloc
            .write()
            .unwrap()
            .alloc.bind_buffer_memory(buffer, fall_back)
    }
}

pub(super) struct TimelineSemaphore {
    pub handle: vk::Semaphore,
    pub locked_resources: GlobalVec<(u64, LinearDeviceAllocId)>,
}

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TimelineSemaphoreId(pub(super) SlotIndex<TimelineSemaphore>);

pub enum ResourceBinderImage<'a> {
    DefaultBinder,
    DefaultBinderMappable,
    LinearDeviceAlloc(LinearDeviceAllocId),
    Owned(&'a mut dyn MemoryBinder, Option<&'a mut dyn FnMut(vk::Image) -> Result<Box<dyn DeviceMemory>, Error>>)
}

pub enum ResourceBinderBuffer<'a> {
    DefaultBinder,
    DefaultBinderMappable,
    LinearDeviceAlloc(LinearDeviceAllocId),
    Owned(&'a mut dyn MemoryBinder, Option<&'a mut dyn FnMut(vk::Buffer) -> Result<Box<dyn DeviceMemory>, Error>>)
}
