pub(crate) trait PtrOption {

    type Ptr;

    fn as_ptr(&self) -> Self::Ptr;
}

impl<T> PtrOption for Option<&T> {

    type Ptr = *const T;

    fn as_ptr(&self) -> Self::Ptr {
        match self {
            Self::Some(x) => *x,
            None => core::ptr::null(),
        }
    }
}

impl<T> PtrOption for Option<&mut T> {

    type Ptr = *mut T;

    fn as_ptr(&self) -> Self::Ptr {
        match self {
            Self::Some(x) => <*const T>::cast_mut(*x),
            None => core::ptr::null_mut(),
        }
    }
}
