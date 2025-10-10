use core::{
    mem::needs_drop,
    ptr::NonNull,
    ops::{Deref, DerefMut},
};

use crate::impl_traits;

#[derive(Eq)]
pub struct Pointer<T: Sized>(NonNull<T>);

impl<T: Sized> Copy for Pointer<T> {}

impl<T: Sized> Clone for Pointer<T> {

    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Pointer<T> {

    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Sized> Pointer<T> {

    #[inline(always)]
    pub fn dangling() -> Self {
        Self(NonNull::dangling())
    }

    #[inline(always)]
    pub fn new(ptr: *mut T) -> Option<Self> {
        Some(Self(NonNull::new(ptr)?))
    }

    #[inline(always)]
    pub unsafe fn add(&self, count: usize) -> Self {
        unsafe {
            self.0.add(count).into()
        }
    }

    #[inline(always)]
    pub unsafe fn sub(&self, count: usize) -> Self {
        unsafe {
            self.0.sub(count).into()
        }
    }

    #[inline(always)] 
    pub unsafe fn move_elements(&self, to: Self, len: usize) {
        unsafe {
            self.copy_to_nonoverlapping(*to, len);
        }
    }

    #[inline(always)]
    pub unsafe fn insert_element(&self, value: T, index: usize, len: usize) -> Pointer<T> {
        unsafe {
            let res = self.add(index);
            res.copy_to(*res.add(1), len - index);
            res.write(value);
            res
        }
    }

    #[inline(always)]
    pub unsafe fn drop_in_place(&self, len: usize) {
        if needs_drop::<T>() {
            unsafe {
                for i in 0..len {
                    self.add(i).deref().drop_in_place();
                }
            }
        }
    }

    #[inline(always)]
    pub unsafe fn clone_elements(&self, to: Self, len: usize)
        where
            T: Clone
    {
        if needs_drop::<T>() {
            unsafe {
                for i in 0..len {
                    to.add(i).write(self.add(i).as_ref().clone());
                }
            }
        }
        else {
            unsafe {
                self.copy_to_nonoverlapping(*to, len);
            }
        }
    }
}

impl_traits!{
    for Pointer<T>
    Deref =>

        type Target = NonNull<T>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    ,
    DerefMut =>

        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    ,
    From<NonNull<T>> =>
        
        fn from(value: NonNull<T>) -> Self {
            Self(value)
        }
    ,
}
