use core::{
    mem::needs_drop,
    ptr,
};

pub trait MemoryStrategy {
    unsafe fn move_elements(src: *const Self, dst: *mut Self, len: usize);
    unsafe fn insert(ptr: *mut Self, value: Self, index: usize, len: usize) -> *mut Self;
    unsafe fn drop_in_place(ptr: *mut Self, len: usize);
}

pub trait DuplicateStrategy: Clone {
    unsafe fn duplicate(src: *const Self, dst: *mut Self, len: usize);
}

impl<T> MemoryStrategy for T {

    #[inline(always)] 
    unsafe fn move_elements(src: *const Self, dst: *mut Self, len: usize) {
        if needs_drop::<T>() {
            unsafe {
                for i in 0..len {
                    dst.add(i).write(src.add(i).read())
                }
            }
        }
        else {
            unsafe { ptr::copy_nonoverlapping(src, dst, len); }
        }
    }

    #[inline(always)]
    unsafe fn insert(ptr: *mut Self, value: Self, index: usize, len: usize) -> *mut Self {
        assert!(len >= index);
        if needs_drop::<T>() {
            unsafe {
                for i in (index + 1..=len).rev() {
                    ptr.add(i).write(ptr.add(i - 1).read());
                }
                let res = ptr.add(index);
                res.write(value);
                res
            }
        }
        else {
            unsafe {
                ptr::copy(ptr, ptr.add(1), len - index);
                let res = ptr.add(index);
                res.write(value);
                res
            }
        }
    }

    #[inline(always)]
    unsafe fn drop_in_place(ptr: *mut Self, len: usize) {
        if needs_drop::<T>() {
            unsafe {
                for i in 0..len {
                    ptr::drop_in_place(ptr.add(i));
                }
            }
        }
    }
}

impl<T: Clone> DuplicateStrategy for T {

    #[inline(always)]
    default unsafe fn duplicate(src: *const Self, dst: *mut Self, len: usize) {
        unsafe {
            for i in 0..len {
                dst.add(i).write(src.add(i).read().clone());
            }
        }
    }
}

impl<T: Copy> DuplicateStrategy for T {

    #[inline(always)]
    unsafe fn duplicate(src: *const Self, dst: *mut Self, len: usize) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len); }
    }
}
