pub use core::alloc::*;

use core::{
    ptr::NonNull,
    error::Error,
    mem,
    marker::PhantomData,
    ops::Deref,
};

use nox_proc::Error;

use crate::dynamic::Dyn;

/// A trait for local, owned allocators.
///
/// # Safety
/// It has to be ensured that the allocations made by allocators implementing this trait return
/// valid, aligned pointers.
pub unsafe trait LocalAlloc
{

    type Error: Error + Send + Sync + 'static;
   
    /// Allocates a raw block of bytes of `size` aligned to `align`.
    /// # Safety
    /// The pointer returned must be aligned to `align` and point to a valid array of bytes up to
    /// `size`.
    unsafe fn allocate_raw(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error>;

    /// Frees a previously allocated block of `size` and `align`.
    /// # Safety
    /// The pointer passed to this function must be the result of a previous allocation from the
    /// *same allocator* of the same `size` and `align`. You *must* not free the same pointer
    /// twice.
    unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout); 
}

/// An auto-trait for all types implementing [`LocalAlloc`].
///
/// Provides methods for allocating memory based on generics.
pub trait LocalAllocExt: LocalAlloc {

    /// Allocates an uninitialized block of [`T`].
    /// # Safety
    /// The pointer returned must be aligned to the alignment of [`T`] and point to a valid array
    /// of type [`T`] up to `count`.
    #[inline(always)]
    unsafe fn allocate_uninit<T>(&self, count: usize) -> Result<NonNull<T>, Self::Error> {
        let size = mem::size_of::<T>() * count;
        let align = mem::align_of::<T>();
        unsafe { self.allocate_raw(Layout::from_size_align_unchecked(size, align)).map(|ptr| ptr.cast::<T>()) }
    }

    /// Allocates a potentially unsized type of [`T`] from a sized type [`U`] implementing the
    /// [`Dyn<T>`] trait.
    /// # Safety
    /// The pointer returned must be aligned to the alignment of [`T`] and point to a valid [`T`].
    unsafe fn allocate_dyn<T: ?Sized, U: Dyn<T, Target = U>>(&self, value: U) -> Result<NonNull<T>, Self::Error> {
        unsafe {
            let mut raw_parts = U::raw_parts(&value);
            let ptr = self.allocate_uninit(1)?;
            ptr.write(value);
            raw_parts.data = ptr.as_ptr();
            Ok(NonNull::new_unchecked(<U as Dyn<T>>::from_raw_parts(raw_parts).cast_mut()))
        }
    }

    /// Frees a previously allocated block of [`T`] of `count`. This does not call [`drop`] on any
    /// values.
    /// # Safety
    /// The pointer passed to this function must be the result of a previous allocation of [`T`]
    /// and `count` from *same allocator*.
    unsafe fn free_uninit<T>(&self, ptr: NonNull<T>, count: usize) {
        let size = mem::size_of::<T>() * count;
        let align = mem::align_of::<T>();
        unsafe { self.free_raw(ptr.cast::<u8>(), Layout::from_size_align_unchecked(size, align)) }
    }

    /// Frees a previously allocated value of [`T`].
    ///
    /// This also drops the value.
    ///
    /// # Safety
    /// The pointer passed to this function must be the result of a previous allocation of [`T`]
    /// from the *same allocator*. The value must point to a valid [`T`] and it must not be dropped
    /// before or after calling this function.
    unsafe fn free_dyn<T: ?Sized>(&self, value: NonNull<T>) {
        unsafe {
            let r = value.as_ref();
            let size = size_of_val(r);
            let align = align_of_val(r);
            value.drop_in_place();
            self.free_raw(value.cast(), Layout::from_size_align_unchecked(size, align));
        }
    }
}

impl<T: LocalAlloc> LocalAllocExt for T {}

pub struct LocalAllocWrap<Alloc, Wrap>
    where
        Alloc: LocalAlloc,
        Wrap: Deref<Target = Alloc>,
{
    pub alloc: Wrap,
    _marker: PhantomData<Alloc>,
}

impl<Alloc, Wrap> LocalAllocWrap<Alloc, Wrap>
    where
        Alloc: LocalAlloc,
        Wrap: Deref<Target = Alloc>,
{

    #[inline(always)]
    pub fn new(alloc: Wrap) -> Self {
        Self {
            alloc,
            _marker: PhantomData,
        }
    }
}

impl<Alloc, Wrap> AsRef<Self> for LocalAllocWrap<Alloc, Wrap>
    where
        Alloc: LocalAlloc,
        Wrap: Deref<Target = Alloc>,
{

    #[inline(always)]
    fn as_ref(&self) -> &Self {
        self
    }
}

unsafe impl<Alloc, Wrap> LocalAlloc for LocalAllocWrap<Alloc, Wrap>
    where
        Alloc: LocalAlloc,
        Wrap: Deref<Target = Alloc>,
{

    type Error = Alloc::Error;

    #[inline(always)]
    unsafe fn allocate_raw(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        unsafe {
            self.alloc.allocate_raw(layout)
        }
    }

    #[inline(always)]
    unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            self.alloc.free_raw(ptr, layout)
        }
    }
}

#[cfg(feature = "std")]
mod std_features {

    use super::*;

    pub use std::alloc::*;


    #[derive(Debug, Error)] #[display("global alloc failed for layout {layout:?}")]
    pub struct StdAllocError {
        layout: Layout,
    }

    /// Allocator using [`GlobalAlloc`].
    pub struct StdAlloc;

    unsafe impl LocalAlloc for StdAlloc {

        type Error = StdAllocError;

        #[inline(always)]
        unsafe fn allocate_raw(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
            let ptr = unsafe { alloc(layout) };
            NonNull::new(ptr).ok_or(StdAllocError { layout })
        }

        #[inline(always)]
        unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout) {
            unsafe { dealloc(ptr.as_ptr(), layout) }
        }
    }
}

#[cfg(feature = "std")]
pub use std_features::*;
