pub trait AsRaw {

    type Repr;

    fn as_raw(self) -> Self::Repr;
}

#[macro_export]
macro_rules! impl_as_raw_bit_op {
    ($($t:ty),+ $(,)?) => {
        $(
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
