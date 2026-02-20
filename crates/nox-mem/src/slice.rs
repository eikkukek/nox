use crate::{
    size_of, align_of,
    Plain,
};

pub use core::slice::*;

/// Converts a slice over [`T`] to a slice over [`u8`].
pub fn as_bytes<T>(slice: &[T]) -> Option<&[u8]> {
    unsafe {
        Some(from_raw_parts(slice.as_ptr() as *const u8, size_of_val(slice)))
    }
}

/// Converts a mutable slice over [`T`] to a mutable slice over [`u8`].
/// # Safety
/// All writes to the slice are unsafe and it is up to the user to ensure those writes don't result
/// in invalid values of [`T`].
pub unsafe fn as_bytes_mut<T>(slice: &mut [T]) -> Option<&mut [u8]> {
    unsafe {
        Some(from_raw_parts_mut(slice.as_ptr() as *mut u8, size_of_val(slice)))
    }
}

/// Converts a value of [`T`] to a slice of [`u8`].
pub fn value_as_bytes<T>(value: &T) -> Option<&[u8]> {
    unsafe {
        Some(from_raw_parts(value as *const T as *const u8, size_of!(T)))
    }
}

/// Converts a value of [`T`] to a slice over [`u8`].
/// # Safety
/// All writes to the slice are unsafe and it is up to the programmer to ensure those writes don't
/// result in an invalid value of [`T`].
pub unsafe fn value_as_bytes_mut<T>(value: &mut T) -> Option<&mut [u8]> {
    unsafe {
        Some(from_raw_parts_mut(value as *mut T as *mut u8, size_of!(T)))
    }
}

/// Converts a slice over [`T`] to a slice over [`U`].
///
/// Alignments and sizessizes  must match.
///
/// # Safety
/// This is unsafe because [`U`] could contain fields which would become unaligned when
/// reinterpreting the bytes of [`T`] to [`U`].
pub unsafe fn cast<T: Copy, U: Copy>(slice: &[T]) -> Option<&[U]> {
    if size_of!(T) != size_of!(U) || align_of!(T) != align_of!(U) {
        None
    }
    else {
        unsafe {
            Some(from_raw_parts(slice.as_ptr().cast(), slice.len()))
        }
    }
}

/// A trait for working with allocated slices.
pub trait AllocSlice<T, SizeType = usize> {

    /// Allocates a slice over [`T`] with the given `value`.
    fn with_len(len: SizeType, value: T) -> Self
        where T: Clone;

    /// Allocates a slice over [`T`] of whose elements are constructed via [`F`].
    fn with_len_with<F>(len: SizeType, f: F) -> Self
        where F: FnMut(SizeType) -> T;

    /// Allocates a slice of uninitialized values.
    fn uninit_slice(len: SizeType) -> Self
        where T: Plain;
}

#[cfg(feature = "std")]
mod std_features {

    use super::*;

    use std::{
        sync::Arc,
        boxed::Box,
    };

    use core::mem;

    use crate::alloc::{StdAlloc, LocalAllocExt};

    /// A helper trait for constructing a boxed slice from the raw parts of a slice.
    pub trait BoxSlice<T> {
        
        /// Constructs a [`Box`] from the raw parts of a slice.
        ///
        /// # Safety
        /// This function assumes that the ownership of the pointers pointee is transferred to the box.
        /// 
        /// Additionally, the pointer must have been allocated via [`GlobalAlloc`] and if [`T`] implements
        /// [`Drop`], it must be assured that all elements of the slice are initialized by the time the
        /// box is dropped.
        unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> Self;
    }

    impl<T> BoxSlice<T> for Box<[T]> {

        unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> Self {
            unsafe {
                Self::from_raw(mem::transmute::<(*mut T, usize), *mut [T]>((ptr, len)))
            }
        }
    }

    impl<T> AllocSlice<T> for Box<[T]>
    {

        #[inline(always)]
        fn with_len(len: usize, value: T) -> Self
            where T: Clone
        {
            unsafe {
                let ptr: *mut T = StdAlloc
                    .allocate_uninit(len)
                    .unwrap().as_ptr();
                for i in 0..len {
                    ptr.add(i).write(value.clone());
                }
                Self::from_raw_parts(ptr, len)
            }
        }

        #[inline(always)]
        fn with_len_with<F>(len: usize, mut f: F) -> Self
            where F: FnMut(usize) -> T
        {
            unsafe {
                let ptr: *mut T = StdAlloc
                    .allocate_uninit(len)
                    .unwrap().as_ptr();
                for i in 0..len {
                    ptr.add(i).write(f(i));
                }
                Self::from_raw_parts(ptr, len)
            }
        }

        #[inline(always)]
        fn uninit_slice(len: usize) -> Self
            where T: Plain
        {
            unsafe {
                let ptr: *mut T = StdAlloc
                    .allocate_uninit(len)
                    .unwrap().as_ptr();
                Self::from_raw_parts(ptr, len)
            }
        }
    }

    impl<T> AllocSlice<T> for Arc<[T]> {

        #[inline(always)]
        fn with_len(len: usize, value: T) -> Self
            where T: Clone
        {
            (0..len).map(|_| value.clone()).collect()
        }

        #[inline(always)]
        fn with_len_with<F>(len: usize, f: F) -> Self
            where F: FnMut(usize) -> T
        {
            (0..len).map(f).collect()
        }

        #[inline(always)]
        fn uninit_slice(len: usize) -> Self
            where T: Plain
        {
            let val = Default::default();
            (0..len).map(|_| val).collect()
        }
    }
}
