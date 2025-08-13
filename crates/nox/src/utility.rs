#[macro_export]
macro_rules! count_idents {
    ($($idents:ident),+) => {
        <[()]>::len(&[$(crate::count_idents![@sub $idents]),*])
    };
    (@sub $i:ident) => { () }
}

#[macro_export]
macro_rules! has_bits {
    ($a:expr, $b:expr) => ($a & $b == $b)
}

#[macro_export]
macro_rules! has_not_bits {
    ($a:expr, $b:expr) => ($a & $b != $b)
}

#[inline(always)]
pub fn clamp<T>(x: T, min: T, max: T) -> T
    where
        T: PartialOrd
{
	let clamped = if x > min { x } else { min };
	return if clamped < max { clamped } else { max };
}
