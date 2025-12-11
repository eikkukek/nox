use core::{
    error,
    fmt::{self, Debug, Display, Formatter},
};

use nox_derive::Error;

use nox_error::BuildInternal;

pub use nox_error::{Location, Context, Tracked, location, caller};

#[derive(Error)] #[display("{0}")]
pub struct Error(#[source(self.0.source())] nox_error::Error);

impl Error {

    #[track_caller]
    pub fn new<C>(ctx: C, err: impl error::Error + Send + Sync + 'static) -> Self
        where C: Display + Send + Sync + 'static
    {
        Self(nox_error::Error::new_internal(ctx, err, caller!()))
    }

    #[track_caller]
    pub fn just_context<C>(ctx: C) -> Self
        where C: Display + Send + Sync + 'static,
    {
        Self(nox_error::Error::just_context_internal(ctx, caller!()))
    }
}

impl From<nox_error::Error> for Error {

    #[track_caller]
    fn from(value: nox_error::Error) -> Self {
        Self(value.with_location(caller!()))
    }
}

impl Debug for Error {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <nox_error::Error as Debug>::fmt(&self.0, f)
    }
}

impl Tracked for Error {

    fn location(&self) -> Option<Location> {
        self.0.location()
    }
}
