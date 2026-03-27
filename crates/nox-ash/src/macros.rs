#[macro_export]
macro_rules! load_fn {
    (
        fn $rname:ident($($arg:ty),* $(,)?) -> $ret:ty,
        $load:ident,
        $cname:literal,
        $pfn:ty $(,)?
    ) => {
        {
            unsafe extern "system" fn $rname(
                $(_: $arg),*
            ) -> $ret {
                panic!(concat!("unable to load ", stringify!($rname)))
            }
            let val = $load($cname);
            if val.is_null() {
                $rname
            } else {
                ::core::mem::transmute::<
                    *const c_void,
                    $pfn,
                >(val)
            }
        }
    };
}

#[macro_export]
macro_rules! ash_style_enum {
    ($(
        $(#[doc = $doc:literal])*
        #[flags($repr:ty)]
        $(#[default = $default:expr])?
        $vis:vis enum $name:ident {
            $(
                $(#[doc = $doc_variant:literal])*
                #[display($display:literal)]
                $field:ident = $value:expr
            ),* $(,)?
        }
    )+) =>
    {$(
        $(#[doc = $doc])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $name($repr);

        impl $name {

            $(
                $(#[doc = $doc_variant])*
                pub const $field: Self = Self($value);
            )*

            /// Creates empty flags.
            #[inline(always)]
            pub const fn empty() -> Self {
                Self(0)
            }

            /// Clears all flags.
            #[inline(always)]
            pub const fn clear(&mut self) {
                self.0 = 0;
            }

            /// Returns the underlying value of `self`.
            #[inline(always)]
            pub const fn as_raw(self) -> $repr {
                self.0
            }

            /// Constructs `self` from the underlying value.
            #[inline(always)]
            pub const fn from_raw(x: $repr) -> Self {
                Self(x)
            }

            /// Returns whether `self` is equal to `0`.
            #[inline(always)]
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }

            /// Returns whether `self` overlaps with `other`.
            #[inline(always)]
            pub const fn intersects(self, other: Self) -> bool {
                self.0 & other.0 != 0
            }

            /// Returns whether `other` is a subset of `self`.
            #[inline(always)]
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }

            const DEBUG_LOOK_UP: [Option<&str>; <$repr>::BITS as usize + 1] = {
                let mut names = [None; <$repr>::BITS as usize + 1];
                $(
                    if Self::$field.0.count_ones() == 1 {
                        names[Self::$field.0.trailing_zeros() as usize]
                            = Some(stringify!($field));
                    }
                )*
                names
            };

            const DISPLAY_LOOK_UP: [Option<&str>; <$repr>::BITS as usize + 1] = {
                let mut names = [None; <$repr>::BITS as usize + 1];
                $(
                    if Self::$field.0.count_ones() == 1 {
                        names[Self::$field.0.trailing_zeros() as usize]
                            = Some($display);
                    }
                )*
                names
            };
        }

        impl ::core::ops::Deref for $name {

            type Target = $repr;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        $(
            impl Default for $name {

                #[inline(always)]
                fn default() -> Self {
                    $default
                }
            }
        )?

        impl ::core::ops::BitOr for $name {

            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl ::core::ops::BitOrAssign for $name {

            #[inline(always)]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl ::core::ops::BitAnd for $name {

            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }

        impl ::core::ops::BitAndAssign for $name {

            #[inline(always)]
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        impl ::core::ops::BitXor for $name {

            type Output = Self;

            #[inline(always)]
            fn bitxor(self, rhs: Self) -> Self {
                Self(self.0 ^ rhs.0)
            }
        }

        impl ::core::ops::BitXorAssign for $name {

            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }

        impl ::core::fmt::Debug for $name {

            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                if self.is_empty() {
                    write!(f, "[NONE]")
                } else {
                    let mut iter = <$repr as $crate::mem::num::Integer>::bit_iter(self.0);
                    let bit = unsafe {
                        iter.next().unwrap_unchecked()
                    };
                    match Self::DEBUG_LOOK_UP[bit.trailing_zeros() as usize] {
                        Some(look_up) => {
                            write!(f, "[{}", look_up)?;
                        },
                        None => {
                            write!(f, "[{:#x}", bit)?;
                        }
                    };
                    for bit in iter {
                        match Self::DEBUG_LOOK_UP[bit.trailing_zeros() as usize] {
                            Some(look_up) => {
                                write!(f, " | {}", look_up)?;
                            },
                            None => {
                                write!(f, " | {:#x}", bit)?;
                            }
                        };
                    }
                    write!(f, "]")
                }
            }
        }

        impl ::core::fmt::Display for $name {

            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                if self.is_empty() {
                    write!(f, "[none]")
                } else {
                    let mut iter = <$repr as $crate::mem::num::Integer>::bit_iter(self.0);
                    let bit = unsafe {
                        iter.next().unwrap_unchecked()
                    };
                    match Self::DISPLAY_LOOK_UP[bit.trailing_zeros() as usize] {
                        Some(look_up) => {
                            write!(f, "[{}", look_up)?;
                        },
                        None => {
                            write!(f, "[{:#x}", bit)?;
                        }
                    };
                    for bit in iter {
                        match Self::DISPLAY_LOOK_UP[bit.trailing_zeros() as usize] {
                            Some(look_up) => {
                                write!(f, " | {}", look_up)?;
                            },
                            None => {
                                write!(f, " | {:#x}", bit)?;
                            }
                        };
                    }
                    write!(f, "]")
                }
            }
        }
    )+};
    ($(
        $(#[doc = $doc:literal])*
        #[enum($repr:ty)]
        $(#[default = $default:expr])?
        $vis:vis enum $name:ident {
            $(
                $(#[doc = $doc_variant:literal])*
                #[display($display:literal)]
                $field:ident = $value:expr
            ),* $(,)?
        }
    )+) =>
    {$(
        $(#[doc = $doc])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $name($repr);

        impl $name {

            $(
                $(#[doc = $doc_variant])*
                pub const $field: Self = Self($value);
            )*

            /// Returns the underlying value of `self`.
            #[inline(always)]
            pub const fn as_raw(&self) -> $repr {
                self.0
            }

            /// Constructs `self` from the underlying value.
            ///
            /// # Safety
            /// For enum, this can result in invalid values.
            #[inline(always)]
            pub const fn from_raw(x: $repr) -> Self {
                Self(x)
            }
        }

        impl ::core::fmt::Debug for $name {

            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let x = *self;
                $(
                    if x == Self::$field {
                        return write!(f, "{}", stringify!($field))
                    }
                )*
                Ok(())
            }
        }

        impl ::core::fmt::Display for $name {

            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let x = *self;
                $(
                    if x == Self::$field {
                        return write!(f, "{}", $display)
                    }
                )*
                Ok(())
            }
        }

        $(
            impl Default for $name {

                #[inline(always)]
                fn default() -> Self {
                    $default
                }
            }
        )?
    )+}
}
