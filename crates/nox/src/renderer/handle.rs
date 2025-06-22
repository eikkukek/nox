use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Clone)]
pub struct Handle<'a, T> {
    handle: T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Handle<'a, T> {

    pub fn new(h: T) -> Self {
        Self {
            handle: h,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Deref for Handle<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<'a, T> DerefMut for Handle<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}
