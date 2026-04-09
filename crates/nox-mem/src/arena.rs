mod guard;

pub use core::{
    sync::atomic::{self, AtomicUsize, AtomicBool},
    ptr::NonNull,
    marker::PhantomData,
};

pub use nox_mem::{
    conditional::{Conditional, True, False},
    alloc::{
        Layout, alloc, dealloc,
        StdAlloc, LocalAlloc,
    },
    align_up,
};
use nox_error::{Result, Context};
pub use nox_error::{Error};

pub use guard::ArenaGuard;

/// Gets the alignment used when allocating an arena.
///
/// Equal to `32`.
pub const fn max_align() -> usize {
    32
}

/// Fast, safe and atomic arena allocator implementing [`LocalAlloc`].
///
/// Is [`Send`] and [`Sync`].
pub struct Arena<F: Conditional = False> {
    data: NonNull<u8>,
    size: usize,
    pos: AtomicUsize,
    active_guard: AtomicBool,
    _marker: PhantomData<F>,
}

impl Arena {

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
            pos: AtomicUsize::new(0),
            active_guard: AtomicBool::new(false),
            _marker: PhantomData
        })
    }
}

impl Arena<True> {

    pub fn with_fallback(size: usize) -> Result<Self> {
        let layout = Layout
            ::from_size_align(size, max_align())
            .context("layout error")?;
        let ptr = unsafe { alloc(layout) };
        Ok(Self {
            data: NonNull
                ::new(ptr)
                .ok_or_else(|| Error::just_context("global alloc failed"))?,
            size,
            pos: AtomicUsize::new(0),
            active_guard: AtomicBool::new(false),
            _marker: PhantomData
        })
    }
}

impl<F: Conditional> Arena<F> { 

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn used(&self) -> usize {
        self.pos.load(atomic::Ordering::Relaxed)
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.size - self.pos.load(atomic::Ordering::Relaxed)
    }

    /// Resets the arena's internal pointer to zero.
    ///
    /// This will panic if there's a [`guard`][1] active.
    ///
    /// # Safety
    /// Previous allocations may be overwritten after this.
    ///
    /// [1]: ArenaGuard
    #[inline]
    pub unsafe fn clear(&self) {
        assert!(
            !self.active_guard.load(atomic::Ordering::Acquire),
            "attempting to clear when a guard is active"
        );
        self.pos.store(0, atomic::Ordering::Release);
    }

    #[inline]
    pub fn into_raw_parts(self) -> (NonNull<u8>, usize) {
        let s = core::mem::ManuallyDrop::new(self);
        (s.data, s.size)
    }

    /// Creates an [`arena guard`][1], which resets the position to the current position when it's
    /// dropped.
    ///
    /// Once a guard is active, all allocations made directly from this arena will return an error.
    ///
    /// Only one guard can be active at a time and this will panic if that's true.
    ///
    /// [1]: ArenaGuard
    #[inline]
    pub fn guard(&self) -> ArenaGuard<'_, F> {
        self.try_guard().expect("guard already active")
    }

    /// Creates an [`arena guard`][1], which resets the position to the current position when it's
    /// dropped.
    ///
    /// Once a guard is active, all allocations made directly from this arena will return an error.
    ///
    /// Only one guard can be active at a time and this will return [`None`] if that's true.
    ///
    /// [1]: ArenaGuard
    #[inline]
    pub fn try_guard(&self) -> Option<ArenaGuard<'_, F>> {
        (!self.active_guard.swap(true, atomic::Ordering::AcqRel)).then(||
            ArenaGuard::new(self)
        )
    }

    unsafe fn alloc_unhecked(
        &self,
        layout: Layout,
    ) -> Option<NonNull<u8>> {
        if layout.size() == 0 {
            unsafe {
                return Some(NonNull::new_unchecked(layout.align() as *mut u8))
            }
        }
        let mut aligned_start = 0;
        self.pos.fetch_update(
            atomic::Ordering::AcqRel,
            atomic::Ordering::Acquire,
            |pos|  {
                let start = self.data.as_ptr() as usize + pos;
                aligned_start = align_up(start, layout.align());
                let end = aligned_start + layout.size();
                if end > self.data.as_ptr() as usize + self.size {
                    return None
                }
                Some(end - self.data.as_ptr() as usize)
            }
        ).ok()?;
        unsafe {
            Some(NonNull::new_unchecked(aligned_start as *mut u8))
        }
    }
}

impl Arena { 

    /// Allocates memory from the arena without fallback.
    ///
    /// # Safety
    /// The pointer becomes invalid immediately after the arena is dropped.
    unsafe fn alloc_raw_internal(
        &self,
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
                self.alloc_unhecked(layout)
                    .ok_or_else(|| Error::just_context(format!(
                        "arena was full with maximum capacity of {}",
                        self.size
                    )))
            }
        }
    } 
}

impl Arena<True> {

    /// Allocates memory from the arena with a fallback to [`GlobalAlloc`].
    ///
    /// # Safety
    /// If the pointer didn't come from the fallback, it becomes immediately invalid after the
    /// arena is dropped. You should not use pointers after the arena is freed.
    unsafe fn alloc_raw_internal(
        &self,
        layout: Layout,
    ) -> Result<NonNull<u8>>
    {
        if layout.align() > max_align() {
            unsafe {
                StdAlloc.alloc_raw(layout)
                    .context("arena fallback allocation failed")
            }
        } else {
            unsafe {
                if let Some(ptr) = self.alloc_unhecked(layout) {
                    Ok(ptr)
                } else {
                    StdAlloc.alloc_raw(layout)
                        .context("arena fallback allocation failed")
                }
            }
        }
    }

    unsafe fn free_raw_internal(&self, ptr: NonNull<u8>, layout: Layout) {
        let ptr_addr = ptr.as_ptr() as usize;
        let data_addr = self.data.as_ptr() as usize;
        if ptr_addr < data_addr || ptr_addr > data_addr + self.size {
            unsafe {
                StdAlloc.free_raw(ptr, layout);
            }
        }
    }
}

unsafe impl LocalAlloc for Arena {

    type Error = Error;

    #[inline]
    unsafe fn alloc_raw(&self, layout: Layout) -> core::result::Result<NonNull<u8>, Self::Error> {
        if self.active_guard.load(atomic::Ordering::Acquire) {
            return Err(Error::just_context("a guard is active"))
        }
        unsafe {
            self.alloc_raw_internal(layout)
        }
    }

    #[inline]
    unsafe fn free_raw(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

unsafe impl LocalAlloc for Arena<True> {

    type Error = Error;

    #[inline]
    unsafe fn alloc_raw(&self, layout: Layout) -> std::result::Result<NonNull<u8>, Self::Error> {
        if self.active_guard.load(atomic::Ordering::Acquire) {
            return Err(Error::just_context("a guard is active"))
        }
        unsafe {
            self.alloc_raw_internal(layout)
        }
    }

    #[inline]
    unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            self.free_raw_internal(ptr, layout);
        }
    }
}

unsafe impl<F: Conditional> Send for Arena<F> {}
unsafe impl<F: Conditional> Sync for Arena<F> {}

impl<F: Conditional> Drop for Arena<F> {

    fn drop(&mut self) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(self.size, max_align());
            dealloc(self.data.as_ptr(), layout);
        }
    }
}
