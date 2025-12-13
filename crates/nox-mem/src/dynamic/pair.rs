use core::{
    ptr::NonNull,
};

use crate::{
    global_alloc::GlobalAlloc,
    Allocator,
    const_fn::align_up,
};

use super::{Dyn, DynRawParts};

pub struct Pair<T: ?Sized, U: ?Sized> {
    t: NonNull<T>,
    u: NonNull<U>,
    a: NonNull<Allocation>,
}

struct Allocation {
    ptr: NonNull<u8>,
    size: usize,
    align: usize,
}

impl<T: ?Sized, U: ?Sized> Pair<T, U> {

    pub fn new<TSource, USource>(t: TSource, u: USource) -> Self
        where
            TSource: Dyn<Target = T>,
            USource: Dyn<Target = U>,
    {
        let t_vtable = unsafe {
            t.raw_parts()
        }.vtable;
        let u_vtable = unsafe {
            u.raw_parts()
        }.vtable;
        let t_align = align_of_val(&t);
        let u_align = align_of_val(&u);
        let alloc_align = align_of::<Allocation>();
        let mut align = alloc_align;
        let mut size = 0;
        let (t_off, u_off) =
        if t_align > u_align {
            align = t_align.max(align);
            size += size_of_val(&t);
            size = align_up(size, u_align);
            let u_off = size;
            size += size_of_val(&u);
            (0, u_off)
        } else {
            align = u_align.max(align);
            size += size_of_val(&u);
            size = align_up(size, t_align);
            let t_off = size;
            size += size_of_val(&t);
            (t_off, 0)
        };
        size = align_up(size, alloc_align);
        let alloc_off = size;
        size += size_of::<Allocation>();
        let ptr = unsafe {
            GlobalAlloc
                .allocate_raw(size, align)
                .expect("global alloc failed")
        };
        let t_ptr = unsafe {
            let ptr = ptr.add(t_off).cast();
            ptr.write(t);
            ptr
        };
        let u_ptr = unsafe {
            let ptr = ptr.add(u_off).cast();
            ptr.write(u);
            ptr
        };
        let t = NonNull::from_mut(unsafe {
            TSource::from_raw_parts_mut(DynRawParts {
                data: t_ptr.as_ptr(),
                vtable: t_vtable,
            })
        });
        let u = NonNull::from_mut(unsafe {
            USource::from_raw_parts_mut(DynRawParts {
                data: u_ptr.as_ptr(),
                vtable: u_vtable,
            })
        });
        let a = unsafe {
            let _ptr = ptr.add(alloc_off).cast();
            _ptr.write(Allocation { ptr, size, align });
            _ptr
        };
        Self {
            t,
            u,
            a
        }
    }

    pub fn first(&self) -> &T {
        unsafe {
            self.t.as_ref()
        }
    }

    pub fn first_mut(&mut self) -> &mut T {
        unsafe {
            self.t.as_mut()
        }
    }

    pub fn second(&self) -> &U {
        unsafe {
            self.u.as_ref()
        }
    }

    pub fn second_mut(&mut self) -> &mut U {
        unsafe {
            self.u.as_mut()
        }
    }
}

impl<T: ?Sized, U: ?Sized> Drop for Pair<T, U> {

    fn drop(&mut self) {
        unsafe {
            self.t.drop_in_place();
            self.u.drop_in_place();
            let a = self.a.read();
            GlobalAlloc.free_raw(
                a.ptr,
                a.size,
                a.align,
            );
        }
    }
}

unsafe impl<T: ?Sized + Send, U: ?Sized + Send> Send for Pair<T, U> {}

unsafe impl<T: ?Sized + Sync, U: ?Sized + Sync> Sync for Pair<T, U> {}
