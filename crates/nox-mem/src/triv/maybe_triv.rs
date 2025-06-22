use super::Triv;

pub unsafe trait MaybeTriv: Sized {

    fn is_triv() -> bool;

    fn as_triv(&self) -> &Triv<Self>;

    fn as_mut_triv(&mut self) -> &mut Triv<Self>;
}

unsafe impl<T> MaybeTriv for T {

    default fn is_triv() -> bool {
        false
    }

    default fn as_triv(&self) -> &Triv<Self> {
        panic!("not a triv")
    }

    default fn as_mut_triv(&mut self) -> &mut Triv<Self> {
        panic!("not a triv")
    }
}

macro_rules! impl_triv {
    ($($t:ty), *) => {
        $(

            unsafe impl crate::triv::MaybeTriv for $t {

                fn is_triv() -> bool { true }

                fn as_triv(&self) -> &Triv<Self> {
                    unsafe {
                        & *(self as *const Self as *const Triv<Self>)
                    }
                }

                fn as_mut_triv(&mut self) -> &mut Triv<Self> {
                    unsafe {
                        &mut *(self as *mut Self as *mut Triv<Self>)
                    }
                }
            }
        )*
    };
}

pub(crate) use impl_triv;

impl_triv!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

unsafe impl<T, const N: usize> MaybeTriv for [T; N] {

    fn is_triv() -> bool { true }

    fn as_triv(&self) -> &Triv<Self> {
        unsafe {
            & *(self as *const Self as *const Triv<Self>)
        }
    }

    fn as_mut_triv(&mut self) -> &mut Triv<Self> {
        unsafe {
            &mut *(self as *mut Self as *mut Triv<Self>)
        }
    }
}
