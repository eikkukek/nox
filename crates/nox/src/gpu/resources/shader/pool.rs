use core::{
    ops::{Deref, DerefMut},
    num::NonZeroU64,
    ptr::NonNull,
};

use nox_ash::vk::Handle;

use ahash::{AHashMap, AHashSet};

use nox_mem::{
    vec::NonNullVec32,
    alloc::dealloc,
};

use nox_alloc::arena::{self, CellArena};

use super::*;

#[repr(i32)]
#[derive(AsRaw, Clone, Copy, PartialEq, Eq, Debug, Display)]
pub(crate) enum ShaderImageLayout {
    #[display("General")]
    General = vk::ImageLayout::GENERAL.as_raw(),
    #[display("Sampled Read Only")]
    SampledReadOnly = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL.as_raw(),
}

impl ShaderImageLayout {

    #[inline(always)]
    pub fn combine(self, other: Self) -> Self {
        if self != other {
            ShaderImageLayout::General
        } else {
            self
        }
    }

    #[inline(always)]
    pub fn access_mask(self) -> vk::AccessFlags2 {
        match self {
            Self::General => vk::AccessFlags2::SHADER_READ | vk::AccessFlags2::SHADER_WRITE,
            Self::SampledReadOnly => vk::AccessFlags2::SHADER_SAMPLED_READ,
        }
    }
}

impl From<ShaderImageLayout> for vk::ImageLayout {

    #[inline(always)] 
    fn from(value: ShaderImageLayout) -> vk::ImageLayout {
        vk::ImageLayout::from_raw(value.as_raw())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ShaderResourceDescriptorBuffer {
    pub buffer: Option<(BufferId, vk::DeviceSize, vk::DeviceSize)>,
    pub buffer_track_id: TrackedShaderResourceId<BufferMeta>,
}

#[derive(Clone)]
pub(crate) struct ShaderResourceDescriptorImage {
    pub sampler: Option<Sampler>,
    pub image: Option<(ImageId, ShaderImageLayout, Option<ImageSubresourceRange>)>,
    pub image_track_id: TrackedShaderResourceId<Image>,
}

#[derive(Display)]
pub(super) enum ShaderResourceDescriptors {
    #[display("Buffer")]
    Buffers(NonNullVec32<'static, ShaderResourceDescriptorBuffer>),
    #[display("Image")]
    Images(NonNullVec32<'static, ShaderResourceDescriptorImage>),
}

pub(crate) struct ShaderResourceBinding {
    binding: u32,
    descriptor_type: DescriptorType,
    descriptors: ShaderResourceDescriptors,
}

impl ShaderResourceBinding {

    #[inline(always)]
    pub fn binding(&self) -> u32 {
        self.binding
    }

    #[inline(always)]
    pub fn descriptor_type(&self) -> DescriptorType {
        self.descriptor_type
    }

    #[inline(always)]
    pub fn descriptor_buffers(&self) -> &[ShaderResourceDescriptorBuffer] {
        match &self.descriptors {
            ShaderResourceDescriptors::Buffers(descriptors) => {
                descriptors
            },
            ShaderResourceDescriptors::Images(_) => {
                slice![]
            },
        }
    }

    #[inline(always)]
    pub fn descriptor_buffers_mut(&mut self) -> &mut [ShaderResourceDescriptorBuffer] {
        match &mut self.descriptors {
            ShaderResourceDescriptors::Buffers(descriptors) => {
                descriptors
            },
            ShaderResourceDescriptors::Images(_) => {
                slice_mut![]
            },
        }
    }

    #[inline(always)]
    pub fn descriptor_images(&self) -> &[ShaderResourceDescriptorImage] {
        match &self.descriptors {
            ShaderResourceDescriptors::Images(descriptors) => {
                descriptors
            },
            ShaderResourceDescriptors::Buffers(_) => {
                slice![]
            },
        }
    }

    #[inline(always)]
    pub fn descriptor_images_mut(&mut self) -> &mut [ShaderResourceDescriptorImage] {
        match &mut self.descriptors {
            ShaderResourceDescriptors::Images(descriptors) => {
                descriptors
            },
            ShaderResourceDescriptors::Buffers(_) => {
                slice_mut![]
            },
        }
    }
}

pub(crate) struct ShaderResource {
    descriptor_set: vk::DescriptorSet,
    bindings: NonNullVec32<'static, ShaderResourceBinding>,
    shader_stage_mask: vk::ShaderStageFlags,
    arena: (NonNull<u8>, usize),
    last_used_frame: u64,
    flags: u32,
}

unsafe impl Send for ShaderResource {}
unsafe impl Sync for ShaderResource {}

impl ShaderResource {

    const MAYBE_POISONED: u32 = 0x1;
    const IS_VALID: u32 = 0x2;

    #[inline(always)]
    fn new(
        bindings: NonNullVec32<'static, ShaderResourceBinding>,
        shader_stage_mask: vk::ShaderStageFlags,
        arena: arena::Inner,
    ) -> Self
    {
        Self {
            descriptor_set: vk::DescriptorSet::null(),
            bindings,
            shader_stage_mask,
            arena: arena.into_raw_parts(),
            last_used_frame: 0,
            flags: 0,
        }
    }

    #[inline(always)]
    pub fn is_maybe_poisoned(&self) -> bool {
        self.flags & Self::MAYBE_POISONED == Self::MAYBE_POISONED
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.flags & Self::IS_VALID == Self::IS_VALID
    }

    fn validate_bindings(
        &mut self,
        resources: &Resources,
    ) {
        self.flags = 0;
        if self.is_maybe_poisoned() {
            for binding in &mut self.bindings {
                for buffer in binding.descriptor_buffers_mut() {
                    buffer.buffer.take();
                }
                for image in binding.descriptor_images_mut() {
                    image.sampler.take();
                    image.image.take();
                }
            }
            self.flags &= !(Self::MAYBE_POISONED | Self::IS_VALID);
            return
        }
        for binding in &mut self.bindings {
            let ty = binding.descriptor_type();
            for buffer in binding.descriptor_buffers_mut() {
                if let Some((id, _, _)) = buffer.buffer {
                    if !resources.is_buffer_valid(id) {
                        self.flags &= !Self::IS_VALID;
                        return
                    }
                } else {
                    self.flags &= !Self::IS_VALID;
                    return 
                }
            }
            let requires_sampler = ty.requires_sampler();
            let requires_image = ty.requires_image();
            for image in binding.descriptor_images_mut() {
                if requires_sampler && image.sampler.is_none() {
                    self.flags &= !Self::IS_VALID;
                    return
                }
                if requires_image {
                    if let Some((id, _, _)) = image.image {
                        if !resources.is_image_valid(id) {
                            self.flags &= !Self::IS_VALID;
                            return
                        }
                    } else {
                        self.flags &= !Self::IS_VALID;
                        return
                    }
                }
            }
        }
        self.flags |= Self::IS_VALID;
    }
}

impl Drop for ShaderResource {

    fn drop(&mut self) {
        unsafe {
            for binding in &mut self.bindings {
                match &mut binding.descriptors {
                    ShaderResourceDescriptors::Buffers(bufs) => {
                        bufs.drop_in_place();
                    },
                    ShaderResourceDescriptors::Images(imgs) => {
                        imgs.drop_in_place();
                    },
                }
            }
            self.bindings.drop_in_place();
            let layout = Layout::from_size_align_unchecked(
                self.arena.1, arena::max_align(),
            );
            dealloc(self.arena.0.as_ptr(), layout);
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Display)] #[display("{0}")]
pub struct ShaderResourcePoolId(SlotIndex<ShaderResourcePool>);

impl ShaderResourcePoolId {

    #[inline(always)]
    pub(crate) fn new(index: SlotIndex<ShaderResourcePool>) -> Self {
        Self(index)
    }

    #[inline(always)]
    pub(crate) fn slot_index(self) -> SlotIndex<ShaderResourcePool> {
        self.0
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)] #[display("(pass id: {0}, inner id: {0})")]
pub struct ShaderResourceId(ShaderResourcePoolId, ShaderResourceInnerId);

impl ShaderResourceId {

    #[inline(always)]
    pub(crate) fn new(pool_id: ShaderResourcePoolId, inner_id: ShaderResourceInnerId) -> Self {
        Self(pool_id, inner_id)
    }

    #[inline(always)]
    pub fn pool_id(self) -> ShaderResourcePoolId {
        self.0
    }

    #[inline(always)]
    pub(crate) fn inner_id(self) -> ShaderResourceInnerId {
        self.1
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(super) struct TrackedShaderResourceId<T>(pub SlotIndex<ShaderResourceInnerId>, pub PhantomData<T>);

impl<T> Default for TrackedShaderResourceId<T> {

    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}

impl<T> Clone for TrackedShaderResourceId<T> {

    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TrackedShaderResourceId<T> {}

#[derive(Default, Clone, Copy)]
pub struct ShaderResourceBufferInfo {
    pub buffer_id: BufferId,
    pub offset: u64,
    pub size: u64,
}

#[derive(Clone)]
pub struct ShaderResourceImageInfo {
    pub sampler: Option<Sampler>,
    pub image_source: Option<(ImageId, Option<ImageRange>)>,
}

#[derive(Default, Clone, Copy)]
pub struct ShaderResourceBufferUpdate<'a> {
    pub resource_id: ShaderResourceId,
    pub binding: u32,
    pub starting_index: u32,
    pub infos: &'a [ShaderResourceBufferInfo],
}

#[derive(Clone, Copy)]
pub struct ShaderResourceImageUpdate<'a> {
    pub resource_id: ShaderResourceId,
    pub binding: u32,
    pub starting_index: u32,
    pub infos: &'a [ShaderResourceImageInfo],
}

#[derive(Clone, Copy)]
pub struct ShaderResourceCopy {
    pub src_resource_id: ShaderResourceId,
    pub src_binding: u32,
    pub src_starting_index: u32,
    pub dst_resource_id: ShaderResourceId,
    pub dst_starting_index: u32,
    pub dst_binding: u32,
    pub array_count: u32,
}

struct Pool {
    size: u32,
    used: u32,
}

struct Inner {
    vk: Arc<Vulkan>,
    handle: vk::DescriptorPool,
    allocated_sets: u32,
    max_sets: u32,
    shader_resources: SlotMap<ShaderResource>,
    descriptor_cache: Vec32<Option<NonZeroU64>>,
    pools: AHashMap<DescriptorType, Pool>,
    pending_validations: AHashSet<ShaderResourceInnerId>,
    tracked_buffers: AHashMap<BufferId, SlotMap<ShaderResourceInnerId>>,
    tracked_images: AHashMap<ImageId, SlotMap<ShaderResourceInnerId>>,
    poisoned: bool,
}

impl Inner {

    #[inline(always)]
    fn update(&mut self, resources: &Resources) {
        for &id in self.pending_validations.iter() {
            if let Ok(resource) = self.shader_resources.get_mut(id.0) {
                resource.validate_bindings(resources);
            }
        }
        self.pending_validations.clear();
        self.poisoned = false;
    }
}

#[derive(Clone)]
pub(crate) struct ShaderResourcePool {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Display)] #[display("{0}")]
pub(crate) struct ShaderResourceInnerId(pub SlotIndex<ShaderResource>);

pub(crate) struct ShaderResourcePoolWriteGuard<'a> {
    inner: RwLockWriteGuard<'a, Inner>,
}

impl ShaderResourcePool {

    pub fn new(
        vk: Arc<Vulkan>,
        pool_sizes: impl IntoIterator<Item = (DescriptorType, u32)>,
        max_sets: u32,
    ) -> Result<Self>
    {
        let mut pools = AHashMap::<DescriptorType, Pool>::default();
        for (ty, size) in pool_sizes.into_iter() {
            pools.entry(ty)
                .and_modify(|pool| pool.size += size)
                .or_insert(Pool { size, used: 0, });

        }
        let mut pool_sizes = Vec32::default();
        for (&ty, pool) in &pools {
            if ty.is_unsupported() {
                return Err(Error::just_context(format_compact!(
                    "unsupported descriptor type {ty}",
                )))
            }
            pool_sizes.push(vk::DescriptorPoolSize {
                ty: ty.into(),
                descriptor_count: pool.size,
            });
        }
        let info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            flags: vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET |
                    vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND,
            max_sets,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            ..Default::default()
        };
        let handle = unsafe {
            vk.device()
                .create_descriptor_pool(&info, None)
                .context("failed to create descriptor pool")?
        };
        Ok(Self {
            inner: Arc::new(RwLock::new(Inner {
                vk,
                handle,
                allocated_sets: 0,
                max_sets,
                shader_resources: SlotMap::default(),
                descriptor_cache: vec32![],
                pools,
                tracked_buffers: AHashMap::default(),
                tracked_images: AHashMap::default(),
                pending_validations: AHashSet::default(),
                poisoned: false,
            }))
        })
    }

    pub fn buffer_delete(&self, id: BufferId) {
        let mut inner = self.inner.write();
        if let Some(tracked) = inner.tracked_buffers.remove(&id) {
            for &resource_id in tracked.values() {
                inner.pending_validations.insert(resource_id);
            }
        }
    }

    pub fn image_delete(&self, id: ImageId) {
        let mut inner = self.inner.write();
        if let Some(tracked) = inner.tracked_images.remove(&id) {
            for &resource_id in tracked.values() {
                inner.pending_validations.insert(resource_id);
            }
        }
    }

    pub fn allocate<Alloc, F>(
        &self,
        set_infos: &[ShaderDescriptorSetInfo],
        pool_id: ShaderResourcePoolId,
        shader_cache: &ShaderCache,
        tmp_alloc: &Alloc,
        mut collect: F,
    ) -> Result<()>
        where
            Alloc: LocalAlloc<Error = Error>,
            F: FnMut(usize, ShaderResourceId),
    {
        let mut inner = self.inner.write();
        let count = set_infos.len() as u32;
        if inner.allocated_sets + count > inner.max_sets {
            return Err(Error::just_context(format_compact!(
                "pool was full with max sets {}", inner.max_sets,
            )))
        }
        let mut set_layouts = FixedVec32
            ::with_capacity(set_infos.len() as u32, tmp_alloc)?;
        let mut resources = FixedVec32
            ::with_capacity(set_infos.len() as u32, tmp_alloc)?;
        for (i, info) in set_infos.iter().enumerate() {
            let set = shader_cache
                .get_shader_set(info.shader_set_id)
                .context_with(|| format_compact!(
                    "failed to get shader set {} at index {i}",
                    info.shader_set_id
                ))?;
            let layout = &set
                .descriptor_set_layouts()
                .get(info.descriptor_set_index as usize)
                .ok_or_else(|| Error::just_context(format_compact!(
                    "invalid descriptor set index {} for shader set {}",
                    info.descriptor_set_index, info.shader_set_id,
                )))?;
            let alloc_size: usize = layout.bindings
                .iter().map(|binding|
                    size_of::<ShaderResourceBinding>() +
                    if binding.descriptor_type.is_buffer() {
                        size_of::<ShaderResourceDescriptorBuffer>() * binding.descriptor_count as usize
                    } else if binding.descriptor_type.is_image() {
                        size_of::<ShaderResourceDescriptorImage>() * binding.descriptor_count as usize
                    } else {
                        0
                    }
                )
                .sum();
            let alloc = CellArena::new(alloc_size)?;
            let mut bindings = NonNullVec32
                ::with_capacity(layout.bindings.len(), &alloc)?
                .into_static();
            bindings.try_extend(layout.bindings
                .iter()
                .map(|binding| {
                    if let Some(pool) = inner.pools.get_mut(&binding.descriptor_type) {
                        let used = pool.used + count;
                        if used > pool.size {
                            Err(Error::just_context(format_compact!(
                                "maximum capacity {} for descriptor type {} reached",
                                pool.size, binding.descriptor_type,
                            )))
                        } else {
                            pool.used = used;
                            let descriptors =
                                if binding.descriptor_type.is_buffer() {
                                    let mut bufs = NonNullVec32::with_capacity(
                                        binding.descriptor_count,
                                        &alloc,
                                    )?.into_static();
                                    bufs.resize(
                                        binding.descriptor_count,
                                        ShaderResourceDescriptorBuffer {
                                            buffer: None, buffer_track_id: Default::default(),
                                        },
                                    );
                                    ShaderResourceDescriptors::Buffers(bufs)
                                } else if binding.descriptor_type.is_image() {
                                    let mut imgs = NonNullVec32::with_capacity(
                                        binding.descriptor_count,
                                        &alloc
                                    )?.into_static();
                                    imgs.resize(
                                        binding.descriptor_count,
                                        ShaderResourceDescriptorImage {
                                            sampler: None,
                                            image: None, image_track_id: Default::default(),
                                        },
                                    );
                                    ShaderResourceDescriptors::Images(imgs)
                                } else {
                                    return Err(Error::just_context(format_compact!(
                                        "invalid descriptor type {}", binding.descriptor_type
                                    )))
                                };
                            Ok(ShaderResourceBinding {
                                binding: binding.binding,
                                descriptor_type: binding.descriptor_type,
                                descriptors,
                            })
                        }
                    } else {
                        Err(Error::just_context(format_compact!(
                            "no space allocated for descriptor type {}",
                            binding.descriptor_type,
                        )))
                    }
                })
            )?;
            set_layouts.push(layout.handle);
            resources.push(ShaderResource::new(
                bindings, layout.shader_stage_mask,
                alloc.into_inner(),
            ));
        }
        let info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: inner.handle,
            descriptor_set_count: count,
            p_set_layouts: set_layouts.as_ptr(),
            ..Default::default()
        };
        let mut sets = FixedVec32
            ::with_capacity(count, tmp_alloc)?;
        let device = inner.vk.device();
        let res = unsafe {
            (device.fp_v1_0().allocate_descriptor_sets)(device.handle(), &info, sets.as_mut_ptr())
        };
        if res != vk::Result::SUCCESS {
            return Err(Error::new(res, "failed to allocate descriptor sets"))
        }
        unsafe {
            sets.set_len(count);
        }
        inner.allocated_sets += count;
        for (i, mut resource) in resources.into_iter().enumerate() {
            let set  = sets[i];
            resource.descriptor_set = set;
            let index = inner.shader_resources.insert(resource);
            if index.index() >= inner.descriptor_cache.len() {
                inner.descriptor_cache.resize(index.index() + 1, None);
            }
            inner.descriptor_cache[index.index() as usize] = Some(NonZeroU64::new(
                set.as_raw(),
            ).unwrap());
            collect(i, ShaderResourceId(pool_id, ShaderResourceInnerId(index)))
        }
        Ok(())
    }

    pub unsafe fn free<Alloc>(
        &self,
        resources: &Resources,
        queue_scheduler: &QueueSchedulerReadLock,
        ids: &[ShaderResourceId],
        tmp_alloc: &Alloc,
    ) -> Result<()>
        where 
            Alloc: LocalAlloc<Error = Error>,
    {
        let mut inner = self.inner.write();
        let mut descriptor_sets = FixedVec32
            ::with_capacity(ids.len() as u32, tmp_alloc)?;
        let finsihed_frame = resources
            .get_semaphore_counter_value(queue_scheduler.get_frame_semaphore_id())?;
        for &id in ids {
            if let Ok(resource) = inner.shader_resources.remove(id.1.0) {
                if resource.last_used_frame > finsihed_frame {
                    return Err(Error::just_context(
                        "attempting to free shader resource {id} while it is still in use by a queue"
                    ))
                }
                for binding in resource.bindings.iter() {
                    let descriptor_buffers = binding.descriptor_buffers();
                    for descriptor in descriptor_buffers {
                        if let Some(pool) = inner.pools.get_mut(&binding.descriptor_type) {
                            pool.used -= descriptor_buffers.len() as u32;
                        }
                        if let Some((buffer_id, _, _)) = descriptor.buffer {
                            inner.untrack_buffer(buffer_id, descriptor.buffer_track_id)?;
                        }
                    }
                    let descriptor_images = binding.descriptor_images();
                    for descriptor in descriptor_images {
                        if let Some((image_id, _, _)) = descriptor.image {
                            inner.untrack_image(image_id, descriptor.image_track_id)?;
                        }
                    }
                }
                let index = id.1.0.index();
                if inner.descriptor_cache.len() > index {
                    inner.descriptor_cache[index as usize] = None;
                }
                descriptor_sets.push(resource.descriptor_set);
            }
        }
        unsafe {
            inner.vk.device().free_descriptor_sets(
                inner.handle,
                &descriptor_sets,
            ).context("failed to free descriptor sets")?;
        }
        inner.allocated_sets -= descriptor_sets.len() as u32;
        Ok(())
    }

    pub fn update(
        &self,
        resources: &Resources,
    ) {
        self.inner.write().update(resources);
    }

    #[inline(always)]
    pub fn get_descriptor_set(
        &self,
        id: ShaderResourceInnerId,
    ) -> Result<vk::DescriptorSet>
    {
        let inner = self.inner.read();
        let Some(Some(handle)) = inner.descriptor_cache.get(id.0.index() as usize) else {
            return Err(Error::just_context(format_compact!(
                "invalid shader resource id {}", id,
            )));
        };
        Ok(<_ as vk::Handle>::from_raw(handle.get()))
    }

    #[inline(always)]
    pub fn write(&self) -> ShaderResourcePoolWriteGuard<'_> {
        ShaderResourcePoolWriteGuard {
            inner: self.inner.write(),
        }
    }
}

impl Inner {

    #[inline(always)]
    fn add_pending_validation(&mut self, id: ShaderResourceInnerId) {
        self.pending_validations.insert(id);
    }

    #[inline(always)]
    fn track_buffer(
        &mut self,
        buffer_id: BufferId,
        resource_id: ShaderResourceInnerId,
    ) -> TrackedShaderResourceId<BufferMeta> {
        TrackedShaderResourceId(
            self.tracked_buffers.entry(buffer_id)
                .or_default()
                .insert(resource_id),
            PhantomData,
        )
    }

    #[inline(always)]
    fn untrack_buffer(
        &mut self,
        buffer_id: BufferId,
        tracked_id: TrackedShaderResourceId<BufferMeta>
    ) -> Result<()>
    {
        let Some(tracked) = self.tracked_buffers
            .get_mut(&buffer_id) else
        {
            return Ok(())
        };
        tracked
            .remove(tracked_id.0)
            .context("failed to untrack buffer")?;
        Ok(())
    } 

    #[inline(always)]
    fn track_image(
        &mut self,
        image_id: ImageId,
        resource_id: ShaderResourceInnerId,
    ) -> TrackedShaderResourceId<Image> {
        TrackedShaderResourceId(
            self.tracked_images.entry(image_id)
                .or_default()
                .insert(resource_id),
            PhantomData,
        )
    }

    #[inline(always)]
    fn untrack_image(
        &mut self,
        image_id: ImageId,
        tracked_id: TrackedShaderResourceId<Image>
    ) -> Result<()>
    {
        let Some(tracked) = self.tracked_images
            .get_mut(&image_id) else {
            return Ok(())
        };
        tracked
            .remove(tracked_id.0)
            .context("failed to untrack image")?;
        Ok(())
    }
}

pub(crate) struct ShaderResourceHandle(*mut ShaderResource, ShaderResourceInnerId);

impl ShaderResourceHandle {

    #[inline(always)]
    pub unsafe fn get(&mut self) -> (&mut ShaderResource, ShaderResourceInnerId) {
        unsafe {
            (self.0.as_mut().unwrap(), self.1)
        }
    }

    #[inline(always)]
    pub unsafe fn unpoison(&mut self) {
        unsafe {
            self.get().0
        }.flags &= !ShaderResource::MAYBE_POISONED;
    }
}

impl<'a> ShaderResourcePoolWriteGuard<'a> {

    #[inline(always)]
    pub fn get_shader_resource_for_update(
        &mut self,
        id: ShaderResourceId,
        finished_frame: u64,
    ) -> Result<ShaderResourceUpdateContext<'_, 'a>>
    {
        if self.inner.poisoned {
            return Err(Error::just_context("pool poisoned"))
        }
        let inner_id = id.inner_id();
        let resource = self.inner.shader_resources
            .get_mut(inner_id.0)
            .context("failed to find shader resource")?;
        if resource.last_used_frame > finished_frame {
            return Err(Error::just_context(format_compact!(
                "attempting to update shader resource {id} while it is still in use by a queue"
            )))
        }
        Ok(ShaderResourceUpdateContext {
            handle: ShaderResourceHandle(resource as *mut _, inner_id),
            pool: self,
        })
    }

    #[inline(always)]
    pub fn get_shader_resource_for_submit(
        &mut self,
        id: ShaderResourceId,
        current_frame: u64,
    ) -> Result<ShaderResourceSubmitContext<'_, 'a>> {
        let inner_id = id.inner_id();
        let mut handle = self.get_shader_resource_handle(inner_id)?;
        unsafe {
            let resource = handle.get().0;
            if !resource.is_valid() {
                return Err(Error::just_context(format_compact!(
                    "resource {id} is in an invalid state",
                )))
            }
            resource.last_used_frame = current_frame;
        }
        Ok(ShaderResourceSubmitContext {
            pool: self,
            handle 
        })
    }

    #[inline(always)]
    pub fn get_shader_resource_handle(
        &mut self,
        inner_id: ShaderResourceInnerId,
    ) -> Result<ShaderResourceHandle>
    {
        if self.inner.poisoned {
            return Err(Error::just_context("pool poisoned"))
        }
        let resource = self.inner.shader_resources
            .get_mut(inner_id.0)
            .context("failed to find shader resource")?;
        Ok(ShaderResourceHandle(resource as *mut _, inner_id))
    }
}

impl<'a> Deref for ShaderResourcePoolWriteGuard<'a> {

    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> DerefMut for ShaderResourcePoolWriteGuard<'a> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for Inner {

    fn drop(&mut self) {
        unsafe {
            self.vk.device().destroy_descriptor_pool(
                self.handle, None,
            );
        }
    }
}

pub(crate) struct ShaderResourceUpdateContext<'a, 'b> {
    pool: &'a mut ShaderResourcePoolWriteGuard<'b>,
    handle: ShaderResourceHandle,
}

impl<'a, 'b> ShaderResourceUpdateContext<'a, 'b> {

    #[inline(always)]
    pub fn descriptor_set(&mut self) -> vk::DescriptorSet {
        unsafe {
            self.handle.get().0
        }.descriptor_set
    }

    #[inline(always)]
    pub fn into_inner(self) -> ShaderResourceHandle {
        self.handle
    }

    #[inline(always)]
    pub fn update_buffer<'c, Alloc>(
        &mut self,
        resources: &Resources,
        update: &ShaderResourceBufferUpdate,
        alloc: &'c Alloc,
    ) -> Result<(vk::DescriptorType, FixedVec32<'c, vk::DescriptorBufferInfo, Alloc>)>
        where 
            Alloc: LocalAlloc<Error = Error>,
    {
        let mut pool = RaiiHandle::new(&mut *self.pool, |pool| {
            pool.poisoned = true;
        });
        let (resource, id) = unsafe {
            self.handle.get()
        };
        pool.add_pending_validation(id);
        resource.flags |= ShaderResource::MAYBE_POISONED;
        let Some(binding) = resource.bindings.iter_mut().find(|b| b.binding == update.binding) else {
            return Err(Error::just_context(format_compact!(
                "invalid buffer update binding {} for shader resource {}",
                update.binding, update.resource_id,
            )))
        };
        let count = binding.descriptor_buffers().len() as u32;
        if update.infos.len() as u32 + update.starting_index > count {
            return Err(Error::just_context(format_compact!(
                "buffer update descriptor starting index {} + count {} is out of range of shader resource {:?} descriptor count {}",
                update.starting_index, update.infos.len(), update.resource_id, count,
            )))
        }
        let ty = binding.descriptor_type;
        let Some(usage) = ty.buffer_usage() else {
            return Err(Error::just_context(format_compact!(
                "non-buffer descriptor on buffer update (binding {} for shader resource {})",
                update.binding, update.resource_id,
            )))
        };
        let mut vk_infos = FixedVec32
            ::with_capacity(update.infos.len() as u32, alloc)?;
        let starting_idx = update.starting_index as usize;
        for (i, descriptor) in binding.descriptor_buffers_mut()[starting_idx..starting_idx + update.infos.len()]
            .iter_mut()
            .enumerate()
        {
            let info = update.infos[i];
            if let Some((id, _, _)) = descriptor.buffer {
                pool.untrack_buffer(id, descriptor.buffer_track_id)?;
            }
            let buffers = resources.buffers.read();
            let buffer = buffers
                .get(info.buffer_id.0)
                .context_with(|| format_compact!(
                    "invalid buffer id {} for shader resource {} update (binding {}, index {}, type {ty})",
                    info.buffer_id, update.resource_id, update.binding, starting_idx + i,
                ))?;
            if let Some(err) = buffer.validate_usage(usage) {
                return Err(Error::new(err, format_compact!(
                    "buffer {} usage mismatch for shader resource {}, (binding {}, index {}, type {ty})",
                    info.buffer_id, update.resource_id, update.binding, starting_idx + i,
                )))
            }
            let properties = buffer.properties();
            if info.offset + info.size > properties.size
            {
                return Err(Error::new(BufferError::OutOfRange {
                    buffer_size: properties.size,
                    requested_offset: info.offset,
                    requested_size: info.size,
                }, format_compact!(
                    "failed to validate buffer {} range for shader resource {} update (binding {}, index {}, type {ty})",
                    info.buffer_id, update.resource_id, update.binding, starting_idx + i,
                )))
            }
            descriptor.buffer = Some((info.buffer_id, info.offset, info.size));
            descriptor.buffer_track_id = pool.track_buffer(info.buffer_id, id);
            let vk_info = vk::DescriptorBufferInfo {
                buffer: buffer.handle(),
                offset: info.offset,
                range: info.size,
            };  
            vk_infos.push(vk_info);
        }
        pool.into_inner();
        Ok((ty.into(), vk_infos))
    }

    #[inline(always)]
    pub fn update_image<'c, Alloc>(
        &mut self,
        resources: &Resources,
        update: &ShaderResourceImageUpdate,
        alloc: &'c Alloc,
    ) -> Result<(vk::DescriptorType, FixedVec32<'c, vk::DescriptorImageInfo, Alloc>)>
        where 
            Alloc: LocalAlloc<Error = Error>,
    {
        let mut pool = RaiiHandle::new(&mut *self.pool, |pool| {
            pool.poisoned = true;
        });
        let (resource, id) = unsafe {
            self.handle.get()
        };
        pool.add_pending_validation(id);
        resource.flags |= ShaderResource::MAYBE_POISONED;
        let Some(binding) = resource.bindings.iter_mut().find(|b| b.binding == update.binding) else {
            return Err(Error::just_context(format_compact!(
                "invalid image update binding {} for shader resource {:?}",
                update.binding, update.resource_id,
            )))
        };
        let ty = binding.descriptor_type();
        let Some(image_usage) = ty.image_usage() else {
            return Err(Error::just_context(format_compact!(
                "non-image descriptor on image update (binding {} for shader resource {})",
                update.binding, update.resource_id,
            )))
        };
        let image_layout =
            if ty == DescriptorType::StorageImage {
                ShaderImageLayout::General
            } else {
                ShaderImageLayout::SampledReadOnly
            };
        let count = binding.descriptor_images().len() as u32;
        if update.infos.len() as u32 + update.starting_index > count {
            return Err(Error::just_context(format_compact!(
                "image update descriptor starting index {} + count {} is out of range of shader resource {:?} descriptor count {}",
                update.starting_index, update.infos.len(), update.resource_id, count,
            )))
        }
        let mut vk_infos = FixedVec32
            ::with_capacity(update.infos.len() as u32, alloc)?;
        let starting_idx = update.starting_index as usize;
        for (i, descriptor) in binding.descriptor_images_mut()[starting_idx..starting_idx + update.infos.len()]
            .iter_mut()
            .enumerate()
        {
            let info = update.infos[i].clone();
            let sampler =
                if ty.requires_sampler() {
                    let Some(sampler) = info.sampler else {
                        return Err(Error::just_context(format_compact!(
                            "shader resource {} (binding {}, index {}, type {ty}) requires a sampler, but none was given",
                            update.resource_id, update.binding, starting_idx + i,
                        )))
                    };
                    descriptor.sampler = Some(sampler.clone());
                    sampler.handle().into_inner()
                } else {
                    if info.sampler.is_some() {
                        log::warn!(
                            "shader resource {} (binding {}, type {ty}) doesn't require a sampler, but a sampler wasn't none",
                            update.resource_id, update.binding,
                        );
                    }
                    vk::Sampler::null()
                };
            let image_view =
                if ty.requires_image() {
                    if let Some((image_id, _, _)) = descriptor.image {
                        pool.untrack_image(image_id, descriptor.image_track_id)?;
                    }
                    let Some((image_id, range_info)) = info.image_source else {
                        return Err(Error::just_context(format_compact!(
                            "shader resource {} (binding {}, type {ty}) requires an image, but none was given",
                            update.resource_id, update.binding,
                        )))
                    };
                    if let Some(range_info) = range_info {
                        let mut images = resources.images.write();
                        let image = images
                            .get_mut(image_id.0)
                            .context_with(|| format_compact!(
                                "invalid image id {} for shader resource {} update (binding {}, index {}, type {ty})",
                                image_id, update.resource_id, update.binding, starting_idx + i,
                            ))?;
                        if let Some(err) = image.validate_usage(image_usage) {
                            return Err(Error::new(err, format_compact!(
                                "image {} usage mismatch for shader resource {} update (binding {}, index {}, type {ty})",
                                image_id, update.resource_id, update.binding, starting_idx + i,
                            )))
                        }
                        let view = image
                            .get_subview(range_info)
                            .context_with(|| format_compact!(
                                "failed to get image {} subview for shader resource {}, (binding {}, index {}, type {ty})",
                                image_id, update.resource_id, update.binding, starting_idx + i,
                            ))?;
                        descriptor.image = Some((image_id, image_layout, Some(range_info.subresource)));
                        descriptor.image_track_id = pool.track_image(image_id, id);
                        view
                    }
                    else {
                        let images = resources.images.read();
                        let image = images
                            .get(image_id.0)
                            .context_with(|| format_compact!(
                                "invalid image id {} for shader resource {} (binding {}, index {}, type {ty})",
                                image_id, update.resource_id, update.binding, starting_idx + i,
                            ))?;
                        if let Some(err) = image.validate_usage(image_usage) {
                            return Err(Error::new(err, format_compact!(
                                "image {} usage mismatch for shader resource {}, (binding {}, index {}, type {ty})",
                                image_id, update.resource_id, update.binding, starting_idx + i,
                            )))
                        }
                        let view = image.get_view();
                        descriptor.image = Some((image_id, image_layout, None));
                        descriptor.image_track_id = pool.track_image(image_id, id);
                        view
                    }
                } else {
                    vk::ImageView::null()
                };
            let vk_info = vk::DescriptorImageInfo {
                sampler,
                image_view,
                image_layout: image_layout.into(),
            };
            vk_infos.push(vk_info);
        }
        pool.into_inner();
        Ok((ty.into(), vk_infos))
    }

    #[inline(always)]
    pub unsafe fn copy_from<'c>(
        &mut self,
        mut from: ShaderResourceHandle,
        src_binding: u32,
        src_starting_index: u32,
        dst_binding: u32,
        dst_starting_index: u32,
        array_count: u32,
    ) -> Result<vk::CopyDescriptorSet<'c>>
    {
        let mut pool = RaiiHandle::new(&mut *self.pool, |pool| {
            pool.poisoned = true;
        });
        let (from, from_id) = unsafe {
            from.get()
        };
        let (to, to_id) = unsafe {
            self.handle.get()
        };
        pool.add_pending_validation(to_id);
        to.flags |= ShaderResource::MAYBE_POISONED;
        let src_binding_id = src_binding;
        let src_binding = from.bindings.iter().find(|b| b.binding == src_binding_id)
            .ok_or_else(|| Error::just_context(format_compact!(
                "invalid source binding {}", src_binding,
            )))?;
        let dst_binding_id = dst_binding;
        let dst_binding = to.bindings.iter_mut().find(|b| b.binding == dst_binding)
            .ok_or_else(|| Error::just_context(format_compact!(
                "invalid destination binding {}", dst_binding,
            )))?;
        let src_ty = src_binding.descriptor_type();
        let dst_ty = dst_binding.descriptor_type();
        if src_ty != dst_ty {
            return Err(Error::just_context(format_compact!(
                "descriptor type mismatch, source type was {src_ty} while destination type {dst_ty}",
            )))
        }
        if to_id == from_id &&
            src_starting_index + array_count > dst_starting_index &&
            dst_starting_index + array_count > src_starting_index
        {
            return Err(Error::just_context(format_compact!(
                "shader resource copy source and destination (id: {from_id}) overlap, if source and destination shader resources are equal, the source and destination ranges must not overlap",
            )))?;
        }
        let ty = src_ty;
        if ty.is_buffer() {
            let src_descriptors = src_binding.descriptor_buffers();
            let dst_descriptors = dst_binding.descriptor_buffers_mut();
            if src_starting_index + array_count > src_descriptors.len() as u32 {
                return Err(Error::just_context(format_compact!(
                    "shader resource copy source starting index {} + array count {} is out of range of source count {}",
                    src_starting_index, array_count, src_descriptors.len(),
                )))
            }
            if dst_starting_index + array_count > dst_descriptors.len() as u32 {
                return Err(Error::just_context(format_compact!(
                    "shader resource copy destination starting index {} + array count {} is out of range of destination count {}",
                    dst_starting_index, array_count, dst_descriptors.len(),
                )))
            }
            let src_starting_index = src_starting_index as usize;
            for (i, dst_descriptor) in dst_descriptors[dst_starting_index as usize..(dst_starting_index + array_count) as usize]
                .iter_mut()
                .enumerate()
            {
                if let Some((id, _, _)) = dst_descriptor.buffer {
                    pool.untrack_buffer(id, dst_descriptor.buffer_track_id)?;
                }
                let new_buffer = src_descriptors[src_starting_index + i].buffer
                    .ok_or_else(|| Error::just_context(format_compact!(
                        "source buffer shader resource (index: {}) had uninitialized values",
                        src_starting_index + i,
                    )))?;
                dst_descriptor.buffer = Some(new_buffer);
                dst_descriptor.buffer_track_id = pool.track_buffer(new_buffer.0, to_id);
            }
        } else if ty.is_image() {
            let src_descriptors = src_binding.descriptor_images();
            let dst_descriptors = dst_binding.descriptor_images_mut();
            if src_starting_index + array_count > src_descriptors.len() as u32 {
                return Err(Error::just_context(format_compact!(
                    "shader resource copy source starting index {} + array count {} is out of range of source count {}",
                    src_starting_index, array_count, src_descriptors.len(),
                )))
            }
            if dst_starting_index + array_count > dst_descriptors.len() as u32 {
                return Err(Error::just_context(format_compact!(
                    "shader resource copy destination starting index {} + array count {} is out of range of destination count {}",
                    dst_starting_index, array_count, dst_descriptors.len(),
                )))
            }
            let src_starting_index = src_starting_index as usize;
            for (i, dst_descriptor) in dst_descriptors[dst_starting_index as usize..(dst_starting_index + array_count) as usize]
                .iter_mut()
                .enumerate()
            {
                if let Some((image_id, _, _)) = dst_descriptor.image {
                    pool.untrack_image(image_id, dst_descriptor.image_track_id)?;
                }
                let src_descriptor = src_descriptors[src_starting_index + i].clone();
                if let Some((image_id, _, _)) = src_descriptor.image {
                    dst_descriptor.image_track_id = pool.track_image(image_id, to_id);
                }
                dst_descriptor.sampler = src_descriptor.sampler.clone();
                dst_descriptor.image = src_descriptor.image;
            }
        }
        pool.into_inner();
        Ok(vk::CopyDescriptorSet {
            s_type: vk::StructureType::COPY_DESCRIPTOR_SET,
            src_set: from.descriptor_set,
            src_binding: src_binding_id,
            src_array_element: src_starting_index,
            dst_set: to.descriptor_set,
            dst_binding: dst_binding_id,
            dst_array_element: dst_starting_index,
            descriptor_count: array_count,
            ..Default::default()
        })
    }
}

pub(crate) struct ShaderResourceSubmitContext<'a, 'b> {
    pool: &'a mut ShaderResourcePoolWriteGuard<'b>,
    handle: ShaderResourceHandle,
}

impl<'a, 'b> ShaderResourceSubmitContext<'a, 'b> {

    #[inline(always)]
    pub fn binding_iter(&mut self) -> impl Iterator<Item = &'_ ShaderResourceBinding> {
        unsafe {
            self.handle.get()
        }.0.bindings.iter()
    }

    #[inline(always)]
    pub fn shader_stage_mask(&mut self) -> vk::ShaderStageFlags {
        unsafe {
            self.handle.get()
        }.0.shader_stage_mask
    }
}
