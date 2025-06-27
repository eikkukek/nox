use core::{
    mem::needs_drop,
    ptr::NonNull,
};

use crate::vec_types::Vector;

pub unsafe trait MemoryStrategy<T> {
    unsafe fn move_elements(src: NonNull<T>, dst: NonNull<T>, len: usize);
    unsafe fn insert_element(ptr: NonNull<T>, value: T, index: usize, len: usize) -> NonNull<T>;
    unsafe fn drop_in_place(ptr: NonNull<T>, len: usize);
}

pub unsafe trait CloneStrategy<T: Clone> {
    unsafe fn clone_elements(src: NonNull<T>, dst: NonNull<T>, len: usize);
}

unsafe impl<T, V: Vector<T>> MemoryStrategy<T> for V {

    #[inline(always)] 
    unsafe fn move_elements(src: NonNull<T>, dst: NonNull<T>, len: usize) {
        if needs_drop::<T>() {
            unsafe {
                for i in 0..len {
                    dst.add(i).write(src.add(i).read())
                }
            }
        }
        else {
            unsafe {
                src.copy_to_nonoverlapping(dst, len);
            }
        }
    }

    #[inline(always)]
    unsafe fn insert_element(ptr: NonNull<T>, value: T, index: usize, len: usize) -> NonNull<T> {
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
                ptr.copy_to(ptr.add(1), len - index);
                let res = ptr.add(index);
                res.write(value);
                res
            }
        }
    }

    #[inline(always)]
    unsafe fn drop_in_place(ptr: NonNull<T>, len: usize) {
        if needs_drop::<T>() {
            unsafe {
                for i in 0..len {
                    ptr.add(i).drop_in_place();
                }
            }
        }
    }
}

unsafe impl<T: Clone, V: Vector<T>> CloneStrategy<T> for V {

    #[inline(always)]
    default unsafe fn clone_elements(src: NonNull<T>, dst: NonNull<T>, len: usize) {
        unsafe {
            for i in 0..len {
                dst.add(i).write(src.add(i).read().clone());
            }
        }
    }
}

unsafe impl<T: Copy, V: Vector<T>> CloneStrategy<T> for V {

    #[inline(always)]
    unsafe fn clone_elements(src: NonNull<T>, dst: NonNull<T>, len: usize) {
        unsafe {
            src.copy_to_nonoverlapping(dst, len);
        }
    }
}
