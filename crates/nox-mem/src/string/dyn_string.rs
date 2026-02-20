use core::{
    cell::RefCell,
    slice,
};

use crate::allocator_traits::{Allocate, Free};

pub struct DynString<'alloc, Alloc>
    where
        Alloc: Allocate + Free
{
    data: *mut u8,
    size: usize,
    len: usize,
    alloc: &'alloc RefCell<Alloc>,
}

impl<'alloc, Alloc> DynString<'alloc, Alloc>
    where
        Alloc: Allocate + Free
{
    pub fn new(alloc: &'alloc RefCell<Alloc>) -> Self {
        Self {
            data: std::ptr::dangling::<u8>() as _,
            size: 0,
            len: 0,
            alloc,
        }
    }

    pub fn from_str(s: &str, alloc: &'alloc RefCell<Alloc>) -> Option<Self> {
        let len = s.len();
        let size = len.next_power_of_two();
        let data = unsafe {
            alloc
                .borrow_mut()
                .allocate_uninit::<u8>(size)?
                .as_ptr()
        };
        unsafe { data.copy_from(s.as_ptr(), len) };
        Some(Self {
            data,
            size,
            len,
            alloc,
        })
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            core::str::from_utf8(
                slice::from_raw_parts(self.data, self.len))
                .unwrap_or("<invalid utf-8>")
        }
    }
}
