//! Provides [`Context`] trait which is implemented for any [`Result<T, E>`] where
//! ```rust
//! E: core::error::Error + Send + Sync + 'static
//! ```

use core::{
    error,
    result,
    fmt::Display,
};

use super::{Error, Tracked, BuildInternal, Result, Location, caller};

pub trait Context<T, E>
    where
        E: error::Error + Send + Sync + 'static,
{

    fn context<C>(self, ctx: C) -> Result<T>
        where C: Display + Send + Sync + 'static;

    fn context_with<C>(self, f: impl FnMut() -> C) -> Result<T>
        where C: Display + Send + Sync + 'static;

    /// Add context from a error with tracked location.
    ///
    /// # Example
    /// ``` rust
    ///
    /// use nox_error::Error;
    /// use nox_error::Context;
    /// use nox_error::Result;
    ///
    /// fn do_thing() -> Result<()> {
    ///     Err(Error::just_ctx("failed"))
    /// }
    ///
    /// fn call_do_thing() -> Result<()> {
    ///     do_thing().context_from_origin(|orig|
    ///         format!("error origin: {orig}")
    ///     )?;
    ///     Ok(())
    /// }
    /// ```
    fn context_from_origin<C>(self, f: impl FnMut(Location) -> C) -> Result<T>
        where
            C: Display + Send + Sync + 'static,
            E: Tracked;
}

impl<T, E: error::Error + Send + Sync + 'static> Context<T, E> for result::Result<T, E> {
    
    #[track_caller]
    fn context<C>(self, ctx: C) -> Result<T>
        where C: Display + Send + Sync + 'static,
    {
        let loc = caller!();
        self.map_err(|err| {
            Error::new_internal(ctx, err, loc)
        })
    }

    #[track_caller]
    fn context_with<C>(self, mut f: impl FnMut() -> C) -> Result<T>
        where C: Display + Send + Sync + 'static,
    {
        let loc = caller!();
        self.map_err(|err| {
            Error::new_internal(f(), err, loc)
        })
    }

    #[track_caller]
    fn context_from_origin<C>(self, mut f: impl FnMut(Location) -> C) -> Result<T>
        where
            C: Display + Send + Sync + 'static,
            E: Tracked, 
    {
        let loc = caller!();
        self.map_err(|err| {
            Error::new_internal(f(err.loc()), err, loc)
        })
    }
}

