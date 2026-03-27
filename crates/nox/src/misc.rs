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

/// ``` rust
/// x |= y & (z as u32) << y.trailing_zeros()
/// ```
#[macro_export]
macro_rules! or_flag {
    ($flags:expr, $flag:expr, $value:expr $(,)?) => {
        $flags |= $flag & ($value as u32) << $flag.trailing_zeros();
    };
}
