pub type Location = &'static core::panic::Location<'static>;

pub trait Tracked {
    
    fn loc(&self) -> Location;
}

impl Tracked for Location {

    fn loc(&self) -> Location {
        self
    }
}

#[track_caller]
pub fn location() -> Location {
    core::panic::Location::caller()
}

#[macro_export]
macro_rules! location {
    () => {
        $crate::tracked::location()
    };
}

#[macro_export]
macro_rules! caller {
    () => {
        core::panic::Location::caller()
    };
}
