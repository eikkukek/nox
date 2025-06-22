use core::{
    cell::UnsafeCell,
    marker::PhantomData,
};

/// A marker trait for types that can be safely read/written as raw bytes.
/// # Safety
/// Only implement for types that:
/// - Are `#[repr(C)]` or `#[repr(transparent)]`
/// - Can be zeroed
/// - Don't implement [`Drop`]
/// - Are [`Copy`]
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

#[repr(transparent)]
pub struct Pod<'a, T>(UnsafeCell<T>, PhantomData<&'a ()>);

pub unsafe trait MaybePod: Sized {

    fn is_pod() -> bool;

    fn as_pod(&self) -> &Pod<'_, Self>;

    fn as_mut_pod(&mut self) -> &mut Pod<'_, Self>;
}

unsafe impl<T> MaybePod for T {

    default fn is_pod() -> bool {
        false
    }

    default fn as_pod(&self) -> &Pod<'_, Self> {
        panic!("not a pod")
    }

    default fn as_mut_pod(&mut self) -> &mut Pod<'_, Self> {
        panic!("not a pod")
    }
}

#[macro_export]
macro_rules! impl_pod {
    ($($t:ty), *) => {
        $(

            unsafe impl crate::pod::MaybePod for $t {

                fn is_pod() -> bool { true }

                fn as_pod(&self) -> &Pod<'_, Self> {
                    unsafe {
                        & *(self as *const Self as *const Pod<'_, Self>)
                    }
                }

                fn as_mut_pod(&mut self) -> &mut Pod<'_, Self> {
                    unsafe {
                        &mut *(self as *mut Self as *mut Pod<'_, Self>)
                    }
                }
            }
        )*
    };
}

pub(crate) use impl_pod;

impl_pod!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

unsafe impl<T, const N: usize> MaybePod for [T; N] {

    fn is_pod() -> bool { true }

    fn as_pod(&self) -> &Pod<'_, Self> {
        unsafe {
            & *(self as *const Self as *const Pod<'_, Self>)
        }
    }

    fn as_mut_pod(&mut self) -> &mut Pod<'_, Self> {
        unsafe {
            &mut *(self as *mut Self as *mut Pod<'_, Self>)
        }
    }
}

pub fn is_pod<T>() -> bool {
    <T as MaybePod>::is_pod()
}
