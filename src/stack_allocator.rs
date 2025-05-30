use super::allocator_traits::AllocateExt;

use std::{
    mem, alloc::{alloc, dealloc, Layout}, ptr::NonNull, marker::PhantomData,
};

pub struct StackMemory {
    data: NonNull<u8>,
    size: usize,
    pos: usize,
}

impl StackMemory {

    pub fn new(size: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, mem::align_of::<usize>()).ok()?;
        let ptr = unsafe { alloc(layout) };
        let data = NonNull::new(ptr)?;
        Some(
            Self {
                data,
                size,
                pos: 0,
            }
        )
    }

    pub unsafe fn allocate(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let start = self.data.as_ptr() as usize + self.pos;
        let aligned_start = (start + align - 1) & !(align - 1);
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

impl Drop for StackMemory {

    fn drop(&mut self) {
        if self.size == 0 { panic!("attempting to drop twice") }
        unsafe {
            let layout = Layout::from_size_align_unchecked(self.size, mem::align_of::<usize>());
            dealloc(self.data.as_ptr(), layout);
            self.size = 0;
        }
    }
}

pub struct StackAllocator<'mem> {
    data: NonNull<u8>,
    size: usize,
    pos: usize,
    _marker: PhantomData<&'mem mut NonNull<u8>>,
}

impl<'mem> StackAllocator<'mem> {

    pub fn new<'short>(size: usize, memory: &'short mut StackMemory) -> Option<Self> {
        let ptr = unsafe { memory.allocate(size, mem::align_of::<usize>())? };
        Some(
            Self {
                data: ptr,
                size,
                pos: 0,
                _marker: PhantomData,
            }
        )
    }

    pub unsafe fn allocate_raw(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let start = self.data.as_ptr() as usize + self.pos;
        let aligned_start = (start + align - 1) & !(align - 1);
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

    pub unsafe fn allocate_uninit<T>(&mut self, count: usize) -> Option<NonNull<T>> {
        let size = std::mem::size_of::<T>() * count;
        let align = std::mem::align_of::<T>();
        unsafe { self.allocate_raw(size, align).map(|ptr| ptr.cast::<T>()) }
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

pub struct StackGuard<'stack, 'mem> {
    stack: &'stack mut StackAllocator<'mem>,
    pos_rollback: usize,
}

impl<'stack, 'mem> StackGuard<'stack, 'mem> {

    pub fn new(stack: &'stack mut StackAllocator<'mem>) -> Self {
        let pos_rollback = stack.pos;
        Self {
            stack,
            pos_rollback,
        }
    }

    pub fn size(&self) -> usize {
        self.stack.size()
    }

    pub fn reminaing(&self) -> usize {
        self.stack.remaining()
    }
}

impl<'stack, 'mem> AllocateExt<'stack> for StackGuard<'stack, 'mem> {

    unsafe fn allocate_raw(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        unsafe { self.stack.allocate_raw(size, align) }
    }

    unsafe fn allocate_uninit<T>(&mut self, count: usize) -> Option<NonNull<T>> {
        unsafe { self.stack.allocate_uninit(count) }
    }
}

impl<'stack, 'mem> Drop for StackGuard<'stack, 'mem> {

    fn drop(&mut self) {
        self.stack.pos = self.pos_rollback;
    }
}

pub struct StackRegion<'mem> {
    data: NonNull<u8>,
    size: usize,
    pos: usize,
    _marker: PhantomData<&'mem mut NonNull<u8>>,
}

impl<'mem> StackRegion<'mem> {

    pub fn new<'stack>(size: usize, stack: &'stack mut StackAllocator<'mem>) -> Option<Self>
    {
        let ptr = unsafe { stack.allocate_raw(size, align_of::<usize>())? };
        Some(
            Self {
                data: ptr,
                size,
                pos: 0,
                _marker: PhantomData,
            }
        )
    }

    pub fn clear(&mut self) {
        self.pos = 0;
    }
}

impl<'mem> AllocateExt<'mem> for StackRegion<'mem> {

    unsafe fn allocate_raw(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let start = self.data.as_ptr() as usize + self.pos;
        let aligned_start = (start + align - 1) & !(align - 1);
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

    unsafe fn allocate_uninit<T>(&mut self, count: usize) -> Option<NonNull<T>> {
        let size = std::mem::size_of::<T>() * count;
        let align = std::mem::align_of::<T>();
        unsafe { self.allocate_raw(size, align).map(|ptr| ptr.cast::<T>()) }
    }
}
