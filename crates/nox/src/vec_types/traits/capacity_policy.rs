#[derive(Debug)]
pub enum CapacityError {
    Fixed {
        capacity: usize,
    },
    InvalidReservation {
        current: usize,
        requested: usize,
    },
    AllocFailed {
        new_capacity: usize,
    },
}

pub trait CapacityPolicy {
    fn power_of_two() -> bool;
    fn can_grow() -> bool;
    fn grow(current: usize, required: usize) -> Option<usize>;
}

pub struct Dyn {}

impl CapacityPolicy for Dyn {

    fn power_of_two() -> bool {
        true
    }

    fn can_grow() -> bool {
        true
    }

    fn grow(current: usize, required: usize) -> Option<usize> {
        if required <= current { None }
        else { Some(required.max(2).next_power_of_two()) }
    }
}

pub struct Fixed {}

impl CapacityPolicy for Fixed {

    fn power_of_two() -> bool {
        false
    }

    fn can_grow() -> bool {
        false
    }

    fn grow(_: usize, _: usize) -> Option<usize> {
        None
    }
}
