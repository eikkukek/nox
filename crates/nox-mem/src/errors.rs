#[derive(Debug)]
pub enum CapacityError {
    FixedCapacity {
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
