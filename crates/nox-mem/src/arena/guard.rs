use super::*;

/// An arena guard returned by [`Arena`] guard methods.
pub struct ArenaGuard<'a, F = False>
    where F: Conditional,
{
    arena: &'a Arena<F>,
    pos_rollback: usize,
}

impl<'a, F> ArenaGuard<'a, F>
    where F: Conditional,
{

    #[inline]
    pub(super) fn new(arena: &'a Arena<F>) -> Self {
        Self {
            pos_rollback: arena.pos.load(atomic::Ordering::Acquire),
            arena,
        }
    }

    /// Returns the total size of the parent arena.
    #[inline]
    pub fn size(&self) -> usize {
        self.arena.size()
    }

    /// Returns how many bytes has been allocated since the guard was created.
    #[inline]
    pub fn used(&self) -> usize {
        self.arena.used() - self.pos_rollback
    }

    /// Returns how many bytes remaining in the parent arena.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.arena.remaining()
    }

    /// Resets the pointer position to the guard position roll back.
    /// # Safety
    /// Any allocations still in use in the guard's region may be overwritten.
    #[inline]
    pub unsafe fn clear(&mut self) {
        self.arena.pos.store(self.pos_rollback, atomic::Ordering::Release);
    }
}

unsafe impl<'a> LocalAlloc for ArenaGuard<'a, False>
{

    type Error = Error;

    #[inline]
    unsafe fn alloc_raw(&self, layout: Layout) -> Result<NonNull<u8>> {
        unsafe {
            self.arena.alloc_raw_internal(layout)
        }
    }

    #[inline]
    unsafe fn free_raw(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

unsafe impl<'a> LocalAlloc for ArenaGuard<'a, True>
{

    type Error = Error;

    #[inline]
    unsafe fn alloc_raw(&self, layout: Layout) -> Result<NonNull<u8>> {
        unsafe {
            self.arena.alloc_raw_internal(layout)
        }
    }

    #[inline]
    unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            self.arena.free_raw_internal(ptr, layout);
        }
    }
}

impl<'a, F> Drop for ArenaGuard<'a, F>
    where F: Conditional,
{

    fn drop(&mut self) {
        self.arena.pos.store(
            self.pos_rollback,
            atomic::Ordering::Release,
        );
        self.arena.active_guard.store(
            false, atomic::Ordering::Release
        );
    }
}
