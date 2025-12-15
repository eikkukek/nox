use core::mem::MaybeUninit;

use nox_error::Context;

use crate::{win, gpu};

use crate::error as pub_error;
use crate::dev::error as dev_error;

pub struct OnInit<T, F>
    where
        F: FnOnce(
            &mut win::WindowContext,
            &mut gpu::GpuContext
        ) -> pub_error::Result<T>
{
    value: MaybeUninit<T>,
    f: Option<F>,
}

impl<T, F> OnInit<T, F>
    where
        F: FnOnce(
            &mut win::WindowContext,
            &mut gpu::GpuContext
        ) -> pub_error::Result<T>
{
    #[inline(always)]
    pub fn new(f: F) -> Self {
        Self {
            value: MaybeUninit::uninit(),
            f: Some(f),
        }
    }

    #[inline(always)]
    pub fn init(
        &mut self,
        win: &mut win::WindowContext,
        gpu: &mut gpu::GpuContext,
    ) -> dev_error::Result<&mut T>
    {
        let Some(f) = self.f.take() else {
            return Err(
                dev_error::Error::just_context("already initialized")
            )
        };
        Ok(self.value.write(f(win, gpu)
            .context("on init failed")?
        ))
    }

    #[inline(always)]
    pub fn get(&self) -> &T {
        if self.f.is_some() {
            panic!("value uninitialized")
        }
        unsafe {
            self.value.assume_init_ref()
        }
    }

    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        if self.f.is_some() {
            panic!("value uninitialized")
        }
        unsafe {
            self.value.assume_init_mut()
        }
    }
}

impl<T, F> Drop for OnInit<T, F>
    where
        F: FnOnce(
            &mut win::WindowContext,
            &mut gpu::GpuContext
        ) -> pub_error::Result<T>
{

    fn drop(&mut self) {
        if self.f.is_none() {
            unsafe {
                self.value.assume_init_drop();
            }
        }
    }
}
