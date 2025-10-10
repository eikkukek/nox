pub mod default_binder;
mod enums;
mod structs;
mod descriptor_pool;

use std::{ptr::NonNull, sync::Arc};

use ash::vk;

use nox_mem::{
    slot_map::{GlobalSlotMap, SlotIndex, SlotMapError},
    vec_types::{FixedVec, Vector},
    Allocator, AsRaw,
};

use crate::{
    has_bits, renderer::{
        buffer::BufferUsage, image::{Format, SamplerBuilder}, pipeline::PipelineLayout, *
    }
};

use super::{
    Error,
    PhysicalDeviceInfo,
    image::{
        ImageBuilder,
        Image,
    },
    buffer::{Buffer, BufferProperties},
    memory_binder::MemoryBinder,
};

pub use default_binder::{DefaultBinder};
pub use enums::*;
pub use structs::*;
use descriptor_pool::*;

pub struct GlobalResources {
    device: Arc<ash::Device>,
    instance: Arc<ash::Instance>,
    physical_device: vk::PhysicalDevice,
    shaders: GlobalSlotMap<Shader>,
    pipeline_layouts: GlobalSlotMap<pipeline::PipelineLayout>,
    pipeline_caches: GlobalSlotMap<PipelineCache>,
    graphics_pipelines: GlobalSlotMap<GraphicsPipeline>,
    compute_pipelines: GlobalSlotMap<ComputePipeline>,
    shader_resources: GlobalSlotMap<ShaderResource>,
    images: GlobalSlotMap<Arc<Image>>,
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
        instance: Arc<ash::Instance>,
        physical_device: vk::PhysicalDevice,
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
            physical_device,
            instance,
            shaders: GlobalSlotMap::with_capacity(16),
            pipeline_layouts: GlobalSlotMap::with_capacity(16),
            pipeline_caches: GlobalSlotMap::with_capacity(4),
            graphics_pipelines: GlobalSlotMap::with_capacity(16),
            compute_pipelines: GlobalSlotMap::with_capacity(16),
            shader_resources: GlobalSlotMap::with_capacity(16),
            images: GlobalSlotMap::with_capacity(16),
            buffers: GlobalSlotMap::with_capacity(16),
            samplers: GlobalSlotMap::with_capacity(4),
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
    pub fn supported_image_format<F: Format, U: Into<u32> + Copy>(
        &self,
        formats: &[F],
        required_features: U,
    ) -> Option<F>
    {
        for format in formats {
            let properties = unsafe {
                self.instance
                    .get_physical_device_format_properties(
                        self.physical_device, format.as_vk_format())
            };
            if has_bits!(
                    properties.optimal_tiling_features,
                    vk::FormatFeatureFlags::from_raw(<U as Into<u32>>::into(required_features))
            ) {
                return Some(*format)
            }
        } 
        None
    }

    #[inline(always)]
    pub fn create_shader(
        &mut self,
        input: &str,
        name: &str,
        stage: ShaderStage,
    ) -> Result<ShaderId, Error>
    {
        let spriv = shader_fn::glsl_to_spirv(
            &input,
            name,
            stage.into(),
            self.api_version,
        )?;
        let shader = Shader::new(
            self.device.clone(),
            spriv.as_binary(),
            stage,
        )?;
        Ok(ShaderId(self.shaders.insert(shader)))
    }

    #[inline(always)]
    pub fn add_shader(
        &mut self,
        spirv: &[u32],
        stage: ShaderStage,
    ) -> Result<ShaderId, Error>
    {
        if spirv.len() % 4 != 0 {
            return Err(Error::ShaderError(format!("spirv binary size must be a multiple of 4")))
        }
        let shader = Shader::new(
            self.device.clone(),
            spirv,
            stage,
        )?;
        Ok(ShaderId(self.shaders.insert(shader)))
    }

    #[inline(always)]
    pub fn destroy_shader(&mut self,shader: ShaderId) {
        self.shaders.remove(shader.0).ok();
    }

    #[inline(always)]
    pub(crate) fn get_shader(&self, id: ShaderId) -> Result<&Shader, SlotMapError> {
        self.shaders.get(id.0)
    }

    #[inline(always)]
    pub fn create_pipeline_layout<const SHADER_COUNT: usize>(
        &mut self,
        shaders: [ShaderId; SHADER_COUNT],
    ) -> Result<PipelineLayoutId, Error>
    {
        let mut s = ArrayVec::<&Shader, SHADER_COUNT>::new();
        for id in shaders {
            s.push(self.shaders.get(id.0)?).unwrap();
        }
        let pipeline_layout = PipelineLayout::new(
            self.device.clone(),
            shaders,
            &self,
        )?;
        let index = self.pipeline_layouts.insert(pipeline_layout);
        Ok(PipelineLayoutId(index))
    }

    #[inline(always)]
    pub fn destroy_pipeline_layout(&mut self, layout_id: PipelineLayoutId) {
        self.pipeline_layouts.remove(layout_id.0).ok();
    }

    #[inline(always)]
    pub(crate) fn get_pipeline_layout(&self, id: PipelineLayoutId) -> Result<&PipelineLayout, SlotMapError> {
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
            F: FnMut(usize, ShaderResourceId)
    {
        let mut set_layouts = FixedVec::with_capacity(resources.len(), alloc)?;
        for resource in resources {
            let layout = self.pipeline_layouts.get(resource.layout_id.0)?;
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
                binding_count: self.pipeline_layouts
                    .get(info.layout_id.0)
                    .unwrap()
                    .pipeline_descriptor_sets()[info.set as usize]
                    .0.len() as u32,
                image_views: Default::default(),
            });
            collect(i, ShaderResourceId(index))
        }
        Ok(())
    }

    #[inline(always)]
    pub fn free_shader_resources(
        &mut self,
        resources: &[ShaderResourceId],
        alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        let mut sets = FixedVec::with_capacity(resources.len(), alloc)?;
        for id in resources {
            let resource = self.shader_resources.get(id.0)?;
            for (image, index) in &resource.image_views {
                if let Ok(image) = self.get_image(*image) {
                    image.destroy_subview(*index).unwrap();
                }
            }
            sets.push(resource.descriptor_set).unwrap();
            self.shader_resources.remove(id.0).unwrap();
        }
        self.descriptor_pool.free(&sets)?;
        Ok(())
    }

    #[inline(always)]
    pub fn get_descriptor_set(
        &mut self,
        resource_id: ShaderResourceId,
    ) -> Result<vk::DescriptorSet, SlotMapError>
    {
        self.shader_resources.get(resource_id.0).map(|v| v.descriptor_set)
    }

    #[inline(always)]
    pub fn update_shader_resources(
        &mut self,
        image_updates: &[ShaderResourceImageUpdate],
        buffer_updates: &[ShaderResourceBufferUpdate],
        copies: &[ShaderResourceCopy],
        alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        let mut writes = FixedVec::with_capacity(image_updates.len() + buffer_updates.len(), alloc)?;
        let mut image_infos = FixedVec::with_capacity(image_updates.len(), alloc)?;
        for update in image_updates {
            let set = self.shader_resources.get_mut(update.resource.0)?;
            for (image, index) in &set.image_views {
                if let Ok(image) = self.images.get(image.0) {
                    image.destroy_subview(*index).unwrap();
                }
            }
            set.image_views.resize(0, Default::default());
            let vk_set = set.descriptor_set;
            let Some(ty) = self.pipeline_layouts
                .get(set.layout_id.0)?
                .pipeline_descriptor_sets()
                [set.set as usize].0
                [update.binding as usize]
            else {
                return Err(Error::ShaderError(
                    format!("invalid shader resource image binding {} for set {}", update.binding, set.set)
                ));
            };
            let mut vk_infos = FixedVec::with_capacity(update.infos.len(), alloc)?;
            for info in update.infos {
                let (id, range_info) = info.image_source;
                let sampler = self.samplers.get(info.sampler.0)?.handle;
                let image_view =
                    if let Some(range_info) = range_info {
                        let (index, view) = self.get_image(id)?.create_subview(range_info)?;
                        let set = self.shader_resources.get_mut(update.resource.0)?;
                        set.image_views.push((id, index));
                        view
                    }
                    else {
                        self.get_image(id)?.get_view()?
                    };
                let vk_info = vk::DescriptorImageInfo {
                    sampler,
                    image_view,
                    image_layout:
                        if info.storage_image {
                            vk::ImageLayout::GENERAL
                        } else {
                            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
                        },
                };
                vk_infos.push(vk_info).unwrap();
            }
            let vk_infos = image_infos.push(vk_infos).unwrap();
            let write = vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                dst_set: vk_set,
                dst_binding: update.binding,
                dst_array_element: update.starting_index,
                descriptor_count: vk_infos.len() as u32,
                descriptor_type: ty,
                p_image_info: vk_infos.as_ptr(),
                ..Default::default()
            };
            writes.push(write).unwrap();
        }
        let mut buffer_infos = FixedVec::with_capacity(buffer_updates.len(), alloc)?;
        for update in buffer_updates {
            let set = self.shader_resources.get(update.resource.0)?;
            let vk_set = set.descriptor_set;
            let Some(ty) = self.pipeline_layouts
                .get(set.layout_id.0)?
                .pipeline_descriptor_sets()
                [set.set as usize].0
                [update.binding as usize]
            else {
                return Err(Error::ShaderError(
                    format!("invalid shader resource image binding {} for set {}", update.binding, set.set)
                ));
            };
            let mut vk_infos = FixedVec::with_capacity(update.infos.len(), alloc)?;
            for info in update.infos {
                let buffer = self.buffers.get(info.buffer.0)?;
                let properties = buffer.properties();
                if info.offset + info.size > properties.size {
                    return Err(Error::BufferError(BufferError::OutOfRange {
                        buffer_size: properties.size,
                        requested_offset: info.offset,
                        requested_size: info.size,
                    }))
                }
                let vk_info = vk::DescriptorBufferInfo {
                    buffer: buffer.handle(),
                    offset: info.offset,
                    range: info.size,
                };
                vk_infos.push(vk_info).unwrap();
            }
            let vk_infos = buffer_infos.push(vk_infos).unwrap();
            let write = vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                dst_set: vk_set,
                dst_binding: update.binding,
                dst_array_element: update.starting_index,
                descriptor_count: vk_infos.len() as u32,
                descriptor_type: ty,
                p_buffer_info: vk_infos.as_ptr(),
                ..Default::default()
            };
            writes.push(write).unwrap();
        }
        let mut vk_copies = FixedVec::with_capacity(copies.len(), alloc)?;
        for copy in copies {
            let src = self.shader_resources.get(copy.src_resource.0)?;
            let dst = self.shader_resources.get(copy.dst_resource.0)?;
            assert!(src.binding_count > copy.src_binding,
                "copy src binding {} out of range with count {}", src.binding_count, copy.src_binding);
            assert!(dst.binding_count > copy.dst_binding,
                "copy src binding {} out of range with count {}", dst.binding_count, copy.dst_binding);
            let vk_copy = vk::CopyDescriptorSet {
                s_type: vk::StructureType::COPY_DESCRIPTOR_SET,
                src_set: src.descriptor_set,
                src_binding: copy.src_binding,
                src_array_element: copy.src_starting_index,
                dst_set: dst.descriptor_set,
                dst_binding: copy.dst_binding,
                dst_array_element: copy.dst_starting_index,
                descriptor_count: copy.array_count,
                ..Default::default()
            };
            vk_copies.push(vk_copy).unwrap();
        }
        unsafe {
            self.device.update_descriptor_sets(&writes, &vk_copies);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn create_pipeline_cache(
        &mut self,
        initial_data: Option<&[u8]>,
    ) -> Result<PipelineCacheId, Error>
    {
        let initial_data = initial_data.unwrap_or(&[]);
        let info = vk::PipelineCacheCreateInfo {
            s_type: vk::StructureType::PIPELINE_CACHE_CREATE_INFO,
            initial_data_size: initial_data.len(),
            p_initial_data: initial_data.as_ptr() as _,
            ..Default::default()
        };
        let handle = unsafe {
            self.device.create_pipeline_cache(&info, None)?
        };
        let index =  self.pipeline_caches.insert(PipelineCache {
            device: self.device.clone(),
            handle: handle,
        });
        Ok(PipelineCacheId(index))
    }

    #[inline(always)]
    pub fn retrieve_pipeline_cache_data(
        &mut self,
        id: PipelineCacheId,
    ) -> Result<GlobalVec<u8>, Error>
    {
        let device = &*self.device;
        let handle = self.pipeline_caches.get(id.0)?.handle;
        unsafe {
            let mut cache_size = 0;
            let result = (device.fp_v1_0().get_pipeline_cache_data)(
                device.handle(),
                handle,
                &mut cache_size,
                Default::default(),
            );
            if result != vk::Result::SUCCESS {
                return Err(result.into())
            }
            let mut data = GlobalVec::with_capacity(cache_size as usize);
            let result = (device.fp_v1_0().get_pipeline_cache_data)(
                device.handle(),
                handle,
                &mut cache_size,
                data.as_mut_ptr() as _,
            );
            if result != vk::Result::SUCCESS {
                return Err(result.into())
            }
            data.set_len(cache_size as usize);
            Ok(data)
        }
    }

    #[inline(always)]
    pub fn destroy_pipeline_cache(&mut self, id: PipelineCacheId) {
        self.pipeline_caches.remove(id.0).ok();
    }

    #[inline(always)]
    pub fn create_graphics_pipelines(
        &mut self,
        infos: &[pipeline::GraphicsPipelineInfo],
        cache_id: Option<PipelineCacheId>,
        alloc: &impl Allocator,
        mut collect: impl FnMut(usize, GraphicsPipelineId),
    ) -> Result<(), Error>
    {
        let pipeline_count = infos.len();
        if pipeline_count == 0 {
            return Ok(())
        }
        let mut create_infos = FixedVec::with_capacity(pipeline_count, alloc)?;
        let mut vk_infos = FixedVec::with_capacity(pipeline_count, alloc)?;
        for info in infos {
            let info = create_infos.push(info.as_create_info(&self, alloc)?).unwrap();
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
            let device = &*self.device;
            let pipeline_cache = cache_id
                .map(|v| self.pipeline_caches.get(v.0).map(|v| v.handle))
                .unwrap_or(Ok(Default::default()))?;
            let result = (device.fp_v1_0().create_graphics_pipelines)(
                device.handle(),
                pipeline_cache,
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
                handle: *handle,
                _color_formats: info.color_output_formats.clone(),
                _dynamic_states: info.dynamic_states.clone(),
                layout_id: info.layout_id,
                samples: info.sample_shading_info.map(|v| v.samples).unwrap_or(MSAA::X1),
                _depth_format: info.depth_output_format,
                _stencil_format: info.stencil_output_format,
            });
            collect(i, GraphicsPipelineId(index));
        }
        Ok(())
    }

    pub fn destroy_graphics_pipeline(&mut self, id: GraphicsPipelineId) {
        self.graphics_pipelines.remove(id.0).ok();
    }

    pub fn create_compute_pipelines(
        &mut self,
        infos: &[pipeline::ComputePipelineInfo],
        cache_id: Option<PipelineCacheId>,
        alloc: &impl Allocator,
        mut collect: impl FnMut(usize, ComputePipelineId),
    ) -> Result<(), Error>
    {
        let pipeline_count = infos.len();
        if pipeline_count == 0 {
            return Ok(())
        }
        let mut vk_infos = FixedVec::with_capacity(pipeline_count, alloc)?;
        for info in infos {
            vk_infos.push(info.as_create_info(self)?).unwrap();
        }
        let mut pipelines = FixedVec::with_capacity(vk_infos.len(), alloc)?;
        unsafe {
            let device = &*self.device;
            let pipeline_cache = cache_id 
                .map(|v| self.pipeline_caches.get(v.0).map(|v| v.handle))
                .unwrap_or(Ok(Default::default()))?;
            let result = (self.device.fp_v1_0().create_compute_pipelines)(
                device.handle(),
                pipeline_cache,
                vk_infos.len() as u32,
                vk_infos.as_ptr(),
                core::ptr::null(),
                pipelines.as_mut_ptr(),

            );
            if result != vk::Result::SUCCESS {
                return Err(result.into())
            }
            pipelines.set_len(vk_infos.len());
        };
        for (i, handle) in pipelines.iter().enumerate() {
            let info = &infos[i];
            let index = self.compute_pipelines.insert(ComputePipeline {
                device: self.device.clone(),
                handle: *handle,
                layout_id: info.layout_id,
            });
            collect(i, ComputePipelineId(index));
        }
        Ok(())
    }

    pub fn destroy_compute_pipeline(&mut self, id: ComputePipelineId) {
        self.compute_pipelines.remove(id.0).ok();
    }

    pub(crate) fn pipeline_get_shader_resource<'a, F, Alloc>(
        &self,
        layout_id: PipelineLayoutId,
        alloc: &'a Alloc,
        mut f: F,
    ) -> Result<(vk::PipelineLayout, FixedVec<'a, vk::DescriptorSet, Alloc>), Error>
        where
            Alloc: Allocator,
            F: FnMut(u32) -> ShaderResourceId,
    {
        let layout = self.pipeline_layouts.get(layout_id.0)?;
        let sets = layout.pipeline_descriptor_sets();
        let mut res = FixedVec::with_capacity(sets.len(), alloc)?;
        for (i, set) in sets.iter().enumerate() {
            if set.0.len() != 0 {
                let resource = self.shader_resources.get(f(i as u32).0)?;
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
        layout_id: PipelineLayoutId,
        alloc: &'a Alloc,
        mut f: F,
    ) -> Result<(vk::PipelineLayout, FixedVec<'a, (PushConstant, &'b [u8]), Alloc>), Error>
        where
            Alloc: Allocator,
            F: FnMut(PushConstant) -> &'b [u8],
    {
        let layout = self.pipeline_layouts.get(layout_id.0)?;
        let push_constants = layout.push_constant_ranges();
        let mut res = FixedVec::with_capacity(push_constants.len(), alloc)?;
        for pc in push_constants.iter().map(|&pc| pc.into()) {
            res.push((pc, f(pc))).unwrap();
        }
        Ok((layout.handle(), res))
    }

    #[inline(always)]
    pub(crate) fn get_graphics_pipeline(&self, id: GraphicsPipelineId) -> Result<&GraphicsPipeline, SlotMapError> {
        self.graphics_pipelines.get(id.0)
    }

    #[inline(always)]
    pub(crate) fn get_compute_pipeline(&self, id: ComputePipelineId) -> Result<&ComputePipeline, SlotMapError> {
        self.compute_pipelines.get(id.0)
    }

    #[inline(always)]
    pub fn create_buffer<Binder: MemoryBinder>(
        &mut self,
        size: u64,
        usage: &[BufferUsage],
        binder: &mut Binder,
    ) -> Result<BufferId, Error>
    {
        if size == 0 {
            return Err(Error::ZeroSizeAlloc)
        }
        let mut vk_usage = vk::BufferUsageFlags::from_raw(0);
        for usage in usage {
            vk_usage |= vk::BufferUsageFlags::from_raw(usage.as_raw());
        }
        let properties = BufferProperties {
            size,
            usage: vk_usage,
            create_flags: Default::default(),
        };
        let mut buffer = Buffer::new(self.device.clone(), properties)?;
        unsafe {
            buffer.set_memory(Box::new(binder.bind_buffer_memory(buffer.handle())?));
        }
        Ok(BufferId(
            self.buffers.insert(buffer)
        ))
    }

    #[inline(always)]
    pub fn destroy_buffer(&mut self, id: BufferId) {
        self.buffers.remove(id.0).ok();
    }

    #[inline(always)]
    pub fn is_buffer_valid(&mut self, id: BufferId) -> bool {
        self.buffers.contains(id.0)
    }

    #[inline(always)]
    pub unsafe fn map_buffer(
        &mut self,
        buffer: BufferId,
    ) -> Option<NonNull<u8>>
    {
        unsafe {
            self.buffers.get_mut(buffer.0).ok()?.map_memory()
        }
    }

    #[inline(always)]
    pub fn buffer_size(&self, buffer: BufferId) -> Option<u64> {
        self.buffers.get(buffer.0).ok().map(|v| v.properties().size)
    }

    #[inline(always)]
    pub(crate) fn get_buffer(&self, id: BufferId) -> Result<&Buffer, SlotMapError> {
        self.buffers.get(id.0)
    }

    #[inline(always)]
    pub(crate) fn get_mut_buffer(&mut self, id: BufferId) -> Result<&mut Buffer, SlotMapError> {
        self.buffers.get_mut(id.0)
    }

    #[inline(always)]
    pub fn create_sampler<F: FnMut(&mut SamplerBuilder)>(
        &mut self,
        mut f: F,
    ) -> Result<SamplerId, Error>
    {
        let mut builder = SamplerBuilder::new();
        f(&mut builder);
        let handle = builder.build(&self.device)?;
        let index = self.samplers.insert(Sampler { device: self.device.clone(), handle, _builder: builder, });
        Ok(SamplerId(index))
    }

    #[inline(always)]
    pub fn destroy_sampler(&mut self, sampler: SamplerId) {
        self.samplers.remove(sampler.0).ok();
    }

    #[inline(always)]
    pub fn create_image<F, Binder: MemoryBinder>(
        &mut self,
        binder: &mut Binder,
        mut f: F,
    ) -> Result<ImageId, Error>
        where
            F: FnMut(&mut ImageBuilder)
    {
        let mut builder = ImageBuilder::new(self.device.clone());
        f(&mut builder);
        let mut image = builder.build()?;
        unsafe {
            image.set_memory(Box::new(binder.bind_image_memory(image.handle())?));
        }
        Ok(ImageId(
            self.images.insert(Arc::new(image))
        ))
    }

    #[inline(always)]
    pub fn destroy_image(&mut self, id: ImageId) {
        self.images.remove(id.0).ok();
    }

    #[inline(always)]
    pub fn is_valid_image(&self, id: ImageId) -> bool {
        self.images.contains(id.0)
    }

    #[inline(always)]
    pub(crate) fn get_image(
        &self,
        id: ImageId,
    ) -> Result<Arc<Image>, SlotMapError>
    {
        self.images.get(id.0).map(|v| v.clone())
    }

    pub(crate) fn clean_up(&mut self) {
        unsafe {
            self.device.destroy_descriptor_set_layout(self.dummy_descriptor_set_layout, None);
            self.device.destroy_descriptor_pool(self.dummy_descriptor_pool, None);
            self.samplers.clear();
            self.buffers.clear();
            self.images.clear();
            self.graphics_pipelines.clear();
            self.pipeline_layouts.clear();
            self.shaders.clear();
            self.descriptor_pool.clean_up();
        }
    }
}
