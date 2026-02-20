use std::sync::Arc;

use super::{
    Inner, ArenaGuard, Arena, ImmutArena, MutArena,
    prelude::{
        NonNull,
        Conditional, True, False,
        LocalAlloc, Layout,
        Deref, DerefMut,
        Result, Error,
    },
};

use parking_lot::{RwLock, RwLockWriteGuard};

pub struct Guard<'a, F: Conditional = False> {
    inner: RwLockWriteGuard<'a, Inner<F>>,
}

impl<'a, F: Conditional> Deref for Guard<'a, F> {

    type Target = Inner<F>;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, F: Conditional> DerefMut for Guard<'a, F> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

/// An arena allocator for multi-threaded use.
pub struct RwArena<AllowFallback = False>
    where AllowFallback: Conditional,
{
    inner: RwLock<Inner<AllowFallback>>,
}

impl RwArena<False> {

    /// Creates a new [`RwArena`] with `size`.
    pub fn new(size: usize) -> Result<Self> {
        Ok(Self {
            inner: RwLock::new(Inner::new(size)?),
        })
    }
}

impl RwArena<True> {

    /// Creates a new [`RwArena`] with `size`.
    ///
    /// This arena has a [`GlobalAlloc`] fallback in case it runs out of space.
    pub fn with_fallback(size: usize) -> Result<Self> {
        Ok(Self {
            inner: RwLock::new(Inner::new(size)?),
        })
    }
}

impl<F> RwArena<F>
    where
        F: Conditional,
        for<'a> ArenaGuard<Guard<'a, F>, F>: LocalAlloc<Error = Error>,
{
    
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.inner.read().size
    }

    #[inline(always)]
    pub fn used(&self) -> usize {
        self.inner.read().pos
    }

    #[inline(always)]
    pub fn remaining(&self) -> usize {
        let inner = self.inner.read();
        inner.size - inner.pos
    }

    /// Resets the pointer position to 0.
    /// # Safety
    /// Any allocations still in use will may become invalid.
    #[inline(always)]
    pub unsafe fn clear(&self) {
        let mut inner = self.inner.write();
        inner.pos = 0;
    }

    #[inline(always)]
    pub fn into_inner(self) -> Inner<F> {
        self.inner.into_inner()
    }
}

impl<F> ImmutArena for RwArena<F>
    where
        F: Conditional,
{}

impl<F> ImmutArena for Arc<RwArena<F>>
    where
        F: Conditional,
{}

impl<F> MutArena for RwArena<F>
    where
        F: Conditional,
{}

impl<HasFallback: Conditional> Arena<HasFallback> for RwArena<HasFallback>
{

    type Guard<'a> = Guard<'a, HasFallback>
        where Self: 'a;

    #[inline(always)]
    fn guard(&self) -> ArenaGuard<Self::Guard<'_>, HasFallback>
        where
            for<'a> ArenaGuard<
                Self::Guard<'a>,
                HasFallback
            >: LocalAlloc<Error = Error>
    {
        ArenaGuard::new(Guard { inner: self.inner.write() })
    }

    #[inline(always)]
    fn try_guard(&self) -> Result<ArenaGuard<Self::Guard<'_>, HasFallback>>
        where
            for<'a> ArenaGuard<
                Self::Guard<'a>,
                HasFallback
            >: LocalAlloc<Error = Error>
    {
        Ok(self.guard())
    }

    #[inline(always)]
    fn guard_mut(&mut self) -> ArenaGuard<&mut Inner<HasFallback>, HasFallback>
        where
            for<'a> ArenaGuard<
                &'a mut Inner<HasFallback>,
                HasFallback
            >: LocalAlloc<Error = Error>
    {
        ArenaGuard::new(self.inner.get_mut())
    }
}

impl<HasFallback: Conditional> Arena<HasFallback> for
    Arc<RwArena<HasFallback>>
{

    type Guard<'a> = Guard<'a, HasFallback>
        where Self: 'a;

    #[inline(always)]
    fn guard(&self) -> ArenaGuard<Self::Guard<'_>, HasFallback>
        where
            for<'a> ArenaGuard<
                Self::Guard<'a>,
                HasFallback
            >: LocalAlloc<Error = Error>
    {
        self.deref().guard()
    }

    #[inline(always)]
    fn try_guard(&self) -> Result<ArenaGuard<Self::Guard<'_>, HasFallback>>
        where
            for<'a> ArenaGuard<
                Self::Guard<'a>,
                HasFallback
            >: LocalAlloc<Error = Error>
    {
        Ok(self.guard())
    }

    #[inline(always)]
    fn guard_mut(&mut self) -> ArenaGuard<&mut Inner<HasFallback>, HasFallback>
        where
            for<'a> ArenaGuard<
                &'a mut Inner<HasFallback>,
                HasFallback
            >: LocalAlloc<Error = Error>
    {
        unreachable!()
    }
}

unsafe impl LocalAlloc for RwArena<False> {

    type Error = Error;

    #[inline(always)]
    unsafe fn allocate_raw(&self, layout: Layout) -> Result<NonNull<u8>> {
        unsafe {
            self.inner.write().allocate_raw_internal(layout)
        }
    }

    #[inline(always)]
    unsafe fn free_raw(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

unsafe impl LocalAlloc for RwArena<True> {

    type Error = Error;

    #[inline(always)]
    unsafe fn allocate_raw(&self, layout: Layout) -> Result<NonNull<u8>> {
        unsafe {
            self.inner.write().allocate_raw_internal(layout)
        }
    }

    #[inline(always)]
    unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            self.inner.read().free_raw_internal(ptr, layout);
        }
    }
}
