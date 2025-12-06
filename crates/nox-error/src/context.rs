use core::error;

use super::any::AnyError;

pub trait Context<T, E: error::Error + Send + Sync + 'static> { 

    fn ctx_err(self, ctx: impl AsRef<str>) -> Result<T, AnyError>;

    fn ctx_err_with<C: AsRef<str>>(self, f: impl FnMut() -> C) -> Result<T, AnyError>;

    fn any_err(self) -> Result<T, AnyError>
        where
            E: Into<AnyError>;
}

impl<T, E: error::Error + Send + Sync + 'static> Context<T, E> for Result<T, E> {

    fn ctx_err(self, ctx: impl AsRef<str>) -> Result<T, AnyError> {
        self.map_err(|err| AnyError::new(ctx, err))
    }

    fn ctx_err_with<C: AsRef<str>>(self, mut f: impl FnMut() -> C) -> Result<T, AnyError> {
        self.map_err(|err| AnyError::new(f(), err))
    }

    fn any_err(self) -> Result<T, AnyError>
        where
            E: Into<AnyError>
    {
        self.map_err(|err| err.into())
    }
}

