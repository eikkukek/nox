mod owned;
mod pair;

pub use owned::Owned;
pub use pair::Pair;

pub use nox_proc::Dyn;

pub struct DynRawParts<T: Sized> {
    pub data: *const T,
    pub vtable: *const (),
}

pub struct DynRawPartsMut<T: Sized> {
    pub data: *const T,
    pub vtable: *const (),
}

pub unsafe trait Dyn: Sized + 'static {

    type Target: ?Sized;

    unsafe fn raw_parts(&self) -> DynRawParts<Self>;

    unsafe fn from_raw_parts_mut<'a>(raw_parts: DynRawParts<Self>) -> &'a mut Self::Target;

    #[inline(always)]
    unsafe fn from_raw_parts<'a>(raw_parts: DynRawParts<Self>) -> &'a Self::Target {
        unsafe {
            Self::from_raw_parts_mut(raw_parts)
        }
    }

    unsafe fn read_self(target: *mut Self::Target) -> Self;
}

pub trait DynFn: Dyn {

    type Args<'a>;
    type Output;

    unsafe fn call_once<'a>(target: Owned<Self::Target>, args: Self::Args<'a>) -> Self::Output;
}
