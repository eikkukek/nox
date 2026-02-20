mod pipeline;
mod image;
pub mod memory_binder;
mod context;
mod attributes;

mod memory_layout;
mod handle;
mod shader;
mod enums;
mod structs;
mod physical_device;
mod surface;
mod vulkan;
mod swapchain;
mod buffer;
mod resources;
pub(crate) mod commands;

use compact_str::format_compact;

use ahash::AHashMap;

use nox_mem::{
    string::*,
    vec::{Vec32, FixedVec32, Vector},
    slot_map::SlotMap,
    conditional::True,
};

use nox_alloc::arena::{RwArena, Arena, ImmutArena};

use nox_threads::executor::ThreadPool;

use crate::sync::{Arc, OnceLock, SwapLock, RwLock};

use crate::dev::{
    error::{Error, Result, Context, location},
    has_bits,
    prelude::*,
};

use crate::win;

pub(crate) mod prelude {

    use super::*;

    pub use super::Gpu;
    pub use attributes::*;
    pub use context::GpuContext;
    pub use enums::*;
    pub use structs::*;
    pub use memory_layout::MemoryLayout;
    pub use handle::{Handle, RaiiHandle};
    pub use image::*;
    pub use buffer::*;
    pub use physical_device::*;
    pub use resources::*;
    pub use pipeline::*;
    pub use commands::prelude::*;
    pub use nox_proc::VertexInput;
    pub use shader::*;
    pub use pipeline::vertex_input::*;
    pub use super::memory_binder;

    pub(crate) use super::Vulkan;
    pub(crate) use surface::Surface;
    pub(crate) use swapchain::{Swapchain, FrameData};
    pub(crate) use super::commands;

    pub(crate) const COMMAND_REQUEST_IGNORED: u32 = u32::MAX;
}
pub(crate) use vulkan::Vulkan;
use swapchain::PresentResult;
use commands::scheduler::QueueScheduler;

pub use prelude::*;

use nox_ash::vk;

pub type DeviceName = ArrayString<{vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;

pub const MIN_BUFFERED_FRAMES: u32 = 2;
pub const MAX_BUFFERED_FRAMES: u32 = 8;

pub struct TmpAllocs {
    fallback_alloc: Arc<RwArena<True>>,
    tmp_allocs: AHashMap<std::thread::ThreadId, Arc<RwArena<True>>>,
}

impl TmpAllocs {

    #[inline(always)]
    pub fn tmp_alloc(
        &self
    ) -> impl Arena<True> + ImmutArena + 'static
    {
        self.tmp_allocs
            .get(&std::thread::current().id())
            .cloned()
            .unwrap_or(self.fallback_alloc.clone())
    }
}

pub struct Gpu {
    vk: Arc<Vulkan>,
    attributes: GpuAttributes,
    thread_pool: ThreadPool,
    queue_scheduler: OnceLock<QueueScheduler>,
    shader_cache: ShaderCache,
    pipeline_batches: SwapLock<Vec32<OnceLock<PipelineBatch>>>,
    shader_resource_pools: SwapLock<SlotMap<ShaderResourcePool>>,
    buffers: RwLock<SlotMap<BufferMeta>>,
    images: RwLock<SlotMap<ImageMeta>>,
    memory_binders: SwapLock<SlotMap<MemoryBinderResource>>,
    timeline_semaphores: RwLock<SlotMap<vk::Semaphore>>,
    default_binder: DefaultBinder,
    default_binder_mappable: DefaultBinder,
    tmp_allocs: Arc<TmpAllocs>,
    buffered_frames: u32,
    current_frame_index: u32,
}

impl Gpu {

    #[inline(always)]
    pub(crate) fn new(
        event_loop: &event_loop::ActiveEventLoop,
        attributes: GpuAttributes,
    ) -> Result<Arc<Self>>
    {
        let vk = Arc::new(Vulkan
            ::new(
                event_loop.winit(),
                &attributes,
            ).context("failed to create vulkan backend")?);
        let default_binder = DefaultBinder::new(
            vk.clone(),
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryPropertyFlags::from_raw(0),
        );
        let default_binder_mappable = DefaultBinder::new(
            vk.clone(),
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::MemoryPropertyFlags::from_raw(0),
        );
        let main_tmp_alloc = RwArena
            ::with_fallback(attributes.memory_layout.tmp_arena_size())
            .context("failed to create arena alloc")?;
        let mut tmp_allocs = AHashMap::default();
        tmp_allocs.insert(
            std::thread::current().id(),
            Arc::new(RwArena::with_fallback(attributes.memory_layout.tmp_arena_size())
                .context("failed to create arena alloc")?)
        );
        let thread_pool = event_loop.thread_pool();
        for id in thread_pool.worker_threads() {
            tmp_allocs
                .entry(id)
                .or_try_insert_with(|| Ok(Arc::new(RwArena::with_fallback(attributes.memory_layout.tmp_arena_size())
                    .context("failed to create arena alloc")?
                )))?;
        }
        let command_workers = attributes.command_workers;
        let s = Arc::new(Self {
            thread_pool,
            attributes,
            queue_scheduler: OnceLock::new(),
            shader_cache: ShaderCache::new(vk.clone()),
            vk,
            pipeline_batches: SwapLock::default(),
            shader_resource_pools: SwapLock::new(SlotMap::new()),
            images: RwLock::new(SlotMap::new()),
            buffers: RwLock::new(SlotMap::new()),
            memory_binders: SwapLock::default(),
            timeline_semaphores: RwLock::new(SlotMap::new()),
            default_binder,
            default_binder_mappable,
            tmp_allocs: Arc::new(TmpAllocs {
                fallback_alloc: Arc::new(main_tmp_alloc),
                tmp_allocs,
            }),
            current_frame_index: 0,
            buffered_frames: attributes.buffered_frames,
        });
        let queue_scheduler = QueueScheduler::new(s.clone(), command_workers)
            .context_with(|| format_compact!("failed to create queue scheduler"))?;
        s.queue_scheduler.get_or_init(|| {
            queue_scheduler
        });
        Ok(s)
    }

    #[inline(always)]
    pub(crate) fn memory_layout(&self) -> MemoryLayout {
        self.memory_layout
    }

    #[inline(always)]
    pub(crate) fn thread_pool(&self) -> ThreadPool {
        self.thread_pool.clone()
    }

    #[inline(always)]
    pub(crate) fn tmp_alloc(
        &self,
    ) -> impl Arena<True> + ImmutArena + 'static
    {
        self.tmp_allocs.tmp_alloc()
    }

    #[inline(always)]
    pub(crate) fn vk(&self) -> &Arc<Vulkan> {
        &self.vk
    }

    #[inline(always)]
    pub(crate) fn queue_scheduler(&self) -> &QueueScheduler {
        unsafe {
            self.queue_scheduler.get().unwrap_unchecked()
        }
    }

    #[inline(always)]
    pub fn physical_device_info(&self) -> &PhysicalDeviceInfo {
        self.vk.physical_device_info()
    }

    #[inline(always)]
    pub(crate) fn shader_cache(&self) -> &ShaderCache {
        &self.shader_cache
    }

    #[inline(always)]
    pub fn default_memory_binder(&self) -> DefaultBinder {
        self.default_binder.clone()
    }

    #[inline(always)]
    pub fn default_memory_binder_mappable(&self) -> DefaultBinder {
        self.default_binder_mappable.clone()
    }

    #[inline(always)]
    pub fn supported_image_format<F: Format>(
        &self,
        formats: &[F],
        required_features: &[FormatFeature],
    ) -> Option<F>
    {
        let mut features = 0;
        required_features
            .iter()
            .map(|&f| features |= f)
            .count();
        for format in formats {
            let properties = unsafe {
                self.vk.instance()
                    .get_physical_device_format_properties(
                        self.vk.physical_device(), format.as_vk_format())
            };
            if has_bits!(
                properties.optimal_tiling_features,
                vk::FormatFeatureFlags::from_raw(features)
            ) {
                return Some(*format)
            }
        } 
        None
    }

    #[inline(always)]
    pub fn api_version(&self) -> Version {
        self.vk.physical_device_info().api_version()
    }

    #[inline(always)]
    pub fn create_shader(
        &mut self,
        attributes: ShaderAttributes,
    ) -> Result<Shader> {
        Ok(Shader::Pending(self.thread_pool.spawn_with_handle(Shader::new(
            attributes.to_owned(), self.api_version(),
        )).context("spawn error")?))
    }

    pub fn create_shader_set<const N_SHADERS: usize>(
        &mut self,
        shaders: [Shader; N_SHADERS],
        attributes: ShaderSetAttributes,
    ) -> Result<ShaderSetId>
    {
        self.shader_cache.create_shader_set(
            shaders,
            attributes,
            self.thread_pool.clone(),
            self.tmp_allocs.clone(),
        )
    }

    pub fn delete_shader_set(&mut self, id: ShaderSetId) -> Result<()> {
        self.shader_cache.delete_shader_set(id)
    }

    #[inline(always)]
    pub fn create_shader_resource_pool(
        &self,
        pool_sizes: impl IntoIterator<Item = (DescriptorType, u32)>,
        max_sets:u32,
    ) -> Result<ShaderResourcePoolId>
    {
        self.shader_resource_pools.modify(|pools| {
            Ok(ShaderResourcePoolId::new(pools.insert(ShaderResourcePool::new(
                self.vk.clone(),
                pool_sizes, max_sets
            ).context("failed to create shader resource pool")?)))
        })
    }

    #[inline(always)]
    pub fn destroy_shader_resource_pool(
        &mut self,
        id: ShaderResourcePoolId,
    ) {
        self.shader_resource_pools.modify(|pools| {
            pools.remove(id.slot_index()).ok();
        });
    }

    #[inline(always)]
    pub(crate) fn get_shader_resource_pools(
        &self
    ) -> impl Deref<Target = Arc<SlotMap<ShaderResourcePool>>>
    {
        self.shader_resource_pools.load()
    }

    #[inline(always)]
    pub fn allocate_shader_resources<F>(
        &self,
        pool_id: ShaderResourcePoolId,
        set_infos: &[ShaderDescriptorSetInfo],
        collect: F,
    ) -> Result<()>
        where
            F: FnMut(usize, ShaderResourceId)
    {
        let tmp_alloc = self.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let pools = self.shader_resource_pools.load();
        let pool = pools
            .get(pool_id.slot_index())
            .context("failed find pool")?;
        pool.allocate(set_infos, pool_id, &self.shader_cache, &tmp_alloc, collect)
            .context("failed to allocate shader resources")
    }

    #[inline(always)]
    pub fn free_shader_resources(
        &self,
        pool_id: ShaderResourcePoolId,
        resource_ids: &[ShaderResourceId],
    ) -> Result<()>
    {
        let queue_scheduler = self.queue_scheduler().read();
        let tmp_alloc = self.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let pools = self.shader_resource_pools.load();
        #[cfg(debug_assertions)]
        if let Some(id) = resource_ids.iter().find(|id| id.pool_id() != pool_id) {
            return Err(Error::just_context(format_compact!(
                "attempting to free shader resource {id} that was allocated from a different pool, expected pool {pool_id}",
            )))
        }
        let pool = pools
            .get(pool_id.slot_index())
            .context("failed to find pool")?;
        unsafe {
            pool.free(
                self,
                &queue_scheduler,
                resource_ids,
                &tmp_alloc,
            )
        }
    }

    #[inline(always)]
    pub fn update_shader_resources(
        &mut self,
        pool_id: ShaderResourcePoolId,
        image_updates: &[ShaderResourceImageUpdate],
        buffer_updates: &[ShaderResourceBufferUpdate],
        copies: &[ShaderResourceCopy],
    ) -> Result<()>
    {
        let tmp_alloc = self.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let queue_scheduler = self.queue_scheduler().read();
        let pools = self.shader_resource_pools.load();
        let pool = pools
            .get(pool_id.slot_index())
            .context_with(|| format_compact!(
                "invalid pool id {pool_id}"
            ))?;
        let mut pool = pool.write();
        let mut writes = FixedVec32
            ::with_capacity(image_updates.len() as u32 + buffer_updates.len() as u32, &tmp_alloc)
            .context("vec error")?; 
        let mut unpoison = FixedVec32::with_capacity(
            writes.capacity() + copies.len() as u32,
            &tmp_alloc,
        ).context("vec error")?;
        let mut buffer_infos = FixedVec32
            ::with_capacity(buffer_updates.len() as u32, &tmp_alloc)
            .context("vec error")?;
        let finished_frame = self.get_semaphore_counter_value(
            queue_scheduler.get_frame_semaphore_id()
        )?;
        for update in buffer_updates {
            #[cfg(debug_assertions)]
            if update.resource_id.pool_id() != pool_id {
                return Err(Error::just_context(format_compact!(
                    "buffer update shader resource {} was allocated from a different pool, expected pool {pool_id}",
                    update.resource_id,
                )))
            }
            let mut resource = pool
                .get_shader_resource_for_update(
                    update.resource_id,
                    finished_frame,
                )
                .context_with(|| format_compact!(
                    "failed to get shader resource {:?}",
                    update.resource_id,
                ))?;
            let (ty, vk_infos) = resource
                .update_buffer(self, update, &tmp_alloc)
                .context_with(|| format_compact!(
                    "failed to update buffer shader resource {}",
                    update.resource_id,
                ))?;
            buffer_infos.push(vk_infos);
            let vk_infos = buffer_infos.last().unwrap();
            let write = vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                dst_set: resource.descriptor_set(),
                dst_binding: update.binding,
                dst_array_element: update.starting_index,
                descriptor_count: vk_infos.len() as u32,
                descriptor_type: ty,
                p_buffer_info: vk_infos.as_ptr(),
                ..Default::default()
            };
            unpoison.push(resource.into_inner());
            writes.push(write);
        }
        let mut image_infos = FixedVec32
            ::with_capacity(image_updates.len() as u32, &tmp_alloc)
            .context("vec error")?;
        for update in image_updates {
            #[cfg(debug_assertions)]
            if update.resource_id.pool_id() != pool_id {
                return Err(Error::just_context(format_compact!(
                    "image update shader resource {} was allocated from a different pool, expected pool {pool_id}",
                    update.resource_id,
                )))
            }
            let mut resource = pool
                .get_shader_resource_for_update(
                    update.resource_id,
                    finished_frame,
                ).context_with(|| format_compact!(
                    "failed to get shader resource {}",
                    update.resource_id,
                ))?;
            let (ty, vk_infos) = resource
                .update_image(self, update, &tmp_alloc)
                .context_with(|| format_compact!(
                    "failed to update image shader resource {}",
                    update.resource_id,
                ))?;
            image_infos.push(vk_infos);
            let vk_infos = image_infos.last().unwrap();
            let write = vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                dst_set: resource.descriptor_set(),
                dst_binding: update.binding,
                dst_array_element: update.starting_index,
                descriptor_count: vk_infos.len() as u32,
                descriptor_type: ty,
                p_image_info: vk_infos.as_ptr(),
                ..Default::default()
            };
            unpoison.push(resource.into_inner());
            writes.push(write);
        }
        let mut vk_copies = FixedVec32
            ::with_capacity(copies.len() as u32, &tmp_alloc)
            .context("vec error")?;
        for copy in copies {
            #[cfg(debug_assertions)]
            if copy.src_resource_id.pool_id() != pool_id {
                return Err(Error::just_context(format_compact!(
                    "shader resource copy source {} was allocated from a different pool, expected pool {pool_id}",
                    copy.src_resource_id,
                )))
            }
            #[cfg(debug_assertions)]
            if copy.dst_resource_id.pool_id() != pool_id {
                return Err(Error::just_context(format_compact!(
                    "shader resource copy destination {} was allocated from a different pool, expected pool {pool_id}",
                    copy.dst_resource_id,
                )))
            }
            let src = pool 
                .get_shader_resource_handle(copy.src_resource_id.inner_id())
                .context_with(|| format_compact!(
                    "failed to get source shader resource {} for copy",
                    copy.src_resource_id,
                ))?;
            let mut dst = pool
                .get_shader_resource_for_update(
                    copy.dst_resource_id,
                    finished_frame,
                ).context_with(|| format_compact!(
                    "failed to get destination shader resource {} for copy",
                    copy.dst_resource_id,
                ))?;
            let vk_copy = unsafe { dst.copy_from(
                src,
                copy.src_binding,
                copy.src_starting_index,
                copy.dst_binding,
                copy.dst_starting_index,
                copy.array_count,
            ) }.context_with(|| format_compact!(
                "failed to copy source shader resource {} to destination shader resource {}",
                copy.src_resource_id, copy.dst_resource_id,
            ))?; 
            unpoison.push(dst.into_inner());
            vk_copies.push(vk_copy);
        }
        unsafe {
            self.vk.device().update_descriptor_sets(&writes, &vk_copies);
            for mut handle in unpoison {
                handle.unpoison();
            }
        }
        Ok(())
    }

    #[inline(always)]
    pub fn create_pipeline_cache(
        &mut self,
        initial_data: Option<&[u8]>,
    ) -> Result<PipelineCache>
    {
        let initial_data = initial_data.unwrap_or(&[]);
        let info = vk::PipelineCacheCreateInfo {
            s_type: vk::StructureType::PIPELINE_CACHE_CREATE_INFO,
            initial_data_size: initial_data.len(),
            p_initial_data: initial_data.as_ptr() as _,
            ..Default::default()
        };
        let device = self.vk.device();
        let handle = unsafe {
            device.create_pipeline_cache(&info, None)
                .context("failed to create pipeline cache")?
        };
        unsafe {
            Ok(PipelineCache::new(self.vk.clone(), handle))
        }
    } 

    #[inline(always)]
    pub(crate) fn reserve_pipeline_batch_slot(&self) -> PipelineBatchId {
        PipelineBatchId(self.pipeline_batches.modify(|data| {
            let idx = data.len();
            data.push(OnceLock::new());
            idx
        }))
    }

    #[inline(always)]
    pub(crate) fn init_pipeline_batch(
        &self,
        id: PipelineBatchId,
        batch: PipelineBatch,
    ) {
        if let Some(b) = self.pipeline_batches.load().get(id.0 as usize) {
            b.get_or_init(|| batch);
        }
    }

    pub fn destroy_graphics_pipeline(&mut self, id: GraphicsPipelineId) -> Result<()> {
        self.graphics_pipelines
            .remove(id.0)
            .context_with(|| format_compact!(
                "invalid graphics pipeline id {id:?}"
            ))?;
        Ok(())
    }

    pub fn create_compute_pipelines(
        &mut self,
        attributes: &[ComputePipelineAttributes],
        cache_id: Option<PipelineCacheId>,
        mut collect: impl FnMut(usize, ComputePipelineId),
    ) -> Result<()>
    {
 
    }

    pub fn destroy_compute_pipeline(&mut self, id: ComputePipelineId) -> Result<()> {
        self.compute_pipelines
            .remove(id.0)
            .context_with(|| format_compact!("invalid compute pipeline id {id:?}"))?;
        Ok(())
    }

    pub(crate) fn get_shader_set_resources<'a, F, Alloc>(
        &self,
        shader_set: &ShaderSetInner,
        tmp_alloc: &'a Alloc,
        mut f: F,
    ) -> Result<FixedVec32<'a, vk::DescriptorSet, Alloc>>
        where
            Alloc: LocalAlloc,
            F: FnMut(u32) -> Option<ShaderResourceId>,
    {
        let sets = shader_set.descriptor_set_layouts();
        let mut res = FixedVec32
            ::with_capacity(sets.len() as u32, tmp_alloc)
            .context("alloc failure")?;
        let pools = self.shader_resource_pools.load();
        for (i, set) in sets.iter().enumerate() {
            let id = f(i as u32);
            if let Some(id) = id &&
                !set.bindings.is_empty()
            {
                let pool = pools
                    .get(id.pool_id().slot_index())
                    .context("failed to find resource pool")?;
                let descriptor_set = pool
                    .get_descriptor_set(id.inner_id())
                    .context_with(|| format_compact!(
                        "failed to get shader resource from shader resource pool {:?}",
                        id.pool_id(),
                    ))?;
                res.push(descriptor_set);
            }
            else {
                res.push(vk::DescriptorSet::null());
            }
        }
        Ok(res)
    }

    pub(crate) fn get_shader_set_push_constant_ranges<'a, 'b, Alloc, F>(
        &self,
        shader_set: &ShaderSetInner,
        tmp_alloc: &'a Alloc,
        mut f: F,
    ) -> Result<FixedVec32<'a, (PushConstantRange, &'b [u8]), Alloc>>
        where
            Alloc: LocalAlloc,
            F: FnMut(PushConstantRange) -> &'b [u8],
    {
        let push_constant_ranges = shader_set.push_constant_ranges();
        let mut res = FixedVec32
            ::with_capacity(push_constant_ranges.len() as u32, tmp_alloc)
            .context("alloc failure")?;
        for &pc in push_constant_ranges.iter() {
            res.push((pc, f(pc)));
        }
        Ok(res)
    }

    #[inline(always)]
    pub(crate) fn get_graphics_pipeline(&self, id: GraphicsPipelineId) -> Result<&GraphicsPipeline> {
        self.graphics_pipelines
            .get(id.0)
            .context("failed to find graphics pipeline")
    }

    #[inline(always)]
    pub(crate) fn get_compute_pipeline(&self, id: ComputePipelineId) -> Result<&ComputePipeline> {
        self.compute_pipelines
            .get(id.0)
            .context("failed to find compute pipeline")
    }

    #[inline(always)]
    pub fn create_buffer(
        &mut self,
        size: u64,
        usage: &[BufferUsage],
        binder: ResourceBinder,
    ) -> Result<BufferWriteGuard<'_>>
    {
        if size == 0 {
            return Err(Error::new(MemoryBinderError::ZeroSizeAlloc, "buffer size was zero"))
        }
        let mut buffers = self.buffers.write();
        let mut vk_usage = vk::BufferUsageFlags::from_raw(0);
        for usage in usage {
            vk_usage |= vk::BufferUsageFlags::from_raw(usage.as_raw());
        }
        let properties = BufferProperties {
            size,
            usage: vk_usage,
            create_flags: Default::default(),
        };
        let buffer =
            match binder {
                ResourceBinder::DefaultBinder => {
                    BufferMeta
                        ::new(self.vk.clone(), properties, &mut self.default_binder)
                        .context("failed to create buffer")?
                },
                ResourceBinder::DefaultBinderMappable => {
                    BufferMeta
                        ::new(self.vk.clone(), properties, &mut self.default_binder_mappable)
                        .context("failed to create buffer")?
                },
                ResourceBinder::LinearBinder(id) => {
                    let binders = self.linear_binders.load();
                    let binder = binders
                        .get(id.0).context("failed to find linear binder")?;
                    BufferMeta
                        ::new(self.vk.clone(), properties, &mut *binder.write())
                        .context("failed to create buffer")?
                },
                ResourceBinder::Owned(binder) => {
                    BufferMeta
                        ::new(self.vk.clone(), properties, binder)
                        .context("failed to create buffer")?
                },
            };
        let id = BufferId(buffers.insert(buffer));
        Ok(BufferWriteGuard::new(id, self.buffers.write()).unwrap())
    }

    #[inline(always)]
    pub fn is_buffer_valid(&self, id: BufferId) -> bool {
        self.buffers.read().contains(id.0)
    }

    #[inline(always)]
    pub(crate) fn read_buffer(&self, id: BufferId) -> Result<BufferReadGuard<'_>> {
        BufferReadGuard::new(id, self.buffers.read())
        .ok_or_else(|| Error::just_context(format_compact!(
            "invalid buffer id {id}"
        )))
    }

    #[inline(always)]
    pub(crate) fn write_buffer(&self, id: BufferId) -> Result<BufferWriteGuard<'_>> {
        BufferWriteGuard::new(id, self.buffers.write())
        .ok_or_else(|| Error::just_context(format_compact!(
            "invalid buffer id {id}"
        )))
    }

    #[inline(always)]
    pub fn destroy_resources(
        &mut self,
        buffers: &[BufferId],
        images: &[ImageId]
    ) -> Result<()>
    {
        let mut all_buffers = self.buffers.write();
        let mut all_images = self.images.write();
        let tmp_alloc = self.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let mut cached_buffers = FixedVec32::with_capacity(
            buffers.len() as u32,
            &tmp_alloc,
        )?;
        let mut cached_images = FixedVec32::with_capacity(
            images.len() as u32,
            &tmp_alloc,
        )?;
        let mut max_frame = 0;
        let pools = self.shader_resource_pools.load();
        for &id in buffers {
            let buffer = all_buffers
                .remove(id.0)
                .context_with(|| format_compact!(
                    "invalid buffer id {id}",
                ))?;
            max_frame = max_frame.max(buffer.get_last_used_frame());
            cached_buffers.push(buffer);
            for pool in pools.values() {
                pool.buffer_delete(id);
            }
        }
        for &id in images {
            let image = all_images
                .remove(id.0)
                .context_with(|| format_compact!(
                    "invalid image id {id}"
                ))?;
            max_frame = max_frame.max(image.get_last_used_frame());
            cached_images.push(image);
            for pool in pools.values() {
                pool.image_delete(id);
            }
        }
        let semaphore = self.get_timeline_semaphore(self
            .queue_scheduler()
            .read()
            .get_frame_semaphore_id()
        ).unwrap();
        let wait_info = vk::SemaphoreWaitInfo {
            s_type: vk::StructureType::SEMAPHORE_WAIT_INFO,
            semaphore_count: 1,
            p_semaphores: &semaphore,
            p_values: &max_frame,
            ..Default::default()
        };
        unsafe {
            if let Err(res) = self.vk.timeline_semaphore_device().wait_semaphores(
                &wait_info,
                self.vk.frame_timeout(),
            ) {
                if res == vk::Result::TIMEOUT {
                    return Err(Error::just_context(format_compact!(
                        "frame timeout {} nanoseconds reached at {}", self.vk.frame_timeout(),
                        location!(),
                    )))
                } else {
                    return Err(Error::new(res, "unexpected vulkan error"))
                }
            }
        }
        Ok(())
    }

    #[inline(always)]
    pub fn create_sampler(
        &mut self,
        attributes: SamplerAttributes,
    ) -> Result<Sampler>
    {
        Ok(attributes
            .build(self.vk.device())
            .context("failed to create sampler")?
        )
    }

    #[inline(always)]
    pub fn create_image(
        &mut self,
        attributes: ImageAttributes,
        binder: ResourceBinder,
    ) -> Result<ImageId>
    {
        let image =
            match binder {
                ResourceBinder::DefaultBinder => {
                    attributes
                        .build(self.vk.clone(), &mut self.default_binder)
                        .context("failed to create image")?
                },
                ResourceBinder::DefaultBinderMappable => {
                    attributes
                        .build(self.vk.clone(), &mut self.default_binder_mappable)
                        .context("failed to create image")?
                }
                ResourceBinder::LinearBinder(id) => {
                    let binders = self.linear_binders.load();
                    let binder = binders
                        .get(id.0)
                        .context_with(|| format_compact!("invalid linear binder id {id}"))?;
                    attributes
                        .build(self.vk.clone(), &mut *binder.write())
                        .context("failed to create image")?
                }
                ResourceBinder::Owned(binder) => {
                    attributes
                        .build(self.vk.clone(), binder)
                        .context("failed to create image")?
                },
            };
        Ok(ImageId(
            self.images.write().insert(image)
        ))
    }

    #[inline(always)]
    pub fn is_image_valid(&self, id: ImageId) -> bool {
        self.images.read().contains(id.0)
    }

    #[inline(always)]
    pub fn image_mip_levels(&self, id: ImageId) -> Option<(u32, Dimensions)> {
        let properties = self.images.read().get(id.0).ok()?.properties();
        Some((properties.mip_levels, properties.dimensions))
    }

    #[inline(always)]
    pub(crate) fn read_buffers(&self) -> ResourceReadGuard<'_, BufferMeta, BufferId>
    {
        ResourceGuard::new(self.buffers.read())
    }

    #[inline(always)]
    pub(crate) fn write_buffers(&self) -> ResourceWriteGuard<'_, BufferMeta, BufferId>
    {
        ResourceGuard::new(self.buffers.write())
    }

    #[inline(always)]
    pub(crate) fn read_images(&self) -> ResourceReadGuard<'_, ImageMeta, ImageId>
    {
        ResourceGuard::new(self.images.read())
    }

    #[inline(always)]
    pub(crate) fn write_images(&self) -> ResourceWriteGuard<'_, ImageMeta, ImageId>
    {
        ResourceGuard::new(self.images.write())
    }

    #[inline(always)]
    pub fn create_memory_binder<Attr>(
        &self,
        attributes: Attr,
    ) -> Result<MemoryBinderId>
        where Attr: MemoryBinderAttributes,
    {
        self.memory_binders.modify(|binders| {
            Ok(MemoryBinderId(binders.insert(MemoryBinderResource::new(
                attributes
                    .build(self.vk.clone())
                    .context_with(|| format_compact!(
                        "failed to create {}", Attr::NAME,
                    ))?
            ))))
        })
    }

    #[inline(always)]
    pub fn destroy_memory_binder(&self, id: MemoryBinderId) {
        self.memory_binders.modify(|binders| {
            binders.remove(id.0).ok()
        });
    }

    #[inline(always)]
    pub fn get_memory_binder(
        &self,
        id: MemoryBinderId
    ) -> Result<MemoryBinderResource>
    {
        Ok(self.memory_binders
            .load()
            .get(id.0)
            .context_with(|| format_compact!(
                "invalid memory binder id {id}"
            ))?.clone()
        )
    }

    /// Creates timeline semaphores from an iterator over their initial values.
    #[inline(always)]
    pub fn create_timeline_semaphores<I, F>(
        &self,
        initial_values: I,
        mut collect: F,
    ) -> Result<()>
        where
            F: FnMut(u32, TimelineSemaphoreId),
            I: ExactSizeIterator<Item = u64>,
    {
        if initial_values.len() == 0 {
            return Ok(())
        }
        let tmp_alloc = self.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let mut indices = FixedVec32::with_capacity(
            initial_values.len().try_into().unwrap(),
            &tmp_alloc,
        )?;
        let mut err = None;
        let mut semaphores = self.timeline_semaphores.write();
        for initial_value in initial_values {
            if err.is_some() {
                break;
            }
            let mut type_info = vk::SemaphoreTypeCreateInfo {
                s_type: vk::StructureType::SEMAPHORE_TYPE_CREATE_INFO,
                semaphore_type: vk::SemaphoreType::TIMELINE,
                initial_value,
                ..Default::default()
            };
            let semaphore_info = vk::SemaphoreCreateInfo {
                s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
                ..Default::default()
            }.push_next(&mut type_info);
            match unsafe {
                self.vk.device()
                    .create_semaphore(&semaphore_info, None)
            } {
                Ok(handle) => {
                    let index = semaphores.insert(handle);
                    indices.push(index);
                },
                Err(e) => { err = Some(e); }
            }
        }
        if let Some(err) = err {
            return Err(err).context("failed to create timeline semaphore")
        }
        for (i, index) in indices.into_iter().enumerate() {
            collect(i as u32, TimelineSemaphoreId(index))
        }
        Ok(())
    }

    /// Gets the counter value of a timeline semaphore.
    #[inline(always)]
    pub fn get_semaphore_counter_value(&self, id: TimelineSemaphoreId) -> Result<u64> {
        let &handle = self.timeline_semaphores
            .read()
            .get(id.0)
            .context_with(|| format_compact!("failed to find timeline semaphore {id}"))?;
        unsafe {
            self.vk.timeline_semaphore_device()
                .get_semaphore_counter_value(handle)
                .context("failed to get timeline semaphore value")
        }
    }

    /// Waits for previous semaphores until `timeout` where `timeout` is in nanoseconds.
    ///
    /// Returns Ok(true) on success, Ok(false) on timeout and Err(err) if there's another error.
    #[inline(always)]
    pub fn wait_for_semaphores(
        &self,
        semaphores: &[(TimelineSemaphoreId, u64)],
        timeout: u64,
    ) -> Result<bool> {
        let tmp_alloc = self.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let n_semaphores = semaphores.len() as u32;
        let mut handles = FixedVec32
            ::with_capacity(n_semaphores, &tmp_alloc)?;
        let mut values = FixedVec32
            ::with_capacity(n_semaphores, &tmp_alloc)?;
        let read = self.timeline_semaphores.read();
        for &(id, value) in semaphores {
            let &semaphore = read
                .get(id.0)
                .context("failed to find timeline semaphore")?;
            handles.push(semaphore);
            values.push(value);
        }
        let wait_info = vk::SemaphoreWaitInfo {
            s_type: vk::StructureType::SEMAPHORE_WAIT_INFO,
            semaphore_count: semaphores.len() as u32,
            p_semaphores: handles.as_ptr(),
            p_values: values.as_ptr(),
            ..Default::default()
        };
        let res = unsafe {
            self.vk.timeline_semaphore_device().wait_semaphores(
                &wait_info,
                timeout,
            )
        };
        if let Err(err) = res {
            if err == vk::Result::TIMEOUT {
                return Ok(false)
            }
            return Err(Error::new(err, "unexpected vulkan error"))
        }
        Ok(true)
    }

    #[inline(always)]
    pub fn destroy_timeline_semaphores(&self, ids: &[TimelineSemaphoreId]) {
        let mut semaphores = self.timeline_semaphores.write();
        for id in ids {
            if let Ok(handle) = semaphores.remove(id.0) {
                unsafe {
                    self.vk.device().destroy_semaphore(handle, None);
                }
            }
        }
    }

    #[inline(always)]
    pub(crate) fn get_timeline_semaphore(&self, id: TimelineSemaphoreId) -> Result<vk::Semaphore> {
        self.timeline_semaphores
            .read()
            .get(id.0).copied()
            .context("failed to find timeline semaphore")
    }

    #[inline(always)]
    pub(crate) fn update(&mut self) -> Result<()> {
        let pools = self.shader_resource_pools.load();
        for pool in pools.values_mut() {
            pool.update(self);
        }
        Ok(())
    }
}

impl Drop for Gpu {

    fn drop(&mut self) {
        unsafe {
            log::info!("cleaning up GPU");
            let device = self.vk.device();
            let semaphores = self.timeline_semaphores.write();
            for &handle in self.timeline_semaphores.write().values() {
                device.destroy_semaphore(handle, None);
            }
        }
    }
}
