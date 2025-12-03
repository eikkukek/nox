use crate::CapacityError;

pub trait CapacityPolicy {
    fn power_of_two() -> bool;
    fn can_grow() -> bool;
    fn grow(current: usize, required: usize) -> Result<usize, CapacityError>;
}

pub struct Dyn {}

impl CapacityPolicy for Dyn {

    #[inline]
    fn power_of_two() -> bool {
        true
    }

    #[inline]
    fn can_grow() -> bool {
        true
    }

    #[inline]
    fn grow(current: usize, required: usize) -> Result<usize, CapacityError> {
        let power_of_2 = required.next_power_of_two().max(2);
        if power_of_2 <= current { Ok(current) }
        else { Ok(power_of_2) }
    }
}

pub struct Fixed {}

impl CapacityPolicy for Fixed {

    #[inline]
    fn power_of_two() -> bool {
        false
    }

    #[inline]
    fn can_grow() -> bool {
        false
    }

    #[inline]
    fn grow(current: usize, _: usize) -> Result<usize, CapacityError> {
        return Err(CapacityError::FixedCapacity { capacity: current })
    }
}
