use core::slice;

use crate::size_of;

pub unsafe fn slice_as_bytes<T>(slice: &[T]) -> Option<&[u8]> {
    unsafe {
        Some(slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len() * size_of!(T)))
    }
}

pub unsafe fn slice_as_bytes_mut<T>(slice: &mut [T]) -> Option<&mut [u8]> {
    unsafe {
        Some(slice::from_raw_parts_mut(slice.as_ptr() as *mut u8, slice.len() * size_of!(T)))
    }
}

pub unsafe fn value_as_bytes<T>(value: &T) -> Option<&[u8]> {
    unsafe {
        Some(slice::from_raw_parts(value as *const T as *const u8, size_of!(T)))
    }
}

pub unsafe fn value_as_bytes_mut<T>(value: &mut T) -> Option<&mut [u8]> {
    unsafe {
        Some(slice::from_raw_parts_mut(value as *mut T as *mut u8, size_of!(T)))
    }
}
