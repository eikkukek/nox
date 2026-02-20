pub trait ToRef<T> {

    fn to_ref(&self) -> &T;
}

impl<T> ToRef<T> for T {

    #[inline(always)]
    fn to_ref(&self) -> &T {
        self
    }
}

impl<T> ToRef<T> for &T {

    #[inline(always)]
    fn to_ref(&self) -> &T {
        self
    }
}

impl<T> ToRef<T> for &mut T {

    #[inline(always)]
    fn to_ref(&self) -> &T {
        self
    }
}

pub const fn assert_send<T: Send>() {}
pub const fn assert_sync<T: Sync>() {}

#[macro_export]
macro_rules! assert_send {
    ($t:ty) => {
        const _: () = $crate::misc::assert_send::<$t>();
    };
}

#[macro_export]
macro_rules! assert_sync {
    ($t:ty) => {
        const _: () = $crate::misc::assert_sync::<$t>();
    };
}
