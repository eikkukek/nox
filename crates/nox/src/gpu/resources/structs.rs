use core::{
    marker::PhantomData,
    fmt::Display,
    ops::DerefMut,
};

use parking_lot::RwLockReadGuard;

use nox_proc::Display;

use crate::gpu::prelude::{
    memory_binder::MemoryBinder,
    *,
};

use super::*;

pub trait ResourceId<Meta>: Display + Copy {

    fn slot_index(self) -> SlotIndex<Meta>;

    fn resource_name() -> &'static str;
}

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display("{0}")]
pub struct BufferId(pub(super) SlotIndex<BufferMeta>);

impl ResourceId<BufferMeta> for BufferId {

    #[inline(always)]
    fn slot_index(self) -> SlotIndex<BufferMeta> {
        self.0
    }

    #[inline(always)]
    fn resource_name() -> &'static str {
        "buffer"
    }
}

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display("{0}")]
pub struct ImageId(pub(super) SlotIndex<ImageMeta>);

impl ResourceId<ImageMeta> for ImageId {

    #[inline(always)]
    fn slot_index(self) -> SlotIndex<ImageMeta> {
        self.0
    }

    #[inline(always)]
    fn resource_name() -> &'static str {
        "image"
    }
}

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct PipelineBatchId(pub(super) u32);

pub(crate) struct ResourceGuard<
    Meta,
    Id: ResourceId<Meta>,
    Guard: Deref<Target = SlotMap<Meta>>,
> {
    guard: Guard,
    _marker: PhantomData<Id>,
}

impl<Meta, Id, Guard> ResourceGuard<Meta, Id, Guard>
    where
        Id: ResourceId<Meta>,
        Guard: Deref<Target = SlotMap<Meta>>
{

    #[inline(always)]
    pub fn new(guard: Guard) -> Self {
        Self {
            guard,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn get(&self, id: Id) -> Result<&Meta> {
        self.guard
            .get(id.slot_index())
            .context_with(|| format_compact!(
                "failed to find {} with id {id}",
                Id::resource_name()
            ))
    }

    #[inline(always)]
    pub fn get_mut(&mut self, id: Id) -> Result<&mut Meta>
        where Guard: DerefMut
    {
        self.guard
            .get_mut(id.slot_index())
            .context_with(|| format_compact!(
                "failed to find {} with id {id}",
                Id::resource_name()
            ))
    }
}

impl<Meta, Id, Guard> Deref for ResourceGuard<Meta, Id, Guard>
    where
        Id: ResourceId<Meta>,
        Guard: Deref<Target = SlotMap<Meta>>
{

    type Target = SlotMap<Meta>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<Meta, Id, Guard> DerefMut for ResourceGuard<Meta, Id, Guard>
    where
        Id: ResourceId<Meta>,
        Guard: DerefMut<Target = SlotMap<Meta>>
{

    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

pub(crate) type ResourceReadGuard<'a, Meta, Id> =
    ResourceGuard<Meta, Id, RwLockReadGuard<'a, SlotMap<Meta>>>;

pub(crate) type ResourceWriteGuard<'a, Meta, Id> =
    ResourceGuard<Meta, Id, RwLockWriteGuard<'a, SlotMap<Meta>>>;

pub(crate) struct DynResourceReadGuard<
    'a,
    Meta,
    Id: ResourceId<Meta>,
> {
    guard: &'a dyn Deref<Target = SlotMap<Meta>>,
    _marker: PhantomData<Id>,
}

impl<'a, Meta, Id> DynResourceReadGuard<'a, Meta, Id>
    where
        Id: ResourceId<Meta>,
{

    #[inline(always)]
    pub fn new(guard: &'a dyn Deref<Target = SlotMap<Meta>>) -> Self {
        Self {
            guard,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn get(&self, id: Id) -> Result<&Meta> {
        self.guard
            .get(id.slot_index())
            .context_with(|| format_compact!(
                "failed to find {} with id {id}",
                Id::resource_name()
            ))
    }
}

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)] #[display("{0}")]
pub struct TimelineSemaphoreId(pub(super) SlotIndex<vk::Semaphore>);

#[repr(C)]
struct MemoryBinderResourceInner<T: ?Sized> {
    last_used_frame: u64,
    binder: T,
}

impl<T> MemoryBinderResourceInner<T> {

    fn new(t: T) -> Self {
        Self {
            last_used_frame: 0,
            binder: t,
        }
    }
}

#[derive(Clone)]
pub struct MemoryBinderResource {
    binder: Arc<RwLock<MemoryBinderResourceInner<dyn MemoryBinder>>>,
}

pub struct MemoryBinderResourceReadGuard<'a> {
    guard: RwLockReadGuard<'a, MemoryBinderResourceInner<dyn MemoryBinder>>,
}

impl MemoryBinderResourceReadGuard<'_> {

    #[inline(always)]
    pub(crate) fn get_last_used_frame(&self) -> u64 {
        self.guard.last_used_frame
    }
}

impl Deref for MemoryBinderResourceReadGuard<'_> {

    type Target = dyn MemoryBinder;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.guard.binder
    }
}

pub struct MemoryBinderResourceWriteGuard<'a> {
    guard: RwLockWriteGuard<'a, MemoryBinderResourceInner<dyn MemoryBinder>>,
}

impl MemoryBinderResourceWriteGuard<'_> {

    #[inline(always)]
    pub(crate) fn get_last_used_frame(&self) -> u64 {
        self.guard.last_used_frame
    }

    #[inline(always)]
    pub(crate) unsafe fn set_last_used_frame(&mut self, frame: u64)  {
        self.guard.last_used_frame = frame;
    }
}

impl Deref for MemoryBinderResourceWriteGuard<'_> {

    type Target = dyn MemoryBinder;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.guard.binder
    }
}

impl DerefMut for MemoryBinderResourceWriteGuard<'_> {

    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard.binder
    }
}

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)] #[display("{0}")]
pub struct MemoryBinderId(pub(super) SlotIndex<MemoryBinderResource>);

pub(super) enum ResourceBinderInner {
    Default,
    DefaultMappable,
    Id(MemoryBinderId),
}

impl Default for ResourceBinderInner {

    #[inline(always)]
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Default)]
pub struct ResourceBinder {
    inner: ResourceBinderInner,
}

impl ResourceBinder {

    #[inline(always)]
    pub fn new(id: MemoryBinderId) -> Self {
        Self {
            inner: ResourceBinderInner::Id(id),
        }
    }
    
    #[inline(always)]
    pub fn global() -> Self {
        <Self as Default>::default()
    }

    #[inline(always)]
    pub fn global_mappable() -> Self {
        Self {
            inner: ResourceBinderInner::DefaultMappable,
        }
    }
}
