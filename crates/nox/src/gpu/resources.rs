use core::{
    ops::{Deref, DerefMut, BitXor, BitAnd},
    fmt::{Debug, Display, self},
    marker::PhantomData,
    error,
};

use compact_str::format_compact;

use nox_proc::Display;

use nox_mem::slot_map::*;

use nox_ash::vk;

use crate::gpu::prelude::{
    memory_binder::MemoryBinder,
    *,
};

use crate::{
    error::*,
    sync::*,
};

pub trait ResourceMeta {

    const NAME: &str;
}

pub trait ResourceId<Meta>: Display + Copy
    where Meta: ResourceMeta
{

    fn slot_index(self) -> SlotIndex<Meta>;
}

impl<Meta> ResourceId<Meta> for SlotIndex<Meta>
    where Meta: ResourceMeta
{

    #[inline(always)]
    fn slot_index(self) -> SlotIndex<Meta> {
        self
    }
}

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display("{0}")]
pub struct SurfaceId(pub(crate) SlotIndex<Surface>);

impl ResourceId<Surface> for SurfaceId {
 
    #[inline(always)]
    fn slot_index(self) -> SlotIndex<Surface> {
        self.0
    }
}

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display("{0}")]
pub struct BufferId(pub(crate) SlotIndex<BufferMeta>);

impl BufferId {
    
    #[inline(always)]
    pub(super) fn new(index: SlotIndex<BufferMeta>) -> Self {
        Self(index)
    }
}

impl ResourceId<BufferMeta> for BufferId {

    #[inline(always)]
    fn slot_index(self) -> SlotIndex<BufferMeta> {
        self.0
    }
}

#[derive(Clone)]
pub(crate) struct ResourceGuard<
    Meta: ResourceMeta,
    Id: ResourceId<Meta>,
    Guard: Deref<Target = SlotMap<Meta>>,
> {
    guard: Guard,
    _marker: PhantomData<Id>,
}

impl<Meta, Id, Guard> ResourceGuard<Meta, Id, Guard>
    where
        Meta: ResourceMeta,
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
                "invalid {} id {id}",
                Meta::NAME
            ))
    }

    #[inline(always)]
    pub fn get_mut(&mut self, id: Id) -> Result<&mut Meta>
        where Guard: DerefMut
    {
        self.guard
            .get_mut(id.slot_index())
            .context_with(|| format_compact!(
                "invalid {} id {id}",
                Meta::NAME
            ))
    }
}

impl<Meta, Id, Guard> Deref for ResourceGuard<Meta, Id, Guard>
    where
        Id: ResourceId<Meta>,
        Meta: ResourceMeta,
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
        Meta: ResourceMeta,
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

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)] #[display("{0}")]
pub struct TimelineSemaphoreId(pub(super) SlotIndex<vk::Semaphore>);

#[derive(Clone)]
pub(crate) struct MemoryBinderResource {
    binder: Arc<RwLock<dyn MemoryBinder>>,
}

impl MemoryBinderResource {

    pub fn new<T: MemoryBinder>(binder: T) -> Self {
        Self {
            binder: Arc::new(RwLock::new(binder))
        }
    }

    #[inline(always)]
    pub fn write(&self) -> MemoryBinderResourceWriteGuard<'_> {
        MemoryBinderResourceWriteGuard { guard: self.binder.write() }
    }

    #[inline(always)]
    pub fn read(&self) -> MemoryBinderResourceReadGuard<'_> {
        MemoryBinderResourceReadGuard { guard: self.binder.read() }
    }
}

pub(crate) struct MemoryBinderResourceReadGuard<'a> {
    guard: RwLockReadGuard<'a, dyn MemoryBinder>,
}

impl Deref for MemoryBinderResourceReadGuard<'_> {

    type Target = dyn MemoryBinder;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

pub(crate) struct MemoryBinderResourceWriteGuard<'a> {
    guard: RwLockWriteGuard<'a, dyn MemoryBinder>,
}

impl Deref for MemoryBinderResourceWriteGuard<'_> {

    type Target = dyn MemoryBinder;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl DerefMut for MemoryBinderResourceWriteGuard<'_> {

    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)] #[display("{0}")]
pub struct MemoryBinderId(pub(super) SlotIndex<MemoryBinderResource>);

#[derive(Clone, Copy)]
pub(crate) enum ResourceBinderInner {
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

#[derive(Default, Clone, Copy)]
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

    #[inline(always)]
    pub(crate) fn into_inner(self) -> ResourceBinderInner {
        self.inner
    }
}

pub trait Flags: 
    Copy +
    BitXor<Output = Self> + BitAnd<Output = Self> +
    Display + Debug
{
    const NAME: &str;
}

#[derive(Debug)]
pub struct MissingFlagsError<T>
    where T: Flags
{
    missing: T,
}

impl<T> MissingFlagsError<T>
    where T: Flags,
{

    #[inline(always)]
    pub fn new(
        requested: T,
        has: T,
    ) -> Self {
        Self {
            missing: requested ^ has & requested,
        }
    }
}

impl<T> Display for MissingFlagsError<T>
    where T: Flags
{

    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missing {} {}", T::NAME, self.missing)
    }
}

impl<T> error::Error for MissingFlagsError<T> where T: Flags {}
