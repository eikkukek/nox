use std::ptr::NonNull;

pub trait AllocateExt<'allocator> {

    unsafe fn allocate_raw(&mut self, size: usize, align: usize) -> Option<NonNull<u8>>;

    unsafe fn allocate_uninit<T>(&mut self, count: usize) -> Option<NonNull<T>>;

    fn allocate_default<T>(&mut self, count: usize) -> Option<NonNull<T>>
        where
            T: Default
    {
        let ptr = unsafe { self.allocate_uninit(count)? };
        for i in 0..count {
            unsafe {
                ptr.add(i).write(T::default());
            }
        }
        Some(ptr)
    }

    fn allocate_with<T, F>(&mut self, count: usize, mut f: F) -> Option<NonNull<T>>
        where
            F: FnMut(usize) -> T
    {
        let ptr = unsafe { self.allocate_uninit(count)? };
        for i in 0..count {
            unsafe {
                ptr.add(i).write(f(i));
            }
        }
        Some(ptr)
    }
}
