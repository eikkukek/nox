//! Provides [`AnyError`] and [`SomeError`] for easy custom errors.

use core::{
    fmt::{self, Display, Debug, Formatter},
    error,
};

use compact_str::CompactString;

pub struct AnyError {
    desc: CompactString,
    err: Box<dyn error::Error + Send + Sync>,
}

impl AnyError {

    pub fn new(desc: impl AsRef<str>, err: impl error::Error + Send + Sync + 'static) -> Self {
        Self {
            desc: CompactString::new(desc),
            err: Box::new(err),
        }
    }

    pub fn source(&self) -> &(dyn error::Error + Send + Sync + 'static) {
        &*self.err
    }
}

impl Debug for AnyError {

    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        (&*self.err as &dyn Debug).fmt(f)
    }
}

impl Display for AnyError {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.desc)
    }
}

impl error::Error for AnyError {

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&*self.err)
    }
}



pub struct SomeError<E>
    where
        E: error::Error + Send + Sync + 'static
{
    desc: CompactString,
    err: E,
}

impl<E> SomeError<E>
    where
        E: error::Error + Send + Sync + 'static
{

    pub fn new(desc: impl AsRef<str>, err: E) -> Self {
        Self {
            desc: CompactString::new(desc),
            err,
        }
    }
    
    pub fn source(&self) -> &(dyn error::Error + 'static) {
        &self.err
    }
}

impl<E> Debug for SomeError<E>
    where
        E: error::Error + Send + Sync + 'static
{

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <E as Debug>::fmt(&self.err, f)
    }
}

impl<E> Display for SomeError<E>
    where
        E: error::Error + Send + Sync + 'static
{

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.desc)
    }
}

impl<E> error::Error for SomeError<E>
    where
        E: error::Error + Send + Sync + 'static
{

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.err)
    }
}

impl<E> From<SomeError<E>> for AnyError
    where
        E: error::Error + Send + Sync + 'static
{

    fn from(value: SomeError<E>) -> Self {
        AnyError::new(value.desc, value.err)
    }
}
