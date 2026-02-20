use core::{
    ptr::NonNull,
    mem,
};

use nox_mem::{
    align_up,
    alloc::{LocalAlloc, StdAlloc, Layout},
};

use nox_error::{Context, Tracked};

use crate::{event_loop, gpu};

use super::pub_error;
use super::dev_error;

trait DynAny {}

impl<T> DynAny for T {}

type InitFnInternal<'a> = dyn FnMut(
    &event_loop::ActiveEventLoop,
    &mut gpu::GpuContext,
    *mut (dyn DynAny + 'a)
) -> dev_error::Result<()>;

pub(super) struct Data<'a> {
    data: NonNull<u8>,
    size: usize,
    align: usize,
    init: NonNull<InitFnInternal<'a>>,
    t: NonNull<dyn DynAny + 'a>,
}

impl<'a> Data<'a> {

    #[inline(always)]
    fn make_f<T, F>(
        f: F,
    ) -> impl FnMut(
        &event_loop::ActiveEventLoop,
        &mut gpu::GpuContext,
        *mut (dyn DynAny + 'a)
    ) -> dev_error::Result<()>
        where 
            F: FnOnce(
                &event_loop::ActiveEventLoop,
                &mut gpu::GpuContext,
            ) -> pub_error::Result<T>
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

    #[inline(always)]
    unsafe fn transmute_f<F>(
        f: *const F,
        vtable: *const ()
    ) -> NonNull<InitFnInternal<'a>>
        where
            F: FnMut(
                &event_loop::ActiveEventLoop,
                &mut gpu::GpuContext,
                *mut (dyn DynAny + 'a)
            ) -> dev_error::Result<()>
    {
        unsafe {
            mem::transmute::<(*const F, *const ()),NonNull<InitFnInternal<'a>>>(
                (f, vtable)
            )
        }
    }

    pub fn new<T, F>(
        f: F
    ) -> Self
        where 
            T: 'a,
            F: FnOnce(
                &event_loop::ActiveEventLoop,
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
        let data = unsafe { StdAlloc
            .allocate_raw(Layout::from_size_align(size, align).unwrap())
            .unwrap()
            .cast()
        };
        let init = unsafe {
            let s: *const dyn FnMut(
                &event_loop::ActiveEventLoop,
                &mut gpu::GpuContext,
                *mut (dyn DynAny + 'a),
            ) -> dev_error::Result<()> = &f;
            let vtable = mem::transmute::<
                *const dyn FnMut(
                    &event_loop::ActiveEventLoop,
                    &mut gpu::GpuContext,
                    *mut (dyn DynAny + 'a),
                ) -> dev_error::Result<()>,
                (*const(), *const())
            >(s).1;
            let ptr = data.add(f_off).cast();
            ptr.write(f);
            Self::transmute_f(ptr.as_ptr(), vtable)
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
        win: &event_loop::ActiveEventLoop,
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
            StdAlloc.free_raw(
                self.data,
                Layout::from_size_align(self.size, self.align).unwrap()
            );
        }
    }
}
