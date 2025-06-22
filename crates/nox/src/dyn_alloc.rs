use super::allocator_traits::{Allocate, Free};

pub struct _DynAlloc {}

impl Allocate for _DynAlloc {

    unsafe fn allocate_raw(&mut self, _size: usize, _align: usize) -> Option<std::ptr::NonNull<u8>> {
        None
    }

    unsafe fn allocate_uninit<T>(&mut self, _count: usize) -> Option<std::ptr::NonNull<T>> {
       None 
    }
}

impl Free for _DynAlloc {

    unsafe fn free_uninit<T>(&mut self, _ptr: std::ptr::NonNull<T>, _count: usize) {}

    unsafe fn free_raw(&mut self, _ptr: std::ptr::NonNull<u8>, _size: usize, _align: usize) {}
}
