pub trait Primitive<const BYTES: usize>: Clone + Copy {

    fn to_le_bytes(self) -> [u8; BYTES];

    fn from_le_bytes(bytes: [u8; BYTES]) -> Self;
}

macro_rules! impl_primitive {
    ($($t:ty),+ $(,)?) => {
        $(
        impl Primitive<{size_of::<$t>()}> for $t {

            fn to_le_bytes(self) -> [u8; size_of::<$t>()] {
                self.to_le_bytes()
            }

            fn from_le_bytes(bytes: [u8; size_of::<$t>()]) -> Self {
                Self::from_le_bytes(bytes)
            }
        }
        )+
    };
}

impl_primitive!(
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64,
);
