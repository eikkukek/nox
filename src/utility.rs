pub fn clamp<T>(x: T, min: T, max: T) -> T
    where
        T: PartialOrd
{
	let clamped = if x > min { x } else { min };
	return if clamped < max { clamped } else { max };
}

macro_rules! has_bit {
    ($a:expr, $b:expr) => ($a & $b == $b)
}
pub(super) use has_bit;

macro_rules! has_not_bit {
    ($a:expr, $b:expr) => ($a & $b != $b)
}
pub(super) use has_not_bit;
