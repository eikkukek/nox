use core::{
    mem::needs_drop,
    ptr::NonNull,
    ops::{Deref, DerefMut},
};

use crate::impl_traits;

pub trait CloneStrategy<T: Sized> {
    unsafe fn clone_elements(&self, to: Self, len: usize);
}

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
            self.copy_to(*self.add(1), len - index);
            let res = self.add(index);
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

impl<T: Clone> CloneStrategy<T> for Pointer<T> {

    #[inline(always)]
    default unsafe fn clone_elements(&self, to: Self, len: usize) {
        unsafe {
            for i in 0..len {
                to.add(i).write(self.add(i).read().clone());
            }
        }
    }
}

impl<T: Copy> CloneStrategy<T> for Pointer<T> {

    #[inline(always)]
    unsafe fn clone_elements(&self, to: Self, len: usize) {
        unsafe {
            self.copy_to_nonoverlapping(*to, len);
        }
    }
}
