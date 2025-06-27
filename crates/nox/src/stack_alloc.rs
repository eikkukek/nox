use std::alloc::{alloc, dealloc, Layout};

use core::{
    mem,
    cell::UnsafeCell,
    ptr::NonNull,
};

use nox_mem::Allocator;

use super::{
    utility::next_align,
};

struct Cell {
    pos: usize,
    guard_active: bool,
}

pub struct StackAlloc {
    data: NonNull<u8>,
    size: usize,
    cell: UnsafeCell<Cell>,
}

impl StackAlloc {

    pub fn new(size: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, mem::align_of::<usize>()).ok()?;
        let ptr = unsafe { alloc(layout) };
        Some(
            Self {
                data: NonNull::new(ptr)?,
                size,
                cell: UnsafeCell::new(Cell {
                    pos: 0,
                    guard_active: false,
                }),
            }
        )
    }
    
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline(always)]
    pub fn used(&self) -> usize {
        unsafe { self.cell().pos }
    }

    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.size - self.used()
    }

    #[inline(always)]
    pub fn full(&self) -> bool {
        self.used() >= self.size
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        let cell = self.cell.get_mut();
        assert!(cell.guard_active == false, "attempting to clear while guard is active");
        cell.pos = 0;
    }

    #[inline(always)]
    pub unsafe fn force_clear(&self) {
        unsafe {
            let cell = self.cell();
            assert!(cell.guard_active == false, "attempting to clear while guard is active");
            cell.pos = 0;
        }
    }

    #[inline(always)]
    unsafe fn cell(&self) -> &mut Cell {
        unsafe { &mut *self.cell.get() }
    }

    #[inline(always)]
    unsafe fn allocate_raw_internal(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let start = self.data.as_ptr() as usize + self.used();
        let aligned_start = next_align(start, align);
        let end = aligned_start + size;
        if end > self.data.as_ptr() as usize + self.size {
            return None
        }
        unsafe { self.cell().pos = end - self.data.as_ptr() as usize; }
        Some(
            unsafe {
                NonNull::new_unchecked(aligned_start as *mut u8)
            }
        )
    }
}

impl Allocator for StackAlloc {

    #[inline(always)]
    unsafe fn allocate_raw(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        unsafe {
            assert!(self.cell().guard_active == false, "attempting to allocate while guard is active");
            self.allocate_raw_internal(size, align)
        }
    }

    #[inline(always)]
    unsafe fn free_raw(&self, _ptr: NonNull<u8>, _size: usize, _align: usize) {}
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
    stack: &'alloc StackAlloc,
}

impl<'alloc> StackGuard<'alloc> {

    #[inline(always)]
    pub fn new(stack: &'alloc StackAlloc) -> Self {
        unsafe {
            let cell = stack.cell();
            assert!(cell.guard_active == false, "attempting to create concurrent guards");
            cell.guard_active = true;
        }
        let pos_rollback = stack.used();
        Self {
            stack,
            pos_rollback,
        }
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.stack.size()
    }

    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.stack.remaining()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        unsafe {
            self.stack.cell().pos = self.pos_rollback;
        }
    }
}

impl<'alloc> Allocator for StackGuard<'alloc> {

    #[inline(always)]
    unsafe fn allocate_raw(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        unsafe { self.stack.allocate_raw_internal(size, align) }
    }

    #[inline(always)]
    unsafe fn free_raw(&self, _ptr: NonNull<u8>, _size: usize, _align: usize) {}
}

impl<'alloc> Drop for StackGuard<'alloc> {

    fn drop(&mut self) {
        unsafe {
            let cell = self.stack.cell();
            cell.pos = self.pos_rollback;
            cell.guard_active = false;
        }
    }
}
