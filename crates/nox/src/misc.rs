pub trait ToRef<T> {

    fn to_ref(&self) -> &T;
}

impl<T> ToRef<T> for T {

    fn to_ref(&self) -> &T {
        self
    }
}

impl<'a, T> ToRef<T> for &'a T {

    fn to_ref(&self) -> &T {
        self
    }
}
