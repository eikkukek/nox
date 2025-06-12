use std::alloc::{alloc, dealloc, Layout};

use core::{
    mem,
    cell::RefCell,
    ptr::NonNull,
    marker::PhantomData,
};

use super::{
    allocator_traits::{Allocate, Free},
    utility::next_align,
};

pub struct StackAlloc {
    data: NonNull<u8>,
    size: usize,
    pos: usize,
}

impl StackAlloc {

    pub fn new(size: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, mem::align_of::<usize>()).ok()?;
        let ptr = unsafe { alloc(layout) };
        Some(
            Self {
                data: NonNull::new(ptr)?,
                size,
                pos: 0,
            }
        )
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn used(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.size - self.pos
    }

    pub fn full(&self) -> bool {
        self.pos >= self.size
    }

    pub fn clear(&mut self) {
        self.pos = 0;
    }
}

impl Allocate for StackAlloc {

    unsafe fn allocate_raw(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let start = self.data.as_ptr() as usize + self.pos;
        let aligned_start = next_align(start, align);
        let end = aligned_start + size;
        if end > self.data.as_ptr() as usize + self.size {
            return None
        }
        self.pos = end - self.data.as_ptr() as usize;
        Some(
            unsafe {
                NonNull::new_unchecked(aligned_start as *mut u8)
            }
        )
    }
}

impl Free for StackAlloc {

    unsafe fn free_raw(&mut self, _ptr: NonNull<u8>, _size: usize, _align: usize) {}
    unsafe fn free_uninit<T>(&mut self, _ptr: NonNull<T>, _count: usize) {}
}

impl Drop for StackAlloc {

    fn drop(&mut self) {
        if self.size == 0 { panic!("attempting to drop twice") }
        unsafe {
            let layout = Layout::from_size_align_unchecked(self.size, mem::align_of::<usize>());
            dealloc(self.data.as_ptr(), layout);
            self.size = 0;
        }
    }
}

pub struct StackGuard<'alloc> {
    pos_rollback: usize,
    stack: &'alloc RefCell<StackAlloc>,
}

impl<'alloc> StackGuard<'alloc> {

    pub fn new(stack: &'alloc RefCell<StackAlloc>) -> Self {
        let pos_rollback = stack.borrow().pos;
        Self {
            stack,
            pos_rollback,
        }
    }

    pub fn size(&self) -> usize {
        self.stack.borrow().size
    }

    pub fn reminaing(&self) -> usize {
        self.stack.borrow().remaining()
    }
}

impl<'alloc> Allocate for StackGuard<'alloc> {

    unsafe fn allocate_raw(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        unsafe { self.stack.borrow_mut().allocate_raw(size, align) }
    }

    unsafe fn allocate_uninit<T>(&mut self, count: usize) -> Option<NonNull<T>> {
        unsafe { self.stack.borrow_mut().allocate_uninit(count) }
    }
}

impl<'alloc> Free for StackGuard<'alloc> {

    unsafe fn free_raw(&mut self, _ptr: NonNull<u8>, _size: usize, _align: usize) {}
    unsafe fn free_uninit<T>(&mut self, _ptr: NonNull<T>, _count: usize) {}
}

impl<'alloc> Drop for StackGuard<'alloc> {

    fn drop(&mut self) {
        self.stack.borrow_mut().pos = self.pos_rollback;
    }
}

pub struct _StackReg<'alloc> {
    data: NonNull<u8>,
    size: usize,
    pos: usize,
    _marker: PhantomData<&'alloc ()>,
}

impl<'alloc> _StackReg<'alloc> {

    pub fn _new<'stack>(size: usize, stack: &'alloc RefCell<StackAlloc>) -> Option<Self>
    {
        let mut a = stack.borrow_mut();
        let ptr = unsafe { a.allocate_raw(size, align_of::<usize>())? };
        Some(
            Self {
                data: ptr,
                size,
                pos: 0,
                _marker: PhantomData,
            }
        )
    }

    pub fn _clear(&mut self) {
        self.pos = 0;
    }
}

impl<'alloc> Allocate for _StackReg<'alloc> {

    unsafe fn allocate_raw(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let start = self.data.as_ptr() as usize + self.pos;
        let aligned_start = next_align(start, align);
        let end = aligned_start + size;
        if end > self.data.as_ptr() as usize + self.size {
            return None
        }
        self.pos = end - self.data.as_ptr() as usize;
        Some(
            unsafe {
                NonNull::new_unchecked(aligned_start as *mut u8)
            }
        )
    }
}

impl<'alloc> Free for _StackReg<'alloc> {

    unsafe fn free_raw(&mut self, _ptr: NonNull<u8>, _size: usize, _align: usize) {}
    unsafe fn free_uninit<T>(&mut self, _ptr: NonNull<T>, _count: usize) {}
}
