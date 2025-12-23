use core::{
    ptr::NonNull,
    mem,
};

use nox_mem::Allocator;
use nox_mem::{
    align_up,
    GlobalAlloc,
};

use nox_error::{Context, Tracked};

use crate::{win, gpu};

use super::pub_error;
use super::dev_error;

trait DynAny {}

impl<T> DynAny for T {}

pub(super) struct Data<'a> {
    data: NonNull<u8>,
    size: usize,
    align: usize,
    init: NonNull<dyn FnMut(
        &mut win::WindowContext,
        &mut gpu::GpuContext,
        *mut (dyn DynAny + 'a),
    ) -> dev_error::Result<()>>,
    t: NonNull<dyn DynAny + 'a>,
}

impl<'a> Data<'a> {

    fn make_f<T>(
        f: impl FnOnce(
            &mut win::WindowContext,
            &mut gpu::GpuContext,
        ) -> pub_error::Result<T>
    ) -> impl FnMut(
            &mut win::WindowContext,
            &mut gpu::GpuContext,
            *mut (dyn DynAny + 'a)
        ) -> dev_error::Result<()>
    {
        let mut f = Some(f);
        move |win, gpu, ptr| {
            if let Some(f) = f.take() {
                unsafe {
                    let value = f(win, gpu)
                        .context_from_tracked(|orig|
                            dev_error::ErrorContext::InitError(
                                orig.or_this()
                            )
                        )?;
                    let ptr = mem::transmute::<
                        *mut (dyn DynAny + 'a),
                        (*mut T, *const ())
                    >(ptr).0;
                    ptr.write(value);
                }
            }
            Ok(())
        }
    }

    pub fn new<T, F>(
        f: F
    ) -> Self
        where 
            T: 'a,
            F: FnOnce(
                &mut win::WindowContext,
                &mut gpu::GpuContext,
            ) -> pub_error::Result<T>,
    {
        let f = Self::make_f(f);
        let f_align = align_of_val(&f);
        let t_align = align_of::<T>();
        let (size, align, f_off, t_off) =
            if f_align > t_align {
                let mut size = size_of_val(&f);
                size = align_up(size, t_align);
                let t_off = size;
                size += size_of::<T>();
                (size, f_align, 0, t_off)
            } else {
                let mut size = size_of::<T>();
                size = align_up(size, f_align);
                let f_off = size;
                size += size_of_val(&f);
                (size, t_align, f_off, 0)
            };
        let data = unsafe { GlobalAlloc
            .allocate_raw(size, align)
            .unwrap()
            .cast()
        };
        let init = unsafe {
            let s: *const dyn FnMut(
                &mut win::WindowContext,
                &mut gpu::GpuContext,
                *mut (dyn DynAny + 'a)
            ) -> dev_error::Result<()> = &f;
            let vtable = mem::transmute::<_, (*const(), *const())>(s).1;
            let ptr = data.add(f_off).cast();
            ptr.write(f);
            let raw_parts = (ptr.as_ptr(), vtable);
            mem::transmute::<
                _,
                NonNull<dyn FnMut(
                    &mut win::WindowContext,
                    &mut gpu::GpuContext,
                    *mut (dyn DynAny + 'a),
                ) -> dev_error::Result<()>>
            >(raw_parts)
        };
        let t = unsafe {
            let ptr: *mut (dyn DynAny + 'a) = data
                .add(t_off)
                .cast::<T>()
                .as_ptr();
            NonNull::new(ptr).unwrap()
        };
        Self {
            data,
            size,
            align,
            init,
            t,
        }
    }

    pub unsafe fn get_t<T>(&self)-> *mut T {
        unsafe {
            mem::transmute::<
                *mut dyn DynAny,
                (*mut T, *const())
            >(self.t.as_ptr()).0
        }
    }

    pub fn init(
        &mut self,
        win: &mut win::WindowContext,
        gpu: &mut gpu::GpuContext,
    ) -> dev_error::Result<()> {
        let init = unsafe {
            self.init.as_mut()
        };
        (init)(win, gpu, self.t.as_ptr())?;
        Ok(())
    }

    pub unsafe fn drop_t(&mut self) {
        unsafe {
            self.t.drop_in_place();
        }
    }
}

impl<'a> Drop for Data<'a> {

    fn drop(&mut self) {
        unsafe {
            self.init.drop_in_place();
            GlobalAlloc.free_raw(
                self.data,
                self.size,
                self.align
            );
        }
    }
}
