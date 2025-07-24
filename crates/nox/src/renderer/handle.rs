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

    pub fn new(handle: T) -> Self {
        Self {
            handle,
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

pub struct RaiiHandle<'a, T, F: FnMut(T)> {
    handle: Option<T>,
    drop: F,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T, F: FnMut(T)> RaiiHandle<'a, T, F> {

    pub fn new(handle: T, drop: F) -> Self {
        Self {
            handle: Some(handle),
            drop,
            _marker: PhantomData,
        }
    }

    pub fn into_inner(mut self) -> T {
        self.handle.take().unwrap()
    }
}

impl<'a, T, F: FnMut(T)> Deref for RaiiHandle<'a, T, F> {

    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.handle.as_ref().unwrap()
    }
}

impl<'a, T, F: FnMut(T)> DerefMut for RaiiHandle<'a, T, F> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.handle.as_mut().unwrap()
    }
}

impl<'a, T, F: FnMut(T)> Drop for RaiiHandle<'a, T, F> {

    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            (self.drop)(handle);
        }
    }
}
