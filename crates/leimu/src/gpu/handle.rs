use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    fmt::{self, Display},
};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TransientHandle<'a, T> {
    handle: T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> TransientHandle<'a, T> {

    #[inline(always)]
    pub fn new(handle: T) -> Self {
        Self {
            handle,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.handle
    }
}

impl<'a, T> Deref for TransientHandle<'a, T>
{

    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<'a, T> DerefMut for TransientHandle<'a, T>
{

    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl<T> Display for TransientHandle<'_, T>
    where T: Display
{

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.handle.fmt(f)
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
