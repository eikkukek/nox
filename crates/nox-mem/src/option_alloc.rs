use core::ptr::NonNull;

use crate::{Allocator, const_assert, size_of};

pub enum OptionAlloc<'alloc, Alloc: Allocator> {
    Some(&'alloc Alloc),
    None,
}

impl<'alloc, Alloc: Allocator> Allocator for OptionAlloc<'alloc, Alloc> {

    unsafe fn allocate_raw(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        if let Self::Some(alloc) = &self {
            unsafe {
                return alloc.allocate_raw(size, align)
            }
        }
        panic!("alloc was none")
    }

    unsafe fn allocate_uninit<T>(&self, count: usize) -> Option<NonNull<T>> {
        if let Self::Some(alloc) = &self {
            unsafe {
                return alloc.allocate_uninit(count)
            }
        }
        panic!("alloc was none")       
    }
    
    unsafe fn free_raw(&self, ptr: NonNull<u8>, size: usize, align: usize) {
        if let Self::Some(alloc) = &self {
            unsafe {
                alloc.free_raw(ptr, size, align)
            }
            return
        }
        panic!("alloc was none")
    }

    unsafe fn free_uninit<T>(&self, ptr: NonNull<T>, count: usize) {
        if let Self::Some(alloc) = &self {
            unsafe {
                alloc.free_uninit(ptr, count)
            }
            return
        }
        panic!("alloc was none")
    }
}

const_assert!(size_of!(OptionAlloc<'static, crate::GlobalAlloc>) == size_of!(&'static crate::GlobalAlloc));
