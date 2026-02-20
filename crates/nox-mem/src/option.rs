pub trait OptionExt<T> {

    fn get_or_try_insert_with<E, F>(&mut self, f: F) -> Result<&mut T, E>
        where F: FnOnce() -> Result<T, E>;

    fn unwrap_or_try_else<F, E>(self, f: F) -> Result<T, E>
        where F: FnOnce() -> Result<T, E>;

    fn as_ptr(&self) -> *const T;
}

impl<T> OptionExt<T> for Option<T> {

    #[inline(always)]
    fn get_or_try_insert_with<E, F>(&mut self, f: F) -> Result<&mut T, E>
        where F: FnOnce() -> Result<T, E>
    {
        match self {
            Self::Some(t) => Ok(t),
            None => {
                let t = f()?;
                Ok(self.insert(t))
            }
        }
    }

    #[inline(always)]
    fn unwrap_or_try_else<F, E>(self, f: F) -> Result<T, E>
        where F: FnOnce() -> Result<T, E>
    {
        match self {
            Self::Some(t) => Ok(t),
            None => {
                let t = f()?;
                Ok(t)
            }
        }
    }

    #[inline(always)]
    fn as_ptr(&self) -> *const T {
        match self.as_ref() {
            Some(value) => value,
            None => core::ptr::null(),
        }
    }
}
