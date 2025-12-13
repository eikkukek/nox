use core::{
    ptr::{self, NonNull},
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
};

use crate::{Allocator, global_alloc::GlobalAlloc};

use super::{Dyn, DynRawParts};

union Data<T: ?Sized> {
    non_null: NonNull<T>,
    raw_parts: (*const (), *const ()),
}

impl<T: ?Sized> Data<T> {

    pub fn non_null(non_null: NonNull<T>) -> Self {
        Self {
            non_null,
        }
    }

    pub fn just_vtable(vtable: *const ()) -> Self {
        Self {
            raw_parts: (ptr::null(), vtable),
        }
    }
}

pub struct Owned<T: ?Sized> {
    data: Data<T>,
}

impl<T: ?Sized> Owned<T> {

    pub fn new<Source>(t: Source) -> Self
        where 
            Source: Dyn<Target = T>
    {
        unsafe {
            let vtable = t
                .raw_parts().vtable;
            if size_of::<Source>() == 0 {
                let _ = MaybeUninit::new(t);
                return Self {
                    data: Data::just_vtable(vtable),
                }
            }
            let data = GlobalAlloc
                .allocate_uninit(1)
                .expect("global alloc failed");
            data.write(t);
            Self {
                data: Data::non_null(NonNull::from_mut(Source::from_raw_parts_mut(
                    DynRawParts { data: data.as_ptr(), vtable }
                )))
            }
        }
    }

    /// If the vtable of [Source] matches the vtable of [T], this function moves the value
    /// and returns [Source], otherwise it returns [None].
    pub unsafe fn take_value<Source>(self) -> Option<Source>
        where
            T: 'static,
            Source: Dyn<Target = T>,
    {
        unsafe {
            let uninit = MaybeUninit::<Source>::uninit();
            let vtable = uninit.assume_init_ref().raw_parts().vtable;
            if self.data.raw_parts.1 != vtable {
                return None
            }
            if self.data.raw_parts.0.is_null() {
                let mut unit = ();
                let value = mem
                    ::transmute::<*mut (), *const Source>(&mut unit)
                    .read();
                Some(value)
            } else {
                let non_null = self.data.non_null;
                let value = Source::read_self(non_null.as_ptr());
                let size = size_of_val(&value);
                let align = align_of_val(&value);
                GlobalAlloc.free_raw(
                    non_null.cast(),
                    size,
                    align
                );
                let _ = MaybeUninit::new(self);
                Some(value)
            }
        }
    }
}

impl<T: ?Sized> Deref for Owned<T> {

    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            if self.data.raw_parts.0.is_null() {
                let addr = &self.data;
                let data = Data {
                    raw_parts: (addr as *const _ as *const (), self.data.raw_parts.1),
                };
                data.non_null.as_ref()
            } else {
                self.data.non_null.as_ref()
            }
        }
    }
}

impl<T: ?Sized> DerefMut for Owned<T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            if self.data.raw_parts.0.is_null() {
                let addr = &self.data;
                let mut data = Data {
                    raw_parts: (addr as *const _ as *const (), self.data.raw_parts.1),
                };
                data.non_null.as_mut()
            } else {
                self.data.non_null.as_mut()
            }
        }
    }
}

impl<T: ?Sized> Drop for Owned<T> {

    fn drop(&mut self) {
        unsafe {
            if self.data.raw_parts.0.is_null() {
                let addr = &self.data;
                let data = Data::<T> {
                    raw_parts: (addr as *const _ as *const (), self.data.raw_parts.1),
                };
                data.non_null.drop_in_place();
            } else {
                let data_mut = self.deref_mut();
                let size = size_of_val(data_mut);
                let align = align_of_val(data_mut);
                let ptr = self.data.non_null.cast();
                self.data.non_null.drop_in_place();
                GlobalAlloc.free_raw(
                    ptr,
                    size,
                    align,
                );
            }
        }
    }
}

unsafe impl<T: ?Sized + Send> Send for Owned<T> {}
unsafe impl<T: ?Sized + Sync> Sync for Owned<T> {}
