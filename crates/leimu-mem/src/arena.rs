//! Fast, safe and atomic arena allocator implementing [`LocalAlloc`].

mod guard;

use core::{
    sync::atomic::{self, AtomicUsize, AtomicBool},
    ptr::NonNull,
    marker::PhantomData,
    error,
    fmt::{self, Display},
};

use crate::{
    conditional::{Conditional, True, False},
    alloc::{
        Layout, alloc, dealloc,
        LocalAlloc,
    },
    align_up,
};

pub use guard::ArenaGuard;

/// The allocation error used by [`Arena`].
#[derive(Debug)]
pub enum Error {
    /// Indicates that [`GlobalAlloc`][1] failed.
    ///
    /// [1]: alloc
    GlobalAllocFailed,
    /// Indicates that an allocation exceeded [`max_align`].
    MaximumAlignmentExceeded {
        /// The requested alignment.
        requested: usize,
    },
    /// Indicates that the arena is full.
    ArenaFull {
        /// The capacity of the arena.
        capacity: usize,
    },
    /// Indicates invalid usage of the arena while a guard is active.
    GuardActive,
}

impl Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GlobalAllocFailed => write!(f, "global alloc failed"),
            Self::MaximumAlignmentExceeded { requested } =>
                write!(f, "maximum supported alignment for arenas {} exceeded with {requested}",
                    max_align()
                ),
            Self::ArenaFull { capacity } => write!(f, "arena is full with capacity {capacity}"),
            Self::GuardActive => write!(f, "a guard is active"),
        }
    }
}

impl error::Error for Error {}

type Result<T> = core::result::Result<T, Error>;

/// Gets the alignment used when allocating an arena.
///
/// Equal to `32`.
pub const fn max_align() -> usize {
    32
}

/// Fast, safe and atomic arena allocator implementing [`LocalAlloc`].
///
/// Is [`Send`] and [`Sync`].
///
/// # Examples
/// ``` rust
/// use leimu_mem::alloc::LocalAlloc;
/// use leimu_mem::arena::Arena;
/// use leimu_mem::vec::FixedVec;
///
/// let arena = Arena::new(64).unwrap();
/// {
///     let guard = arena.guard();
///     let mut vec = FixedVec::with_capacity(5, &guard).unwrap();
///     vec.extend([0, 1, 2, 3, 4]);
///     assert_eq!(arena.used(), 5 * size_of::<i32>());
///     // Guard already active
///     assert!(arena.try_guard().is_none());
///     unsafe {
///         // Guard active
///         assert!(arena.alloc_raw(Layout::new::<i32>()).is_err())
///     }
/// }
/// // Guard dropped
/// assert_eq!(arena.used(), 0);
/// ```
pub struct Arena<F: Conditional = False> {
    data: NonNull<u8>,
    size: usize,
    pos: AtomicUsize,
    active_guard: AtomicBool,
    _marker: PhantomData<F>,
}

impl Arena {

    /// Creates a new arena with `size`.
    pub fn new(size: usize) -> Result<Self> {
        let layout = Layout
            ::from_size_align(size, max_align())
            .unwrap();
        let ptr = unsafe { alloc(layout) };
        Ok(Self {
            data: NonNull
                ::new(ptr)
                .ok_or(Error::GlobalAllocFailed)?,
            size,
            pos: AtomicUsize::new(0),
            active_guard: AtomicBool::new(false),
            _marker: PhantomData
        })
    }
}

impl Arena<True> {

    /// Creates a new arena with a fallback to [`GlobalAlloc`][1].
    ///
    /// [1]: std::alloc::alloc
    pub fn with_fallback(size: usize) -> Result<Self> {
        let layout = Layout
            ::from_size_align(size, max_align())
            .unwrap();
        let ptr = unsafe { alloc(layout) };
        Ok(Self {
            data: NonNull
                ::new(ptr)
                .ok_or(Error::GlobalAllocFailed)?,
            size,
            pos: AtomicUsize::new(0),
            active_guard: AtomicBool::new(false),
            _marker: PhantomData
        })
    }
}

impl<F: Conditional> Arena<F> { 

    /// Returns the total size of the arena.
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns how many bytes have been already used.
    #[inline]
    pub fn used(&self) -> usize {
        self.pos.load(atomic::Ordering::Relaxed)
    }

    /// Returns how much space is remaining.
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

    /// Consumes self, and returns the inner raw pointer and size of the arena.
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
    /// Only one guard can be active at a time and this will panic if a guard is already active.
    ///
    /// [1]: ArenaGuard
    #[inline]
    pub fn guard(&self) -> ArenaGuard<'_, F> {
        self.try_guard().expect("guard already active")
    }

    /// Tries to creates an [`arena guard`][1], which resets the position to the current position
    /// when it's dropped.
    ///
    /// Once a guard is active, all allocations made directly from this arena will return an error.
    ///
    /// Only one guard can be active at a time and this will return [`None`] if a guard is already
    /// active.
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

    unsafe fn alloc_raw_internal(
        &self,
        layout: Layout,
    ) -> Result<NonNull<u8>>
    {
        if layout.align() > max_align() {
            Err(Error::MaximumAlignmentExceeded { requested: layout.align() })
        } else {
            unsafe {
                self.alloc_unhecked(layout)
                    .ok_or(Error::ArenaFull { capacity: self.size })
            }
        }
    } 
}

impl Arena<True> {

    unsafe fn alloc_raw_internal(
        &self,
        layout: Layout,
    ) -> Result<NonNull<u8>>
    {
        if layout.align() > max_align() {
            unsafe {
                NonNull::new(alloc(layout))
                    .ok_or(Error::GlobalAllocFailed)
            }
        } else {
            unsafe {
                if let Some(ptr) = self.alloc_unhecked(layout) {
                    Ok(ptr)
                } else {
                    NonNull::new(alloc(layout))
                        .ok_or(Error::GlobalAllocFailed)
                }
            }
        }
    }

    unsafe fn free_raw_internal(&self, ptr: NonNull<u8>, layout: Layout) {
        let ptr_addr = ptr.as_ptr() as usize;
        let data_addr = self.data.as_ptr() as usize;
        if ptr_addr < data_addr || ptr_addr > data_addr + self.size {
            unsafe {
                dealloc(ptr.as_ptr(), layout);
            }
        }
    }
}

unsafe impl LocalAlloc for Arena {

    type Error = Error;

    #[inline]
    unsafe fn alloc_raw(&self, layout: Layout) -> core::result::Result<NonNull<u8>, Self::Error> {
        if self.active_guard.load(atomic::Ordering::Acquire) {
            return Err(Error::GuardActive)
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
            return Err(Error::GuardActive)
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
