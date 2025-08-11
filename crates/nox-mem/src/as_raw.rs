/// A trait for `repr({integer})` enums with unit-only variants.
///
/// This trait allows extracting the underlying integer representation of such enums
/// using a consistent and safe interface.
///
/// For bitflag-style enums, you can implement bitwise ops:
///
/// ```
/// impl_as_raw_bit_op!(MyEnum);
/// ```
///
/// # Example
///
/// ```
/// #[repr(u32)]
/// #[derive(AsRaw)]
/// pub enum MyEnum {
///     Read = 0x1,
///     Write = 0x2,
///     Execute = 0x4,
/// }
///
/// impl_as_raw_bit_op!(MyEnum);
///
/// let execute: u32 = MyEnum::Execute.as_raw();
/// assert_eq!(execute, 0x4);
/// assert_eq!(MyEnum::Read | MyEnum::Write, 0x1 | 0x2)
/// ```
pub trait AsRaw {

    type Repr;

    fn as_raw(self) -> Self::Repr;
}

#[macro_export]
macro_rules! impl_as_raw_bit_op {
    ($($t:ty),+ $(,)?) => {
        $(

        impl PartialEq<<$t as AsRaw>::Repr> for $t {

            fn eq(&self, rhs: &<$t as AsRaw>::Repr) -> bool {
                *self as <$t as AsRaw>::Repr == *rhs
            }
        }

        impl PartialEq<$t> for <$t as AsRaw>::Repr {

            fn eq(&self, rhs: &$t) -> bool {
                *self == *rhs as Self
            }
        }

        impl core::ops::BitAnd for $t {

            type Output = <Self as AsRaw>::Repr;

            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self::Output {
                self.as_raw() & rhs
            }
        }

        impl core::ops::BitAnd<<$t as AsRaw>::Repr> for $t
        {
            type Output = <Self as AsRaw>::Repr;

            #[inline(always)]
            fn bitand(self, rhs: <$t as AsRaw>::Repr) -> Self::Output {
                self.as_raw() & rhs
            }
        }

        impl core::ops::BitAnd<$t> for <$t as AsRaw>::Repr
        {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: $t) -> Self::Output {
                self & rhs.as_raw()
            }
        }

        impl core::ops::BitAndAssign<$t> for <$t as AsRaw>::Repr
        {
            #[inline(always)]
            fn bitand_assign(&mut self, rhs: $t) {
                *self &= rhs.as_raw()
            }
        }

        impl core::ops::BitOr for $t {

            type Output = <Self as AsRaw>::Repr;

            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self::Output {
                self.as_raw() | rhs
            }
        }

        impl core::ops::BitOr<<$t as AsRaw>::Repr> for $t
        {
            type Output = <Self as AsRaw>::Repr;

            #[inline(always)]
            fn bitor(self, rhs: <$t as AsRaw>::Repr) -> Self::Output {
                self.as_raw() | rhs
            }
        }

        impl core::ops::BitOr<$t> for <$t as AsRaw>::Repr
        {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: $t) -> Self::Output {
                self | rhs.as_raw()
            }
        }

        impl core::ops::BitOrAssign<$t> for <$t as AsRaw>::Repr
        {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: $t) {
                *self |= rhs.as_raw()
            }
        }
        )+
    };
}
