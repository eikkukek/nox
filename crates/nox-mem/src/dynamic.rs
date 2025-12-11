mod pair;

pub use pair::Pair;

pub use nox_derive::Dyn;

pub struct DynRawParts<T: Sized> {
    pub data: *const T,
    pub vtable: *const (),
}

pub unsafe trait Dyn: Sized + 'static {

    type Target: ?Sized;

    unsafe fn raw_parts(&self) -> DynRawParts<Self>;

    unsafe fn from_raw_parts<'a>(raw_parts: DynRawParts<Self>) -> &'a Self::Target;
}
