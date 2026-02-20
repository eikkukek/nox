#[cfg(feature = "std")]
mod owned;
#[cfg(feature = "std")]
mod pair;

use core::{
    any::Any,
    mem::{self, MaybeUninit},
};

#[cfg(feature = "std")]
pub use owned::{Owned, OwnedMaybeUninit};
#[cfg(feature = "std")]
pub use pair::Pair;

pub use nox_proc::Dyn;

#[repr(C)]
pub struct DynRawParts<T: Sized> {
    pub data: *const T,
    pub meta: *const (),
}

/// A trait for handling dynamic objects.
/// # Safety
/// You should not implement this trait manually, but use the derive macro for [`Dyn`] instead.
pub unsafe trait Dyn<T: ?Sized> {

    type Target: Sized;

    /// Extracts the raw parts (pointer to [`Self`] and pointer to meta) of [`Self`].
    /// # Safety
    /// The pointer must a be a valid, aligned pointer to [`Self`].
    unsafe fn raw_parts(ptr: *const Self) -> DynRawParts<Self::Target>;

    /// Gets the meta of [`Self`].
    fn meta() -> *const()
        where Self: Sized
    {
        unsafe {
            let value = MaybeUninit::uninit();
            Self::raw_parts(value.as_ptr())
                .meta
        }
    }

    /// Constructs [`T`] from [`DynRawParts<Self>`].
    /// # Safety
    /// `raw_parts` must be a valid [`DynRawParts<Self>`] containing a valid, aligned pointer to
    /// [`Self`] and a pointer to the correct meta.
    unsafe fn from_raw_parts(raw_parts: DynRawParts<Self::Target>) -> *const T;
   
    /// Constructs a pointer to [`Self`] from a pointer [`T`].
    /// # Safety
    /// It has to be guaranteed that the pointer passed to this function originated from [`Self`].
    unsafe fn get_self(target: *const T) -> *const Self;
}

pub struct NonDyn<T>(pub T);

unsafe impl<T: 'static> Dyn<T> for NonDyn<T> {

    type Target = T;

    unsafe fn raw_parts(ptr: *const Self) -> DynRawParts<Self::Target> {
        let any: *const dyn Any = ptr;
        let meta = unsafe {
            mem::transmute::<
                *const dyn Any,
                (*const(), *const())
            >(any).1
        };
        DynRawParts {
            data: ptr.cast(),
            meta,
        }
    }

    unsafe fn from_raw_parts(raw_parts: DynRawParts<Self::Target>) -> *const T {
        raw_parts.data.cast()
    }
    
    unsafe fn get_self(target: *const T) -> *const Self {
        target.cast()
    }
}

unsafe impl<T> Dyn<[T]> for [T] {

    type Target = T;

    unsafe fn raw_parts(ptr: *const Self) -> DynRawParts<Self::Target> {
        unsafe {
            mem::transmute::<*const Self, DynRawParts<Self::Target>>(ptr)
        }
    }

    unsafe fn from_raw_parts(raw_parts: DynRawParts<Self::Target>) -> *const [T] {
        unsafe {
            mem::transmute::<DynRawParts<Self::Target>, *const Self>(raw_parts)
        }
    }

    unsafe fn get_self(target: *const [T]) -> *const Self {
        target
    }
}

/*
pub trait DynFn: Dyn {

    type Args<'a>;
    type Output;

    unsafe fn call_once<'a>(target: Owned<Self::Target>, args: Self::Args<'a>) -> Self::Output;
}
*/
