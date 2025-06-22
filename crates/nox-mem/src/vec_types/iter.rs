use core::marker::PhantomData;

pub struct Iter<'a, T> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iter<'a, T> {

    #[inline(always)]
    pub unsafe fn new(ptr: *const T, end: *const T, marker: PhantomData<&'a T>) -> Self {
        Self {
            ptr,
            end,
            _marker: marker,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {

    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            let item = unsafe { &*self.ptr };
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
            Some(unsafe { &*self.end })
        }
    }
}

pub struct IterMut<'a, T> {
    ptr: *mut T,
    end: *mut T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> IterMut<'a, T> {

    #[inline(always)]
    pub unsafe fn new(ptr: *mut T, end: *mut T, marker: PhantomData<&'a T>) -> Self {
        Self {
            ptr,
            end,
            _marker: marker,
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
            let item = unsafe { &mut *self.ptr };
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
            Some(unsafe { &mut *self.end })
        }
    }
}
