pub trait CapacityPolicy {
    fn power_of_two() -> bool;
    fn can_grow() -> bool;
    fn grow(current: usize, required: usize) -> Option<usize>;
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
    fn grow(current: usize, required: usize) -> Option<usize> {
        if required <= current { None }
        else { Some(required.max(2).next_power_of_two()) }
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
    fn grow(_: usize, _: usize) -> Option<usize> {
        None
    }
}
