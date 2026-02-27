use core::{
    ops::DerefMut,
    cell::UnsafeCell,
    ptr::NonNull,
};

use nox_mem::{
    conditional::{Conditional, True, False},
    alloc::{LocalAlloc, Layout},
};

use nox_error::{Result, Error};

use super::{Inner, Arena, MutArena};

/// An arena guard returned by [`Arena::guard`] methods.
///
/// This also implements [`Arena`], but nested guards can only be created via [`Arena::guard_mut`].
pub struct ArenaGuard<Alloc, F = False>
    where 
        Alloc: DerefMut<Target = Inner<F>>,
        F: Conditional,
        Self: LocalAlloc<Error = Error>,
{
    inner: UnsafeCell<Alloc>,
    pos_rollback: usize,
}

impl<Alloc, F> ArenaGuard<Alloc, F>
    where 
        Alloc: DerefMut<Target = Inner<F>>,
        F: Conditional,
        Self: LocalAlloc<Error = Error>
{

    #[inline(always)]
    pub(super) fn new(stack: Alloc) -> Self {
        Self {
            pos_rollback: stack.pos,
            inner: UnsafeCell::new(stack),
        }
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        unsafe {
            &*self.inner.get()
        }.size
    }

    /// Returns how many bytes has been allocated since the guard was created.
    #[inline(always)]
    pub fn used(&self) -> usize {
        unsafe {
            &*self.inner.get()
        }.pos - self.pos_rollback
    }

    #[inline(always)]
    pub fn remaining(&self) -> usize {
        let inner = unsafe {
            &*self.inner.get()
        };
        inner.size - inner.pos
    }

    /// Resets the pointer position to the guard position roll back.
    /// # Safety
    /// Any allocations still in use in the guard's region will become invalid.
    #[inline(always)]
    pub unsafe fn clear(&mut self) {
        self.inner.get_mut().pos = self.pos_rollback;
    }
}

impl<Alloc, HasFallback> MutArena for ArenaGuard<Alloc, HasFallback>
    where
        Alloc: DerefMut<Target = Inner<HasFallback>>,
        HasFallback: Conditional,
        Self: LocalAlloc<Error = Error>
{}

impl<Alloc> Arena<False> for ArenaGuard<Alloc, False>
    where
        Alloc: DerefMut<Target = Inner<False>>,
        Self: LocalAlloc<Error = Error>,
{

    type Guard<'a> = &'a mut Inner<False>
        where Self: 'a;

    fn guard(&self) -> ArenaGuard<Self::Guard<'_>, False>
    {
        unreachable!()
    }

    fn try_guard(&self) -> Result<ArenaGuard<Self::Guard<'_>, False>>
    {
        unreachable!()
    }

    #[inline(always)]
    fn guard_mut(&mut self) -> ArenaGuard<&mut Inner<False>, False> {
        ArenaGuard::<&mut Inner<False>, False>::new(
            self.inner.get_mut().deref_mut()
        )
    }
}

impl<Alloc> Arena<True> for ArenaGuard<Alloc, True>
    where
        Alloc: DerefMut<Target = Inner<True>>,
        Self: LocalAlloc<Error = Error>,
{

    type Guard<'a> = &'a mut Inner<True>
        where Self: 'a;

    fn guard(&self) -> ArenaGuard<Self::Guard<'_>, True>
    {
        unreachable!()
    }

    fn try_guard(&self) -> Result<ArenaGuard<Self::Guard<'_>, True>>
    {
        unreachable!()
    }

    #[inline(always)]
    fn guard_mut(&mut self) -> ArenaGuard<&mut Inner<True>, True> {
        ArenaGuard::<&mut Inner<True>, True>::new(
            self.inner.get_mut().deref_mut()
        )
    }
}

unsafe impl<Alloc> LocalAlloc for ArenaGuard<Alloc, False>
    where Alloc: DerefMut<Target = Inner>,
{

    type Error = Error;

    #[inline(always)]
    unsafe fn allocate_raw(&self, layout: Layout) -> Result<NonNull<u8>> {
        unsafe { (&mut *self.inner.get()).allocate_raw_internal(layout) }
    }

    #[inline(always)]
    unsafe fn free_raw(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

unsafe impl<Alloc> LocalAlloc for ArenaGuard<Alloc, True>
    where 
        Alloc: DerefMut<Target = Inner<True>>,
{

    type Error = Error;

    #[inline(always)]
    unsafe fn allocate_raw(&self, layout: Layout) -> Result<NonNull<u8>> {
        unsafe { (&mut *self.inner.get()).allocate_raw_internal(layout) }
    }

    #[inline(always)]
    unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { (&*self.inner.get()).free_raw_internal(ptr, layout); }
    }
}

impl<Alloc, F> Drop for ArenaGuard<Alloc, F>
    where
        Alloc: DerefMut<Target = Inner<F>>,
        F: Conditional,
        Self: LocalAlloc<Error = Error>,
{

    fn drop(&mut self) {
        self.inner.get_mut().pos = self.pos_rollback;
    }
}

pub type ArenaGuardMut<'a, F = False> = ArenaGuard<&'a mut Inner<F>, F>;
