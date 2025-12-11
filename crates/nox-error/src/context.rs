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

    fn context_tracked<C>(self, ctx: C) -> Result<T>
        where C: Display + Send + Sync + 'static;

    fn context_with<C>(self, f: impl FnMut() -> C) -> Result<T>
        where C: Display + Send + Sync + 'static;

    fn context_tracked_with<C>(self, f: impl FnMut() -> C) -> Result<T>
        where C: Display + Send + Sync + 'static;

    /// Add context from an error with possibly tracked location.
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
    ///     do_thing().context_from_tracked(|orig|
    ///         format!("error origin: {}", orig.or_this())
    ///     )?;
    ///     Ok(())
    /// }
    /// ```
    fn context_from_tracked<C>(self, f: impl FnMut(Option<Location>) -> C) -> Result<T>
        where
            C: Display + Send + Sync + 'static,
            E: Tracked;

    /// Add context from an error with possibly tracked location in addition to tracking the caller
    /// of this function.
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
    ///     do_thing().context_tracked_from_tracked(|orig|
    ///         format!("error origin: {}", orig.or_this())
    ///     )?;
    ///     Ok(())
    /// }
    /// ```
    fn context_tracked_from_tracked<C>(self, f: impl FnMut(Option<Location>) -> C) -> Result<T>
        where
            C: Display + Send + Sync + 'static,
            E: Tracked;
}

impl<T, E: error::Error + Send + Sync + 'static> Context<T, E> for result::Result<T, E> {
    
    fn context<C>(self, ctx: C) -> Result<T>
        where C: Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            Error::new_internal(ctx, err, None)
        })
    }

    #[track_caller]
    fn context_tracked<C>(self, ctx: C) -> Result<T>
        where C: Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            Error::new_internal(ctx, err, Some(caller!()))
        })
    }

    fn context_with<C>(self, mut f: impl FnMut() -> C) -> Result<T>
        where C: Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            Error::new_internal(f(), err, None)
        })
    }

    #[track_caller]
    fn context_tracked_with<C>(self, mut f: impl FnMut() -> C) -> Result<T>
        where C: Display + Send + Sync + 'static
    {
        self.map_err(|err| {
            Error::new_internal(f(), err, Some(caller!()))
        })
    }

    fn context_from_tracked<C>(self, mut f: impl FnMut(Option<Location>) -> C) -> Result<T>
        where
            C: Display + Send + Sync + 'static,
            E: Tracked, 
    {
        self.map_err(|err| {
            Error::new_internal(f(err.location()), err, None)
        })
    }

    #[track_caller]
    fn context_tracked_from_tracked<C>(self, mut f: impl FnMut(Option<Location>) -> C) -> Result<T>
        where
            C: Display + Send + Sync + 'static,
            E: Tracked
    {
        self.map_err(|err| {
            Error::new_internal(f(err.location()), err, Some(caller!()))
        })
    }
}

