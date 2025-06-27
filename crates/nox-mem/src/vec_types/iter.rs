use core::{
    ptr::NonNull,
    marker::PhantomData,
};

use crate::conditional::{Conditional, True, False};

pub struct IterBase<'a, T, IsMut: Conditional> {
    ptr: NonNull<T>,
    end: NonNull<T>,
    _markers: PhantomData<(&'a T, IsMut)>,
}

impl<'a, T, IsMut: Conditional> IterBase<'a, T, IsMut> {

    #[inline(always)]
    pub unsafe fn new(ptr: NonNull<T>, end: NonNull<T>) -> Self {
        Self {
            ptr,
            end,
            _markers: PhantomData,
        }
    }
}

pub type Iter<'a, T> = IterBase<'a, T, False>;
pub type IterMut<'a, T> = IterBase<'a, T, True>;

impl<'a, T> Iterator for Iter<'a, T> {

    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            let item = unsafe { self.ptr.as_ref() };
            self.ptr = unsafe { self.ptr.add(1) };
            Some(item)
        }
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {

    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            self.end = unsafe { self.ptr.sub(1) };
            Some(unsafe { self.end.as_ref() })
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {

    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            let item = unsafe { self.ptr.as_mut() };
            self.ptr = unsafe { self.ptr.add(1) };
            Some(item)
        }
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {

    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            self.end = unsafe { self.ptr.sub(1) };
            Some(unsafe { self.end.as_mut() })
        }
    }
}
