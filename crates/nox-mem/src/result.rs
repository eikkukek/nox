pub trait ResultExt<T, E> {

    /// Filters out errors with the given predicate.
    ///
    /// Note that for this to work, a success value needs to be provided by the predicate.
    fn filter_err<F>(self, f: F) -> Self
        where F: FnOnce(&E) -> Option<T>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {

    #[inline(always)]
    fn filter_err<F>(self, f: F) -> Self
        where F: FnOnce(&E) -> Option<T>
    {
        match self {
            Self::Ok(s) => Ok(s),
            Self::Err(e) => {
                if let Some(s) = f(&e) {
                    Ok(s)
                } else {
                    Err(e)
                }
            },
        }
    }
}
