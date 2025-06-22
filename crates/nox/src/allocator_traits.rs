use core::ptr::NonNull;

pub trait Allocate {

    unsafe fn allocate_raw(&mut self, size: usize, align: usize) -> Option<NonNull<u8>>;

    unsafe fn allocate_uninit<T>(&mut self, count: usize) -> Option<NonNull<T>> {
        let size = std::mem::size_of::<T>() * count;
        let align = std::mem::align_of::<T>();
        unsafe { self.allocate_raw(size, align).map(|ptr| ptr.cast::<T>()) }
    }
}

pub trait Free {

    unsafe fn free_raw(&mut self, ptr: NonNull<u8>, size: usize, align: usize);
    unsafe fn free_uninit<T>(&mut self, ptr: NonNull<T>, count: usize);
}
