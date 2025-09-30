use std::alloc::{Layout, alloc, dealloc};

use core::ptr::NonNull;

use crate::Allocator;

pub struct GlobalAlloc;

impl Allocator for GlobalAlloc {

    unsafe fn allocate_raw(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let layout = Layout::from_size_align(size, align).ok()?;
        let ptr = unsafe { alloc(layout) };
        if layout.size() == 0 {
            return None
        }
        NonNull::new(ptr)
    }

    unsafe fn free_raw(&self, ptr: NonNull<u8>, size: usize, align: usize) {
        let layout = match Layout::from_size_align(size, align) {
            Ok(l) => l,
            Err(_) => return,
        };
        unsafe { dealloc(ptr.as_ptr(), layout) }
    }
}
