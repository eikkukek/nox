mod owned;
mod pair;

use core::{
    any::Any,
    mem::{self, MaybeUninit},
};

pub use owned::{Owned, OwnedMaybeUninit};
pub use pair::Pair;

pub use nox_proc::Dyn;

pub struct DynRawParts<T: Sized> {
    pub data: *const T,
    pub vtable: *const (),
}

/// You should not implement this trait manually, but use the derive macro for [`Dyn`].
pub unsafe trait Dyn: Sized {

    type Target: ?Sized;

    unsafe fn raw_parts(ptr: *const Self) -> DynRawParts<Self>;

    fn raw_parts_uninit() -> DynRawParts<Self> {
        unsafe {
            let value = MaybeUninit::uninit();
            Self::raw_parts(value.as_ptr())
        }
    }

    unsafe fn from_raw_parts(raw_parts: DynRawParts<Self>) -> *const Self::Target;
    
    /// This function is extremely unsafe. It has to be guaranteed that the pointer passed to this
    /// function originated from `Self`.
    unsafe fn get_self(target: *const Self::Target) -> *const Self;
}

pub trait DynFn: Dyn {

    type Args<'a>;
    type Output;

    unsafe fn call_once<'a>(target: Owned<Self::Target>, args: Self::Args<'a>)
        -> Self::Output;
}

pub struct NonDyn<T>(pub T);

unsafe impl<T: 'static> Dyn for NonDyn<T> {

    type Target = T;

    unsafe fn raw_parts(ptr: *const Self) -> DynRawParts<Self> {
        let any: *const dyn Any = ptr;
        let vtable = unsafe {
            mem::transmute::<
                *const dyn Any,
                (*const(), *const())
            >(any).1
        };
        DynRawParts {
            data: ptr,
            vtable,
        }
    }

    unsafe fn from_raw_parts(raw_parts: DynRawParts<Self>) -> *const Self::Target {
        raw_parts.data.cast()
    }
    
    unsafe fn get_self(target: *const Self::Target) -> *const Self {
        target.cast()
    }
}
