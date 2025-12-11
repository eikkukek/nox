use core::{
    error,
    fmt::{self, Display, Debug, Formatter},
};

use nox_derive::Error;

use nox_mem::dynamic::{Dyn, Pair};

use super::{Location, Tracked, caller};

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

#[derive(Error)] #[display(format_args!("{}", self.internal.context()))]
pub struct Error {
    #[source(self.source())] internal: Internal,
    loc: Location,
}

#[derive(Dyn)] #[wrapped(&self.0)] #[bounds(Display + Send + Sync)]
struct WrapCtx<T: Display + Send + Sync + 'static>(T);

#[derive(Dyn)] #[wrapped(&self.0)] #[bounds(error::Error + Send + Sync)]
struct WrapErr<T: error::Error + Send + Sync + 'static>(T);

impl Error { 

    #[track_caller]
    pub fn new<C>(ctx: C, err: impl error::Error + Send + Sync + 'static) -> Self
        where C: Display + Send + Sync + 'static,
    {
        Self::new_internal(ctx, err, caller!())
    }

    #[track_caller]
    pub fn just_context<C>(ctx: C) -> Self
        where C: Display + Send + Sync + 'static,
    {
        Self::just_context_internal(ctx, caller!())
    } 

    #[inline(always)]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.internal.source()
    }

    #[inline(always)]
    fn new_internal(
        ctx: impl Display + Send + Sync + 'static,
        err: impl error::Error + Send + Sync + 'static,
        loc: Location,
    ) -> Self
    {
        Self {
            internal: Internal::WithSource(Pair::new(WrapCtx(ctx), WrapErr(err))),
            loc,
        }
    }

    #[inline(always)]
    fn just_context_internal(
        ctx: impl Display + Send + Sync + 'static,
        loc: Location,
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

    fn loc(&self) -> Location {
        self.loc
    }
}

pub trait BuildInternal {

    fn new_internal(
        ctx: impl Display + Send + Sync + 'static,
        err: impl error::Error + Send + Sync + 'static,
        loc: Location,
    ) -> Self;

    fn just_context_internal(
        ctx: impl Display + Send + Sync + 'static,
        loc: Location,
    ) -> Self;

    fn with_location(self, loc: Location) -> Self;
}

impl BuildInternal for Error {

    fn new_internal(
        ctx: impl Display + Send + Sync + 'static,
        err: impl error::Error + Send + Sync + 'static,
        loc: Location,
    ) -> Self {
        Self::new_internal(ctx, err, loc)
    }

    fn just_context_internal(
        ctx: impl Display + Send + Sync + 'static,
        loc: Location,
    ) -> Self {
        Self::just_context_internal(ctx, loc)
    }

    fn with_location(mut self, loc: Location) -> Self {
        self.loc = loc;
        self
    }
}

#[macro_export]
macro_rules! impl_wrapper {
    ($vis:vis $ident:ident) => {
        #[derive($crate::Error)] #[display("{0}")]
        $vis struct $ident(#[source(self.0.source())] $crate::Error);

        impl $ident {

            #[track_caller]
            pub fn new<C>(ctx: C, err: impl core::error::Error + Send + Sync + 'static) -> Self
                where C: core::fmt::Display + Send + Sync + 'static
            {
                Self(<$crate::Error as $crate::BuildInternal>::new_internal(ctx, err, $crate::caller!()))
            }

            #[track_caller]
            pub fn just_context<C>(ctx: C) -> Self
                where C: core::fmt::Display + Send + Sync + 'static,
            {
                Self(<$crate::Error as $crate::BuildInternal>::just_context_internal(ctx, $crate::caller!()))
            }
        }

        impl core::fmt::Debug for $ident {

            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                <$crate::Error as core::fmt::Debug>::fmt(&self.0, f)
            }
        }
    };
}
