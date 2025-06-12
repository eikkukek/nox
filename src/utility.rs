#[inline(always)]
pub fn clamp<T>(x: T, min: T, max: T) -> T
    where
        T: PartialOrd
{
	let clamped = if x > min { x } else { min };
	return if clamped < max { clamped } else { max };
}

macro_rules! has_bits {
    ($a:expr, $b:expr) => ($a & $b == $b)
}
pub(super) use has_bits;

macro_rules! has_not_bits {
    ($a:expr, $b:expr) => ($a & $b != $b)
}
pub(super) use has_not_bits;

#[inline(always)]
pub fn next_align(offset: usize, align: usize) -> usize {
    (offset + align - 1) & !(align - 1)
}
