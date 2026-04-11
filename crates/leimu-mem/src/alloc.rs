//! An extension of [`alloc`][1].
//!
//! # New traits
//! - [`LocalAlloc`]: A trait for local, owned allocators.
//! - [`LocalAllocExt`]: An auto-trait for types implementing [`LocalAlloc`].
//! # New types
//! - [`StdAlloc`]: An implementation of [`LocalAlloc`] using [`GlobalAlloc`][2]. Requires the
//!   "std" feature.
//!
//! [1]: core::alloc
//! [2]: std::alloc::alloc

pub use core::alloc::*;

use core::{
    ptr::NonNull,
    error::Error,
    mem,
    marker::PhantomData,
    ops::Deref,
};

/// A trait for local, owned allocators.
///
/// # Safety
/// It has to be ensured that the allocations made by allocators implementing this trait return
/// valid, aligned pointers.
pub unsafe trait LocalAlloc
{

    /// The error returned when an allocation fails.
    type Error: Error + Send + Sync + 'static;
   
    /// Allocates a raw block of bytes of `size` aligned to `align`.
    /// # Safety
    /// The pointer returned must be aligned to `align` and point to a valid array of bytes up to
    /// `size`.
    unsafe fn alloc_raw(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error>;

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

    /// Allocates an uninitialized block of `T`.
    /// # Safety
    /// The pointer returned must be aligned to the alignment of `T` and point to a valid array
    /// of type `T` up to `count`.
    #[inline]
    unsafe fn alloc_uninit<T>(&self, count: usize) -> Result<NonNull<T>, Self::Error> {
        let size = mem::size_of::<T>() * count;
        let align = mem::align_of::<T>();
        unsafe { self.alloc_raw(Layout::from_size_align_unchecked(size, align)).map(|ptr| ptr.cast::<T>()) }
    }

    /// Frees a previously allocated block of `T` of `count`. This does not call [`drop`] on any
    /// values.
    /// # Safety
    /// The pointer passed to this function must be the result of a previous allocation of `T`
    /// and `count` from *same allocator*.
    unsafe fn free_uninit<T>(&self, ptr: NonNull<T>, count: usize) {
        let size = mem::size_of::<T>() * count;
        let align = mem::align_of::<T>();
        unsafe { self.free_raw(ptr.cast::<u8>(), Layout::from_size_align_unchecked(size, align)) }
    }

    /// Frees a previously allocated value of `T`.
    ///
    /// This also drops the value.
    ///
    /// # Safety
    /// The pointer passed to this function must be the result of a previous allocation of `T`
    /// from the *same allocator*. The value must point to a valid `T` and it must not be dropped
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

unsafe impl<T> LocalAlloc for T
    where
        T: Deref,
        <T as Deref>::Target: LocalAlloc,
{

    type Error = <<T as Deref>::Target as LocalAlloc>::Error;

    #[inline]
    unsafe fn alloc_raw(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        unsafe {
            self.deref().alloc_raw(layout)
        }
    }

    #[inline]
    unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            self.deref().free_raw(ptr, layout);
        }
    }
}

impl<T: LocalAlloc> LocalAllocExt for T {}

/// A wrapper around a type containing/referencing [`LocalAlloc`].
///
/// Used by custom containers.
pub struct LocalAllocWrap<Alloc, Wrap>
    where
        Alloc: LocalAlloc + ?Sized,
        Wrap: Deref<Target = Alloc>,
{
    alloc: Wrap,
    _marker: PhantomData<Alloc>,
}

impl<Alloc, Wrap> LocalAllocWrap<Alloc, Wrap>
    where
        Alloc: LocalAlloc + ?Sized,
        Wrap: Deref<Target = Alloc>,
{

    /// Creates a new [`LocalAllocWrap`].
    #[inline]
    pub fn new(alloc: Wrap) -> Self {
        Self {
            alloc,
            _marker: PhantomData,
        }
    }
}

impl<Alloc, Wrap> AsRef<Self> for LocalAllocWrap<Alloc, Wrap>
    where
        Alloc: LocalAlloc + ?Sized,
        Wrap: Deref<Target = Alloc>,
{

    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

unsafe impl<Alloc, Wrap> LocalAlloc for LocalAllocWrap<Alloc, Wrap>
    where
        Alloc: LocalAlloc + ?Sized,
        Wrap: Deref<Target = Alloc>,
{

    type Error = Alloc::Error;

    #[inline]
    unsafe fn alloc_raw(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        unsafe {
            self.alloc.alloc_raw(layout)
        }
    }

    #[inline]
    unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            self.alloc.free_raw(ptr, layout)
        }
    }
}

#[cfg(feature = "std")]
mod std_features {

    use super::*;

    use core::{
        fmt::{self, Display},
        error::Error,
    };

    pub use std::alloc::*;

    /// An global alloc error.
    #[derive(Debug)]
    pub struct StdAllocError {
        layout: Layout,
    }

    impl Display for StdAllocError {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "global alloc failed for layout {:?}", self.layout)
        }
    }

    impl Error for StdAllocError {}

    /// Allocator using [`GlobalAlloc`].
    pub struct StdAlloc;

    unsafe impl LocalAlloc for StdAlloc {

        type Error = StdAllocError;

        #[inline]
        unsafe fn alloc_raw(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
            let ptr = unsafe { alloc(layout) };
            NonNull::new(ptr).ok_or(StdAllocError { layout })
        }

        #[inline]
        unsafe fn free_raw(&self, ptr: NonNull<u8>, layout: Layout) {
            unsafe { dealloc(ptr.as_ptr(), layout) }
        }
    }
}

#[cfg(feature = "std")]
pub use std_features::*;

/// Allocates data as contiguously as possible.
///
/// # Examples
/// ``` rust
/// use nox_mem::pack_alloc;
///
/// let layout;
/// let ptr;
/// let pack1;
/// let pack2;
/// unsafe {
///     pack_alloc!(
///         layout as Layout,
///         ptr as *mut u8,
///         pack1 as [u8; 10],
///         pack2 as [String; 1],
///     );
///     pack2.write("hello".to_string());
///     for i in 0..10 {
///         pack.add(i).write(i as u8 / 2);
///     }
///     pack2.drop_in_place();
///     std::alloc::dealloc(ptr, layout);
/// }
/// ```
#[cfg(feature = "std")]
#[macro_export]
macro_rules! pack_alloc {
    (
        $layout:ident as Layout,
        $ptr:ident as *mut u8,
        $($pack:ident as [$t:ty; $n:expr]),* $(,)?
    ) => {
        let mut align = 1;
        $(
            align = align.max(align_of::<$t>());
        )*
        let mut size = 0;
        $($crate::paste! {
            size = $crate::align_up(size, align_of::<$t>());
            let mut [<$pack _ off>] = size;
            size += size_of::<$t>() * $n;
        })*
        $layout = std::alloc::Layout::from_size_align_unchecked(size, align);
        $ptr = std::alloc::alloc(
            $layout
        ).cast::<u8>();
        assert!(!$ptr.is_null(), "global alloc failed");
        $($crate::paste! {
            $pack = $ptr.add([<$pack _ off>]).cast::<$t>();
        })*
    };
}
