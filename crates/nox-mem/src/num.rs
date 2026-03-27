use core::{
    ops::{
        Range,
        Add, AddAssign, Sub, SubAssign, Mul, MulAssign,
        BitOr, BitOrAssign, BitAnd, BitAndAssign,
        BitXor, BitXorAssign,
        Shl, ShlAssign, Shr, ShrAssign,
    },
    hash::Hash,
    fmt::{Display, Debug},
};

pub struct BitIter<T: Integer> {
    pub num: T,
    pub bit: u32,
}

impl<T: Integer> Iterator for BitIter<T> {

    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let n = size_of::<T>() as u32 * 8 - 1;
        if self.bit > n {
            None
        } else {
            let mut bit = self.bit;
            while bit <= n && T::ONE << bit & self.num == T::ZERO {
                bit += 1;
            }
            self.bit = bit + 1;
            (bit <= n).then(|| T::ONE << bit)
        }
    }
}

/// A trait for working with generic integers.
pub trait Integer:
    'static + Sized + Copy + Send + Sync +
    Add<Output = Self> + AddAssign + Sub<Output = Self> + SubAssign +
    Mul<Output = Self> + MulAssign +
    PartialEq + Eq + PartialOrd + Ord + Hash +
    BitOr<Output = Self> + BitOrAssign + BitAnd<Output = Self> + BitAndAssign +
    BitXor<Output = Self> + BitXorAssign +
    Shl<Output = Self> + ShlAssign + Shr<Output = Self> + ShrAssign +
    Shl<u32, Output = Self> + ShlAssign<u32> + Shr<u32, Output = Self> + ShrAssign<u32> +
    Display + Debug
{

    const ZERO: Self;
    const ONE: Self;
    const MIN: Self;
    const MAX: Self;

    type NonZero: NonZeroInteger<Self>;
    type Iter: Iterator<Item = Self> + DoubleEndedIterator<Item = Self>;

    #[must_use]
    fn iter(self, to: Self) -> Self::Iter;

    /// Returns an iterator over bits.
    #[must_use]
    #[inline(always)]
    fn bit_iter(self) -> BitIter<Self> {
        BitIter { num: self, bit: 0, }
    }

    #[must_use]
    fn range(self, to: Self) -> Range<Self>;

    #[must_use]
    fn step_forward(self) -> Self;

    #[must_use]
    fn step_backward(self) -> Self;

    #[must_use]
    fn wrapping_add(self, x: Self) -> Self;

    #[must_use]
    fn wrapping_sub(self, x: Self) -> Self;

    #[must_use]
    fn wrapping_neg(self) -> Self;

    #[must_use]
    fn saturating_sub(self, x: Self) -> Self;

    #[must_use]
    fn from_bool(x: bool) -> Self;
}

macro_rules! impl_integers {
    ($($num:ty),*) => {
        $(
            impl Integer for $num {

                const ZERO: Self = 0;
                const ONE: Self = 1;
                const MIN: Self = Self::MIN;
                const MAX: Self = Self::MAX;

                type NonZero = core::num::NonZero<$num>;
                type Iter = Range<Self>;

                #[inline(always)]
                fn iter(self, to: Self) -> Self::Iter {
                    self..to
                }

                #[inline(always)]
                fn range(self, to: Self) -> Range<Self> {
                    self..to
                }

                #[inline(always)]
                fn step_forward(self) -> Self {
                    self + 1
                }

                #[inline(always)]
                fn step_backward(self) -> Self {
                    self - 1
                }

                #[inline(always)]
                fn wrapping_add(self, x: Self) -> Self {
                    self.wrapping_add(x)
                }

                #[inline(always)]
                fn wrapping_sub(self, x: Self) -> Self {
                    self.wrapping_sub(x)
                }

                #[inline(always)]
                fn wrapping_neg(self) -> Self {
                    self.wrapping_neg()
                }

                #[inline(always)]
                fn saturating_sub(self, x: Self) -> Self {
                    self.saturating_sub(x)
                }

                #[inline(always)]
                fn from_bool(x: bool) -> Self {
                    x as Self
                }
            }
        )*
    };
}

impl_integers!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);


pub trait NonZeroInteger<T: Integer>:
    Clone + Copy + PartialEq + Eq + Hash +
    Debug + Display
{

    fn new(value: T) -> Option<Self>;
    
    /// Creates a new [`NonZero`] without checking if the value is zero.
    ///
    /// # Safety
    /// If the value is zero, this will produce and invalid value.
    unsafe fn new_unchecked(value: T) -> Self;

    fn get(self) -> T;
}

macro_rules! impl_non_zero_integer {
    ($($t:ty),*) => {
    
        $(
        impl NonZeroInteger<$t> for core::num::NonZero<$t> {

            #[inline(always)]
            fn new(value: $t) -> Option<Self> {
                Self::new(value)
            }

            #[inline(always)]
            unsafe fn new_unchecked(value: $t) -> Self {
                unsafe {
                    Self::new_unchecked(value)
                }
            }

            #[inline(always)]
            fn get(self) -> $t {
                self.get()
            }
        }
        )*
    };
}

impl_non_zero_integer!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

/// A trait for working with generic unsigned integers.
pub trait UInteger: Integer {}

impl UInteger for u8 {}
impl UInteger for u16 {}
impl UInteger for u32 {}
impl UInteger for u64 {}
impl UInteger for u128 {}
impl UInteger for usize {}

pub trait IntoUsize: Integer {

    fn into_usize(self) -> usize;
}

macro_rules! impl_into_usize {
    ($($num:ty),*) => {
        $(
            impl IntoUsize for $num {
                
                #[inline(always)]
                fn into_usize(self) -> usize {
                    self as usize
                }
            }
        )*
    };
}

impl_into_usize!(u8, u16, u32, usize, i32);

pub trait FromU32: Integer {

    fn from_u32(value: u32) -> Self;
}

impl FromU32 for u32 {

    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value
    }
}

impl FromU32 for usize {

    fn from_u32(value: u32) -> Self {
        value as Self
    }
}

pub trait FromUsize: Integer {

    fn from_usize(value: usize) -> Option<Self>;

    fn from_usize_unchecked(value: usize) -> Self;
}

impl FromUsize for usize {

    fn from_usize(value: usize) -> Option<Self> {
        Some(value)
    }

    fn from_usize_unchecked(value: usize) -> Self {
        value
    }
}

impl FromUsize for u32 {

    fn from_usize(value: usize) -> Option<Self> {
        if value > Self::MAX as usize {
            None
        } else {
            Some(value as u32)
        }
    }

    fn from_usize_unchecked(value: usize) -> Self {
        value as u32
    }
}

pub trait NonUsize {}

impl NonUsize for u8 {}
impl NonUsize for u16 {}
impl NonUsize for u32 {}
impl NonUsize for i32 {}
