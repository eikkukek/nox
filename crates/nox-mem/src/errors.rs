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
    ZeroSizedElement,
}
