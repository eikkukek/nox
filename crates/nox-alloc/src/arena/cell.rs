use std::rc::Rc;
use core::cell::UnsafeCell;

use super::{
    Inner,
    ArenaGuard,
    ImmutArena, MutArena,
    Arena,
    prelude::{
        NonNull,
        Deref, DerefMut,
        Conditional, True, False,
        Error, Result,
        LocalAlloc, Layout,
    },
};

pub struct Ref<'a, F: Conditional = False> {
    inner: &'a mut (Inner<F>, bool),
}

impl<'a, F: Conditional> Deref for Ref<'a, F> {

    type Target = Inner<F>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner.0
    }
}

impl<'a, F: Conditional> DerefMut for Ref<'a, F> {

    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.0
    }
}

impl<'a, F: Conditional> Drop for Ref<'a, F> {
    
    #[inline(always)]
    fn drop(&mut self) {
        self.inner.1 = false;
    }
}

/// An arena allocator for single-thread use.
pub struct CellArena<AllowFallback = False>
    where AllowFallback: Conditional,
{
    inner: UnsafeCell<(Inner<AllowFallback>, bool)>,
}

impl CellArena<False> {

    /// Creates a new [`CellArena`] with `size`.
    pub fn new(size: usize) -> Result<Self> {
        Ok(Self {
            inner: UnsafeCell::new((Inner::new(size)?, false)),
        })
    }
}

impl CellArena<True> {

    /// Creates a new [`CellArena`] with `size`.
    ///
    /// This arena has a [`GlobalAlloc`] fallback in case it runs out of space.
    pub fn with_fallback(size: usize) -> Result<Self> {
        Ok(Self {
            inner: UnsafeCell::new((Inner::new(size)?, false)),
        })
    }
}

impl<F> CellArena<F>
    where F: Conditional,
{
    
    #[inline(always)]
    pub fn size(&self) -> usize {
        unsafe { &*self.inner.get() }.0.size
    }

    #[inline(always)]
    pub fn used(&self) -> usize {
        unsafe { &*self.inner.get() }.0.pos
    }

    #[inline(always)]
    pub fn remaining(&self) -> usize {
        let inner = unsafe { &*self.inner.get() };
        inner.0.size - inner.0.pos
    }

    #[inline(always)]
    /// Resets the pointer position to 0.
    /// # Safety
    /// Any allocations still in use will become invalid.
    pub unsafe fn clear(&self) -> Result<()> {
        let inner = unsafe { &mut *self.inner.get() };
        if inner.1 {
            Err(Error::just_context("attempting to clear arena while guard is active"))
        } else {
            inner.0.pos = 0;
            Ok(())
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> Inner<F> {
        self.inner.into_inner().0
    }
}

impl<F> ImmutArena for CellArena<F>
    where
        F: Conditional,
{}

impl<F> ImmutArena for Rc<CellArena<F>>
    where
        F: Conditional,
{}

impl<F> MutArena for CellArena<F>
    where
        F: Conditional,
{}

impl<HasFallback: Conditional> Arena<HasFallback> for CellArena<HasFallback> {

    type Guard<'a> = Ref<'a, HasFallback>
        where Self: 'a;

    #[inline(always)]
    fn guard(&self) -> ArenaGuard<Self::Guard<'_>, HasFallback>
        where
            for<'a> ArenaGuard<
                Self::Guard<'a>,
                HasFallback
            >: LocalAlloc<Error = Error>
    {
        self.try_guard().unwrap()
    }

    #[inline(always)]
    fn try_guard(&self) -> Result<ArenaGuard<Self::Guard<'_>, HasFallback>>
        where
            for<'a> ArenaGuard<
                Self::Guard<'a>,
                HasFallback
            >: LocalAlloc<Error = Error>
    {
        let inner = unsafe { &mut *self.inner.get() };
        if inner.1 {
            Err(Error::just_context(
                "attempting to create a new arena guard while another guard is active"
            ))
        } else {
            Ok(ArenaGuard::new(Ref { inner }))
        }
    }

    #[inline(always)]
    fn guard_mut(&mut self) -> ArenaGuard<&mut Inner<HasFallback>, HasFallback>
        where
            for<'a> ArenaGuard<
                &'a mut Inner<HasFallback>,
                HasFallback
            >: LocalAlloc<Error = Error>
    {
        ArenaGuard::new(&mut self.inner.get_mut().0)
    }
}

impl<HasFallback: Conditional> Arena<HasFallback> for
    Rc<CellArena<HasFallback>>
{

    type Guard<'a> = Ref<'a, HasFallback>
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
        self.deref().try_guard()
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

unsafe impl LocalAlloc for CellArena<False> {

    type Error = Error;

    #[inline(always)]
    unsafe fn allocate_raw(&self, layout: Layout) -> Result<NonNull<u8>> {
        let inner = unsafe { &mut *self.inner.get() };
        if inner.1 {
            return Err(Error::just_context(
                "attempting to allocate directly with an arena that has an active guard"
            ))
        }
        unsafe {
            inner.0.allocate_raw_internal(layout)
        }
    }

    #[inline(always)]
    unsafe fn free_raw(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

unsafe impl LocalAlloc for CellArena<True> {

    type Error = Error;

    #[inline(always)]
    unsafe fn allocate_raw(&self, layout: Layout) -> Result<NonNull<u8>> {
        let inner = unsafe { &mut *self.inner.get() };
        if inner.1 {
            return Err(Error::just_context(
                "attempting to allocate directly with an arena that has an active guard"
            ))
        }
        unsafe {
            inner.0.allocate_raw_internal(layout)
        }
    }

    #[inline(always)]
    unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            (&mut *self.inner.get()).0.free_raw_internal(ptr, layout);
        }
    }
}
