use core::fmt::{self, Debug, Formatter};

use crate::Display;

#[derive(Display, Clone, Copy, PartialEq, Eq, Hash)] #[display("{0}")]
pub struct Location(&'static core::panic::Location<'static>);

impl Debug for Location {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <core::panic::Location<'_> as Debug>::fmt(self.0, f)
    }
}

#[inline(always)]
#[track_caller]
pub fn location() -> Location {
    Location(core::panic::Location::caller())
}

#[inline(always)]
pub fn new(loc: &'static core::panic::Location<'static>) -> Location {
    Location(loc)
}

#[macro_export]
macro_rules! location {
    () => {
        $crate::location::location()
    };
}

#[macro_export]
macro_rules! caller {
    () => {
        $crate::location::new(core::panic::Location::caller()).into()
    };
}
