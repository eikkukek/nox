mod data;
mod r#ref;

use core::cell::UnsafeCell;

use crate::{win, gpu};

use crate::error as pub_error;
use crate::dev::error as dev_error;

use nox_mem::vec_types::GlobalVec;

use data::Data;
pub use r#ref::Ref;

pub struct OnInit<'a> {
    data: UnsafeCell<GlobalVec<Data<'a>>>,
    initialized: UnsafeCell<bool>,
}

impl<'a> OnInit<'a> {

    pub(crate) fn new() -> Self {
        Self {
            data: Default::default(),
            initialized: UnsafeCell::new(false),
        }
    }

    pub fn add<T: 'a>(
        &self,
        f: impl FnOnce(&mut win::WindowContext, &mut gpu::GpuContext) -> pub_error::Result<T> + 'a,
    ) -> Ref<'_, T> {
        unsafe {
            assert!(!*self.initialized.get(), "OnInit can't be reused");
        }
        let data = unsafe {
            &mut *self.data.get()
        };
        let data = data.push(Data::new(f));
        let t = unsafe {
            data.get_t()
        };
        Ref {
            initialized: unsafe {
                &*self.initialized.get()
            },
            t,
        }
    }

    pub(crate) fn init(
        &self,
        win: &mut win::WindowContext,
        gpu: &mut gpu::GpuContext,
    ) -> dev_error::Result<()> {
        unsafe {
            if *self.initialized.get() {
                return Err(dev_error::Error
                    ::just_context("OnInit can't be reused")
                )
            }
        }
        for data in unsafe { &mut *self.data.get() } {
            data.init(win, gpu)?;
        }
        unsafe {
            *self.initialized.get() = true;
        }
        Ok(())
    }
}

impl<'a> Drop for OnInit<'a> {

    fn drop(&mut self) {
        unsafe {
            if *self.initialized.get() {
                for data in &mut *self.data.get() {
                    data.drop_t();
                }
            }
        }
    }
}
