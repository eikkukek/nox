/// A marker trait for types that can be safely read/written as raw bytes.
/// # Safety
/// Only implement for types that:
/// - Don't implement [`Drop`]
/// - Are [`Copy`]
/// - Have no invalid values (e.g. bool with values other than 0 or 1)
/// - Are `#[repr(C)]` or `#[repr(transparent)]`
/// # Example (Macro Implementation)
/// ```
///
/// use nox::impl_pod;
///
/// #[repr(transparent)]
/// struct MyU32(u32);
///
/// #[repr(transparent)]
/// struct MyU64(u64);
/// 
/// impl_pod!(MyU32, MyU64);
/// ```
pub unsafe trait Pod {
    fn is_pod() -> bool;
}

unsafe impl<T> Pod for T {
    default fn is_pod() -> bool { false }
}

#[macro_export]
macro_rules! impl_pod {
    ($($t:ty), *) => {
        $(
            unsafe impl crate::pod::Pod for $t {
                fn is_pod() -> bool { true }
            }
        )*
    };
}
pub(crate) use impl_pod;

impl_pod!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

unsafe impl<T, const N: usize> Pod for [T; N] {
    fn is_pod() -> bool { T::is_pod() }
}

pub fn is_pod<T>() -> bool
{
    T::is_pod()
}
