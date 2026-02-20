mod prelude;
mod guard;
mod sync;
mod cell;

use std::alloc::{alloc, dealloc, Layout};

use prelude::{
    PhantomData, NonNull,
    DerefMut,
    LocalAlloc, StdAlloc,
    Conditional, True, False,
};

use nox_mem::align_up;

use nox_error::{Context, Result, Error};

pub use guard::{ArenaGuard, ArenaGuardMut};
pub use sync::RwArena;
pub use cell::CellArena;

/// A trait for arenas that can create an [`ArenaGuard`] through an immutable reference.
pub trait ImmutArena {}

/// A trait for arenas that can create an [`ArenaGuard`] through a mutable reference.
pub trait MutArena {}

/// A trait for arenas.
///
/// Provides methods for creating [`ArenaGuard`]s.
pub trait Arena<HasFallback = False>
    where HasFallback: Conditional,
{
    type Guard<'a>: DerefMut<Target = Inner<HasFallback>>
        where Self: 'a;

    /// Creates a new [`ArenaGuard`] that resets the position of [`Arena`] to the current position
    /// when it's dropped.
    ///
    /// May panic if a guard can't be created.
    fn guard(&self) -> ArenaGuard<Self::Guard<'_>, HasFallback>
        where
            Self: ImmutArena,
            for<'a> ArenaGuard<
                Self::Guard<'a>,
                HasFallback
            >: LocalAlloc<Error = Error>;

    /// Creates a new [`ArenaGuard`] that resets the position of [`Arena`] to the current position
    /// when it's dropped.
    ///
    /// Returns [`Err`] if a guard can't be created.
    fn try_guard(&self) -> Result<ArenaGuard<Self::Guard<'_>, HasFallback>>
        where
            Self: ImmutArena,
            for<'a> ArenaGuard<
                Self::Guard<'a>,
                HasFallback
            >: LocalAlloc<Error = Error>;
    
    /// Creates a new [`ArenaGuard`] from a mutable reference to [`Arena`], which resets the
    /// current position to the current position when it's dropped.
    ///
    /// Since this requires a mutable reference, no locks/checks need to take place.
    fn guard_mut(&mut self) -> ArenaGuard<&mut Inner<HasFallback>, HasFallback>
        where
            Self: MutArena,
            for<'a> ArenaGuard<
                &'a mut Inner<HasFallback>,
                HasFallback
            >: LocalAlloc<Error = Error>;
}

/// A helper subtrait for [`ArenaGuard`].
pub trait Guard<HasFallback>:
    Arena<HasFallback> +
    MutArena +
    LocalAlloc<Error = Error>
    where HasFallback: Conditional
{}

impl<Alloc, HasFallback> Guard<HasFallback> for
    ArenaGuard<Alloc, HasFallback>
    where
        Alloc: DerefMut<Target = Inner<HasFallback>>,
        HasFallback: Conditional,
        Self:
            Arena<HasFallback> +
            MutArena +
            LocalAlloc<Error = Error>,
{}


/// Gets the alignment used when allocating an arena.
///
/// Equal to `32`.
pub const fn max_align() -> usize {
    32
}

pub struct Inner<F: Conditional = False> {
    data: NonNull<u8>,
    size: usize,
    pos: usize,
    _marker: PhantomData<F>,
}

impl<F: Conditional> Inner<F> {

    pub fn new(size: usize) -> Result<Self> {
        let layout = Layout
            ::from_size_align(size, max_align())
            .context("layout error")?;
        let ptr = unsafe { alloc(layout) };
        Ok(Self {
            data: NonNull
                ::new(ptr)
                .ok_or_else(|| Error::just_context("global alloc failed"))?,
            size,
            pos: 0,
            _marker: PhantomData
        })
    }
}

impl<F: Conditional> Inner<F> {

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline(always)]
    pub fn used(&self) -> usize {
        self.pos
    }

    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.size - self.pos
    }

    #[inline(always)]
    pub fn into_raw_parts(self) -> (NonNull<u8>, usize) {
        let s = core::mem::ManuallyDrop::new(self);
        (s.data, s.size)
    }

    unsafe fn allocate_unhecked(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<u8>> {
        if layout.size() == 0 {
            unsafe {
                return Some(NonNull::new_unchecked(layout.align() as *mut u8))
            }
        }
        let start = self.data.as_ptr() as usize + self.pos;
        let aligned_start = align_up(start, layout.align());
        let end = aligned_start + layout.size();
        if end > self.data.as_ptr() as usize + self.size {
            return None
        }
        self.pos = end - self.data.as_ptr() as usize;
        unsafe {
            Some(NonNull::new_unchecked(aligned_start as *mut u8))
        }
    }
}

impl Inner { 

    /// Allocates memory from the arena without fallback.
    ///
    /// # Safety
    /// The pointer becomes invalid immediately after the arena is dropped.
    #[inline(always)]
    pub unsafe fn allocate_raw_internal(
        &mut self,
        layout: Layout,
    ) -> Result<NonNull<u8>>
    {
        if layout.align() > max_align() {
            Err(Error::just_context(format!(
                "maximum supported alignment for arenas on this platform is {}, requested alignment was {}",
                max_align(), layout.align(),
            )))
        } else {
            unsafe {
                self.allocate_unhecked(layout)
                .ok_or_else(|| Error::just_context(format!(
                    "arena was full with maximum capacity of {}",
                    self.size
                )))
            }
        }
    } 
}

impl Inner<True> {

    /// Allocates memory from the arena with a fallback to [`GlobalAlloc`].
    ///
    /// # Safety
    /// If the pointer didn't come from the fallback, it becomes immediately invalid after the
    /// arena is dropped. You should not use pointers after the arena is freed.
    #[inline(always)]
    pub unsafe fn allocate_raw_internal(
        &mut self,
        layout: Layout,
    ) -> Result<NonNull<u8>>
    {
        if layout.align() > max_align() {
            unsafe {
                StdAlloc.allocate_raw(layout)
                .context("arena fallback allocation failed")
            }
        } else {
            unsafe {
                if let Some(ptr) = self.allocate_unhecked(layout) {
                    Ok(ptr)
                } else {
                    StdAlloc.allocate_raw(layout)
                    .context("arena fallback allocation failed")
                }
            }
        }
    }

    #[inline(always)]
    unsafe fn free_raw_internal(&self, ptr: NonNull<u8>, layout: Layout) {
        let ptr_addr = ptr.as_ptr() as usize;
        let data_addr = self.data.as_ptr() as usize;
        if ptr_addr < data_addr || ptr_addr >= data_addr + self.size {
            unsafe {
                StdAlloc.free_raw(ptr, layout);
            }
        }
    }
}

unsafe impl<F: Conditional> Send for Inner<F> {}
unsafe impl<F: Conditional> Sync for Inner<F> {}

impl<F: Conditional> Drop for Inner<F> {

    fn drop(&mut self) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(self.size, max_align());
            dealloc(self.data.as_ptr(), layout);
        }
    }
}
