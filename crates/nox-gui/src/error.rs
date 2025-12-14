use core::fmt::Display;

use nox::dev::error::{self, BuildInternal};

pub use error::{Context, Location, Tracked, caller};

#[derive(error::Error, Debug)] #[display("{0}")]
pub struct Error(#[source] error::Error);

impl Error {

    #[track_caller]
    pub fn new<C>(ctx: C, err: impl core::error::Error + Send + Sync + 'static) -> Self
        where C: Display + Send + Sync + 'static
    {
        Self(error::Error::new_internal(ctx, err, caller!()))
    }

    #[track_caller]
    pub fn just_context<C>(ctx: C) -> Self
        where C: Display + Send + Sync + 'static,
    {
        Self(error::Error::just_context_internal(ctx, caller!()))
    }
}

impl From<nox::dev::error::Error> for Error {

    #[track_caller]
    fn from(value: nox::dev::error::Error) -> Self {
        Self(value.with_location(caller!()))
    }
}

impl From<Error> for nox::error::Error {

    #[track_caller]
    fn from(value: Error) -> Self {
        Self(value.0.with_location(caller!()))
    }
}

pub type Result<T> = core::result::Result<T, Error>;
