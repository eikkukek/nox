use core::{
    ptr::NonNull,
    mem,
};

pub trait Allocator {

    unsafe fn allocate_raw(&self, size: usize, align: usize) -> Option<NonNull<u8>>;

    unsafe fn allocate_uninit<T>(&self, count: usize) -> Option<NonNull<T>> {
        let size = mem::size_of::<T>() * count;
        let align = mem::align_of::<T>();
        unsafe { self.allocate_raw(size, align).map(|ptr| ptr.cast::<T>()) }
    }

    unsafe fn free_raw(&self, ptr: NonNull<u8>, size: usize, align: usize);

    unsafe fn free_uninit<T>(&self, ptr: NonNull<T>, count: usize) {
        let size = mem::size_of::<T>() * count;
        let align = mem::align_of::<T>();
        unsafe { self.free_raw(ptr.cast::<u8>(), size, align) }
    }
}
