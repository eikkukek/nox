mod default_binder;
mod enums;
mod structs;
mod descriptor_pool;

use std::sync::{Arc, RwLock};

use ash::vk;

use nox_mem::{
    slot_map::{GlobalSlotMap, SlotIndex},
    vec_types::{Vector, FixedVec},
    Allocator,
};

use crate::renderer::{image::SamplerBuilder, pipeline::PipelineLayout, *};

use super::{
    Error,
    PhysicalDeviceInfo,
    image::{
        ImageBuilder,
        Image,
        ImageRangeInfo,
        ImageSubresourceRange,
        ImageSource,
        ImageSourceMut,
    },
    buffer::{Buffer, BufferProperties},
    memory_binder::MemoryBinder,
};

pub use default_binder::*;
pub use enums::*;
pub use structs::*;
use descriptor_pool::*;

pub struct GlobalResources {
    device: Arc<ash::Device>,
    graphics_pipelines: GlobalSlotMap<GraphicsPipeline>,
    shaders: GlobalSlotMap<Shader>,
    pipeline_layouts: GlobalSlotMap<pipeline::PipelineLayout>,
    shader_resources: GlobalSlotMap<ShaderResource>,
    images: GlobalSlotMap<ImageResource>,
    buffers: GlobalSlotMap<Buffer>,
    samplers: GlobalSlotMap<Sampler>,
    default_binder: DefaultBinder,
    default_binder_mappable: DefaultBinder,
    descriptor_pool: DescriptorPool,
    dummy_descriptor_set_layout: vk::DescriptorSetLayout,
    dummy_descriptor_pool: vk::DescriptorPool,
    dummy_descriptor_set: vk::DescriptorSet,
    api_version: Version,
}

impl GlobalResources {

    #[inline(always)]
    pub(crate) fn new(
        device: Arc<ash::Device>,
        physical_device_info: &PhysicalDeviceInfo,
        memory_layout: MemoryLayout,
    ) -> Result<Self, Error>
    {
        let default_binder = DefaultBinder::new(
            device.clone(),
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryPropertyFlags::from_raw(0),
            physical_device_info,
        );
        let default_binder_mappable = DefaultBinder::new(
            device.clone(),
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::MemoryPropertyFlags::from_raw(0),
            physical_device_info,
        );
        let dummy_layout_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            ..Default::default()
        };
        let dummy_layout = unsafe { RaiiHandle::new(
            device.create_descriptor_set_layout(&dummy_layout_info, None)?,
            |v| device.destroy_descriptor_set_layout(v, None))
        };
        let dummy_pool_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            max_sets: 1,
            ..Default::default()
        };
        let dummy_descriptor_pool = unsafe { RaiiHandle::new(
            device.create_descriptor_pool(&dummy_pool_info, None)?,
            |v| device.destroy_descriptor_pool(v, None))
        };
        let dummy_alloc_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: *dummy_descriptor_pool,
            descriptor_set_count: 1,
            p_set_layouts: &*dummy_layout,
            ..Default::default()
        };
        let mut dummy_descriptor_set = Default::default();
        let res = unsafe {
            (device.fp_v1_0().allocate_descriptor_sets)(device.handle(), &dummy_alloc_info, &mut dummy_descriptor_set)
        };
        if res != vk::Result::SUCCESS {
            return Err(res.into())
        }
        Ok(Self {
            dummy_descriptor_set_layout: dummy_layout.into_inner(),
            dummy_descriptor_pool: dummy_descriptor_pool.into_inner(),
            descriptor_pool: DescriptorPool::new(device.clone(), memory_layout)?,
            device,
            graphics_pipelines: GlobalSlotMap::with_capacity(16).unwrap(),
            shaders: GlobalSlotMap::with_capacity(16).unwrap(),
            pipeline_layouts: GlobalSlotMap::with_capacity(16).unwrap(),
            shader_resources: GlobalSlotMap::with_capacity(16).unwrap(),
            images: GlobalSlotMap::with_capacity(16).unwrap(),
            buffers: GlobalSlotMap::with_capacity(16).unwrap(),
            samplers: GlobalSlotMap::with_capacity(4).unwrap(),
            default_binder,
            default_binder_mappable,
            dummy_descriptor_set,
            api_version: physical_device_info.api_version(),
        })
    }

    #[inline(always)]
    pub fn default_binder(&self) -> DefaultBinder {
        self.default_binder.clone()
    }

    #[inline(always)]
    pub fn default_binder_mappable(&self) -> DefaultBinder {
        self.default_binder_mappable.clone()
    }

    #[inline(always)]
    pub fn create_shader(
        &mut self,
        input: &str,
        name: &str,
        stage: ShaderStage,
    ) -> Result<ShaderID, Error>
    {
        let shader = Shader::new(
            self.device.clone(),
            input,
            name,
            stage,
            self.api_version,
        )?;
        Ok(ShaderID(self.shaders.insert(shader)))
    }

    #[inline(always)]
    pub(crate) fn get_shader(&self, id: ShaderID) -> &Shader {
        self.shaders.get(id.0)
    }

    #[inline(always)]
    pub fn create_pipeline_layout<const SHADER_COUNT: usize>(
        &mut self,
        shaders: [ShaderID; SHADER_COUNT],
    ) -> Result<PipelineLayoutID, Error>
    {
        let mut s = ArrayVec::<&Shader, SHADER_COUNT>::new();
        for id in shaders {
            s.push(self.shaders.get(id.0)).unwrap();
        }
        let pipeline_layout = PipelineLayout::new(
            self.device.clone(),
            shaders,
            &self,
        )?;
        let index = self.pipeline_layouts.insert(pipeline_layout);
        Ok(PipelineLayoutID(index))
    }

    #[inline(always)]
    pub(crate) fn get_pipeline_layout(&self, id: PipelineLayoutID) -> &PipelineLayout {
        self.pipeline_layouts.get(id.0)
    }

    #[inline(always)]
    pub fn allocate_shader_resources<F>(
        &mut self,
        resources: &[ShaderResourceInfo],
        mut collect: F,
        alloc: &impl Allocator,
    ) -> Result<(), Error>
        where
            F: FnMut(usize, ShaderResourceID)
    {
        let mut set_layouts = FixedVec::with_capacity(resources.len(), alloc)?;
        for resource in resources {
            let layout = self.pipeline_layouts.get(resource.layout_id.0);
            let set = layout.pipeline_descriptor_sets()[resource.set as usize].1;
            set_layouts.push(set).unwrap();
        }
        let sets = self.descriptor_pool.allocate(&set_layouts, alloc)?;
        for (i, set) in sets.iter().enumerate() {
            let info = resources[i];
            let index = self.shader_resources.insert(ShaderResource {
                descriptor_set: *set,
                layout_id: info.layout_id,
                set: info.set,
            });
            collect(i, ShaderResourceID(index))
        }
        Ok(())
    }

    #[inline(always)]
    pub fn free_shader_resources(
        &mut self,
        resources: &[ShaderResourceID],
        alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        let mut sets = FixedVec::with_capacity(resources.len(), alloc)?;
        for id in resources {
            let resource = self.shader_resources.get(id.0);
            sets.push(resource.descriptor_set).unwrap();
        }
        self.descriptor_pool.free(&sets)?;
        Ok(())
    }

    #[inline(always)]
    pub fn update_shader_resource(
        &mut self,
        writes: &[ShaderResourceUpdate],
        alloc: &impl Allocator,
    )
    {
    }

    #[inline(always)]
    pub fn create_graphics_pipelines<F>(
        &mut self,
        infos: &[pipeline::GraphicsPipelineInfo],
        mut collect: F,
        alloc: &impl Allocator,
    ) -> Result<(), Error>
        where
            F: FnMut(usize, GraphicsPipelineID)
    {
        let pipeline_count = infos.len();
        if pipeline_count == 0 {
            return Ok(())
        }
        let mut create_infos = FixedVec::with_capacity(pipeline_count, alloc)?;
        for info in infos {
            create_infos.push(info.as_create_info(&self, alloc)?).unwrap();
        }
        let mut vk_infos = FixedVec::with_capacity(pipeline_count, alloc)?;
        for info in &create_infos {
            const VIEWPORT_STATE: vk::PipelineViewportStateCreateInfo = vk::PipelineViewportStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
                p_next: core::ptr::null(),
                flags: vk::PipelineViewportStateCreateFlags::empty(),
                viewport_count: 1,
                p_viewports: core::ptr::null(),
                scissor_count: 1,
                p_scissors: core::ptr::null(),
                _marker: core::marker::PhantomData,
            };
            vk_infos.push(vk::GraphicsPipelineCreateInfo {
                s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
                p_next: &info.rendering_info as * const _ as *const _,
                stage_count: info.shader_stage_infos.len() as u32,
                p_stages: info.shader_stage_infos.as_ptr(),
                p_vertex_input_state: &info.vertex_input_state,
                p_input_assembly_state: &info.input_assembly_state,
                p_tessellation_state: &info.tesellation_state,
                p_viewport_state: &VIEWPORT_STATE,
                p_rasterization_state: &info.rasterization_state,
                p_multisample_state: &info.multisample_state,
                p_depth_stencil_state: &info.depth_stencil_state,
                p_color_blend_state: &info.color_blend_state,
                p_dynamic_state: &info.dynamic_state,
                layout: info.layout,
                ..Default::default()
            })?;
        }
        let mut pipelines = FixedVec::with_capacity(create_infos.len(), alloc)?;
        unsafe {
            let device = &self.device;
            let result = (device.fp_v1_0().create_graphics_pipelines)(
                device.handle(),
                vk::PipelineCache::null(),
                vk_infos.len() as u32,
                vk_infos.as_ptr(),
                core::ptr::null(),
                pipelines.as_mut_ptr(),
            );
            if result != vk::Result::SUCCESS {
                return Err(result.into())
            }
            pipelines.set_len(create_infos.len());
        }
        for (i, handle) in pipelines.iter().enumerate() {
            let info = &infos[i];
            let index = self.graphics_pipelines.insert(GraphicsPipeline {
                device: self.device.clone(),
                _color_formats: info.color_output_formats.clone(),
                _dynamic_states: info.dynamic_states.clone(),
                handle: *handle,
                layout_id: info.layout_id,
                _depth_format: info.depth_output_format,
                _stencil_format: info.stencil_output_format,
            });
            collect(i, GraphicsPipelineID(index));
        }
        Ok(())
    }

    pub(crate) fn pipeline_get_shader_resource<'a, F, Alloc>(
        &self,
        id: GraphicsPipelineID,
        mut f: F,
        alloc: &'a Alloc,
    ) -> Result<(vk::PipelineLayout, FixedVec<'a, vk::DescriptorSet, Alloc>), Error>
        where
            F: FnMut(u32) -> ShaderResourceID,
            Alloc: Allocator,
    {
        let pipeline = self.graphics_pipelines.get(id.0);
        let layout = self.pipeline_layouts.get(pipeline.layout_id.0);
        let sets = layout.pipeline_descriptor_sets();
        let mut res = FixedVec::with_capacity(sets.len(), alloc)?;
        for (i, set) in sets.iter().enumerate() {
            if set.0 {
                let resource = self.shader_resources.get(f(i as u32).0);
                res.push(resource.descriptor_set).unwrap();
            }
            else {
                res.push(self.dummy_descriptor_set).unwrap();
            }
        }
        Ok((layout.handle(), res))
    }

    pub(crate) fn pipeline_get_push_constants<'a, 'b, F, Alloc>(
        &self,
        id: GraphicsPipelineID,
        mut f: F,
        alloc: &'a Alloc,
    ) -> Result<(vk::PipelineLayout, FixedVec<'a, (vk::PushConstantRange, &'b [u8]), Alloc>), Error>
        where
            F: FnMut(PushConstant) -> &'b [u8],
            Alloc: Allocator,
    {
        let pipeline = self.graphics_pipelines.get(id.0);
        let layout = self.pipeline_layouts.get(pipeline.layout_id.0);
        let push_constants = layout.push_constant_ranges();
        let mut res = FixedVec::with_capacity(push_constants.len(), alloc)?;
        for pc in push_constants.iter().map(|v| *v) {
            let bytes = f(pc.into());
            res.push((pc, bytes)).unwrap();
        }
        Ok((layout.handle(), res))
    }

    #[inline(always)]
    pub(crate) fn get_pipeline_handle(&self, id: GraphicsPipelineID) -> vk::Pipeline {
        self.graphics_pipelines.get(id.0).handle
    }

    #[inline(always)]
    pub fn create_vertex_buffer<Binder: MemoryBinder>(
        &mut self,
        size: u64,
        binder: &mut Binder,
    ) -> Result<BufferID, Error>
    {
        let properties = BufferProperties {
            size,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            create_flags: Default::default(),
        };
        let mut buffer = Buffer::new(self.device.clone(), properties)?;
        unsafe {
            buffer.set_memory(Box::new(binder.bind_buffer_memory(buffer.handle())?));
        }
        Ok(BufferID(
            self.buffers.insert(buffer)
        ))
    }

    #[inline(always)]
    pub(crate) fn get_buffer(&self, id: BufferID) -> &Buffer {
        self.buffers.get(id.0)
    }

    #[inline(always)]
    pub(crate) fn get_mut_buffer(&mut self, id: BufferID) -> &mut Buffer {
        self.buffers.get_mut(id.0)
    }

    #[inline(always)]
    pub fn create_sampler<F: FnMut(&mut SamplerBuilder)>(
        &mut self,
        mut f: F,
    ) -> Result<SamplerID, Error>
    {
        let mut builder = SamplerBuilder::new();
        f(&mut builder);
        let handle = builder.build(&self.device)?;
        let index = self.samplers.insert(Sampler { handle, builder, });
        Ok(SamplerID(index))
    }

    #[inline(always)]
    pub fn create_image<F, Binder: MemoryBinder>(
        &mut self,
        mut f: F,
        binder: &mut Binder,
    ) -> Result<ImageID, Error>
        where
            F: FnMut(&mut ImageBuilder)
    {
        let mut builder = ImageBuilder::new(self.device.clone());
        f(&mut builder);
        let mut image = builder.build()?;
        unsafe {
            image.set_memory(Box::new(binder.bind_image_memory(image.handle())?));
        }
        Ok(ImageID(
            self.images.insert(ImageResource {
                image,
                subresources: Default::default(),
            })
        ))
    }

    #[inline(always)]
    pub fn create_image_subresource(
        s: Arc<RwLock<Self>>,
        id: ImageID,
        range_info: ImageRangeInfo
    ) -> Result<ImageSubresourceID, Error>
    {
        let subresource = ImageSubresourceRange::new(range_info, id, s.clone())?;
        let mut s = s.write().unwrap();
        Ok(ImageSubresourceID(id.0, s.images
            .get_mut(id.0)
            .subresources.insert(subresource)
        ))
    }

    #[inline(always)]
    pub fn destroy_image(&mut self, id: ImageID) {
        self.images.remove(id.0);
    }

    #[inline(always)]
    pub fn destroy_image_subresource(&mut self, id: ImageSubresourceID) {
        self.images.get_mut(id.0).subresources.remove(id.1);
    }

    #[inline(always)]
    pub fn destroy_image_source(&mut self, id: ImageSourceID) {
        match id {
            ImageSourceID::ImageID(id) => {
                self.destroy_image(id);
            },
            ImageSourceID::SubresourceID(id) => {
                self.destroy_image_subresource(id);
            },
        }
    }

    #[inline(always)]
    pub fn is_valid_image_id(&self, id: ImageSourceID) -> bool {
        match id {
            ImageSourceID::ImageID(id) => {
                self.images.contains(id.0)
            },
            ImageSourceID::SubresourceID(id) => {
                self.images
                    .try_get(id.0)
                    .map(|v| v.subresources.contains(id.1))
                    .unwrap_or(false)
            },
        }
    }

    #[inline(always)]
    pub(crate) fn get_image(
        &self,
        id: ImageID,
    ) -> &Image
    {
        &self.images.get(id.0).image
    }

    #[inline(always)]
    pub(crate) fn get_mut_image(
        &mut self,
        id: ImageID,
    ) -> &mut Image
    {
        &mut self.images.get_mut(id.0).image
    }

    #[inline(always)]
    pub(crate) fn _get_image_subresource(
        &self,
        id: ImageSubresourceID,
    ) -> &ImageSubresourceRange
    {
        self.images
            .get(id.0)
            .subresources
            .get(id.1)
    }

    #[inline(always)]
    pub(crate) fn get_mut_image_subresource(
        &mut self,
        id: ImageSubresourceID,
    ) -> &mut ImageSubresourceRange
    {
        self.images
            .get_mut(id.0)
            .subresources
            .get_mut(id.1)
    }

    #[inline(always)]
    pub(crate) fn get_image_source(
        &self,
        id: ImageSourceID,
    ) -> ImageSource<'_>
    {
        match id {
            ImageSourceID::ImageID(id) => {
                ImageSource::Image(
                    &self.images.get(id.0).image
                )
            },
            ImageSourceID::SubresourceID(id) => {
                ImageSource::Subresource(
                    self.images
                        .get(id.0)
                        .subresources
                        .get(id.1)
                )
            }
        }
    }

    #[inline(always)]
    pub(crate) fn get_mut_image_source(
        &mut self,
        id: ImageSourceID,
    ) -> ImageSourceMut<'_>
    {
        match id {
            ImageSourceID::ImageID(id) => {
                ImageSourceMut::Image(
                    &mut self.images.get_mut(id.0).image
                )
            },
            ImageSourceID::SubresourceID(id) => {
                ImageSourceMut::Subresource(self.images
                    .get_mut(id.0)
                    .subresources
                    .get_mut(id.1)
                )
            }
        }
    }

/*
    #[inline(always)]
    pub(crate) fn create_graphics_pipelines(
        infos: &[GraphicsPipelineInfo],
    ) -> Result<GlobalVec<PipelineID>, Error>
    {
        let mut create_infos = GlobalVec::with_capacity(infos.len())?;
        for info in infos {
            create_infos.push(info.as_create_info()).unwrap();
        }
        let mut vk_infos = GlobalVec::with_capacity(infos.len())?;
        for info in &create_infos {
        }
        todo!()
    }
    */
}

impl Drop for GlobalResources {

    fn drop(&mut self) {
        unsafe {
            self.device.destroy_descriptor_set_layout(self.dummy_descriptor_set_layout, None);
            self.device.destroy_descriptor_pool(self.dummy_descriptor_pool, None);
        }
    }
}
