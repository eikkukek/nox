use core::{
    mem::needs_drop,
    ptr,
};

use crate::vec_types::Vector;

pub unsafe trait MemoryStrategy<T> {
    unsafe fn move_elements(src: *const T, dst: *mut T, len: usize);
    unsafe fn insert_element(ptr: *mut T, value: T, index: usize, len: usize) -> *mut T;
    unsafe fn drop_in_place(ptr: *mut T, len: usize);
}

pub unsafe trait CloneStrategy<T: Clone> {
    unsafe fn clone_elements(src: *const T, dst: *mut T, len: usize);
}

unsafe impl<T, V: Vector<T>> MemoryStrategy<T> for V {

    #[inline] 
    unsafe fn move_elements(src: *const T, dst: *mut T, len: usize) {
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

    #[inline]
    unsafe fn insert_element(ptr: *mut T, value: T, index: usize, len: usize) -> *mut T {
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

    #[inline]
    unsafe fn drop_in_place(ptr: *mut T, len: usize) {
        if needs_drop::<T>() {
            unsafe {
                for i in 0..len {
                    ptr::drop_in_place(ptr.add(i));
                }
            }
        }
    }
}

unsafe impl<T: Clone, V: Vector<T>> CloneStrategy<T> for V {

    #[inline]
    default unsafe fn clone_elements(src: *const T, dst: *mut T, len: usize) {
        unsafe {
            for i in 0..len {
                dst.add(i).write(src.add(i).read().clone());
            }
        }
    }
}

unsafe impl<T: Copy, V: Vector<T>> CloneStrategy<T> for V {

    #[inline]
    unsafe fn clone_elements(src: *const T, dst: *mut T, len: usize) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len); }
    }
}
