use core::{
    error,
    fmt::{self, Display, Debug, Formatter},
};

use nox_proc::{Error, Display};

use nox_mem::dynamic::{Dyn, Pair};

use super::{Location, Tracked};

enum Internal {
    JustContext(Box<dyn Display + Send + Sync>),
    WithSource(Pair<dyn Display + Send + Sync, dyn error::Error + Send + Sync>),
}

impl Internal {

    fn context(&self) -> &(dyn Display + 'static) {
        match self {
            Self::JustContext(ctx) => ctx,
            Self::WithSource(pair) => pair.first(),
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::JustContext(_) => None,
            Self::WithSource(pair) => Some(pair.second()),
        }
    }
}

impl Debug for Internal {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::JustContext(ctx) => write!(f, "Error(ctx: {}, err: None)", ctx),
            Self::WithSource(pair) => write!(f, "Error(ctx: {}, err: {:?})", pair.first(), pair.second()),
        }
    }
}

/// The error type Nox uses.
///
/// Allows giving errors a context with the [`Context`][crate::Context] trait.
#[derive(Error)]
#[display(format_args!("{}", self.internal.context()))]
pub struct Error {
    #[source(self.source())] internal: Internal,
    loc: Option<Location>,
}

#[derive(Dyn, Display)]
#[display("{0}")]
#[bounds(Display + Send + Sync)]
struct WrapCtx<T: Display + Send + Sync + 'static>(T);

#[derive(Dyn, Error)]
#[display("{0}")]
#[bounds(error::Error + Send + Sync)]
struct WrapErr<T: error::Error + Send + Sync + 'static>(#[source(self.0.source())] T);

impl<T: error::Error + Send + Sync + 'static> Debug for WrapErr<T> {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <T as Debug>::fmt(&self.0, f)
    }
}

impl Error { 

    #[inline(always)]
    pub fn new<E, C>(err: E, ctx: C) -> Self
        where
            E: error::Error + Send + Sync + 'static,
            C: Display + Send + Sync + 'static,
    {
        Self::new_internal(err, ctx, None)
    }

    #[track_caller]
    pub fn new_tracked<E, C>(err: E, ctx: C) -> Self
        where
            E: error::Error + Send + Sync + 'static,
            C: Display + Send + Sync + 'static,
    {
        Self::new_internal(err, ctx, Some(caller!()))
    }

    #[inline(always)]
    pub fn just_context<C>(ctx: C) -> Self
        where C: Display + Send + Sync + 'static,
    {
        Self::just_context_internal(ctx, None)
    }

    #[track_caller]
    pub fn just_context_tracked<C>(ctx: C) -> Self
        where C: Display + Send + Sync + 'static,
    {
        Self::just_context_internal(ctx, Some(caller!()))
    }

    #[inline(always)]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.internal.source()
    }

    #[inline(always)]
    pub fn with_location(mut self, loc: Location) -> Self {
        self.loc = Some(loc);
        self
    }

    fn new_internal(
        err: impl error::Error + Send + Sync + 'static,
        ctx: impl Display + Send + Sync + 'static,
        loc: Option<Location>,
    ) -> Self
    {
        Self {
            internal: Internal::WithSource(Pair::new(WrapCtx(ctx), WrapErr(err))),
            loc,
        }
    }

    fn just_context_internal(
        ctx: impl Display + Send + Sync + 'static,
        loc: Option<Location>,
    ) -> Self
    {
        Self {
            internal: Internal::JustContext(Box::new(ctx)),
            loc,
        }
    }
}

impl Debug for Error {

    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Internal as Debug>::fmt(&self.internal, f)
    }
}

impl Tracked for Error {

    fn location(&self) -> Option<Location> {
        self.loc
    }
}

pub mod build_error {

    use super::*;

    pub fn new<E, C>(
        err: E,
        ctx: C,
        loc: Option<Location>,
    ) -> Error
        where 
        E: error::Error + Send + Sync + 'static,
        C: Display + Send + Sync + 'static,
    {
        Error::new_internal(err, ctx, loc)
    }

    pub fn just_context<C>(
        ctx: C,
        loc: Option<Location>,
    ) -> Error
        where C: Display + Send + Sync + 'static
    {
        Error::just_context_internal(ctx, loc)
    }
}
