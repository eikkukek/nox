use super::allocator_traits::{Allocate, Free};

use std::alloc::{Layout, alloc, dealloc};

use core::ptr::NonNull;

#[derive(Default)]
pub struct GlobalAlloc {}

impl Allocate for GlobalAlloc {

    unsafe fn allocate_raw(&mut self, size: usize, align: usize) -> Option<std::ptr::NonNull<u8>> {
        let layout = Layout::from_size_align(size, align).ok()?;
        let ptr = unsafe { alloc(layout) };
        Some(NonNull::new(ptr)?)
    }

    unsafe fn allocate_uninit<T>(&mut self, count: usize) -> Option<NonNull<T>> {
        let layout = Layout::from_size_align(count * size_of::<T>(), align_of::<T>()).ok()?;
        let ptr = unsafe { alloc(layout) };
        Some(NonNull::new(ptr as *mut T)?)
    }
}

impl Free for GlobalAlloc {

    unsafe fn free_raw(&mut self, ptr: NonNull<u8>, size: usize, align: usize) {
        let layout = match Layout::from_size_align(size, align) {
            Ok(r) => r,
            Err(_) => return,
        };
        unsafe { dealloc(ptr.as_ptr(), layout); }
    }

    unsafe fn free_uninit<T>(&mut self, ptr: NonNull<T>, count: usize) {
        let layout = match Layout::from_size_align(count * size_of::<T>(), align_of::<T>()) {
            Ok(r) => r,
            Err(_) => return,
        };
        unsafe { dealloc(ptr.as_ptr() as *mut u8, layout); }
    }
}
