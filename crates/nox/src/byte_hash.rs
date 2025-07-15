use blake3::Hasher;
use nox_mem::AsRaw;

pub trait ByteHash {

    fn byte_hash(&self, hasher: &mut Hasher);
}

macro_rules! impl_le_hash {
    ($($t:ty),+) => {
        $(
            impl ByteHash for $t {

                #[inline(always)]
                fn byte_hash(&self, hasher: &mut Hasher) {
                    hasher.update(self.to_le_bytes().as_slice());
                }
            }
        )+
    };
}

impl_le_hash!(
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64
);

impl ByteHash for &str {

    #[inline(always)]
    fn byte_hash(&self, hasher: &mut Hasher) {
        hasher.update(self.as_bytes());
    }
}

impl<T: ByteHash> ByteHash for [T] {

    #[inline(always)]
    fn byte_hash(&self, hasher: &mut Hasher) {
        (self.len() as u128).byte_hash(hasher);
        for t in self {
            t.byte_hash(hasher);
        }
    }
}

impl<T: ByteHash> ByteHash for Option<T> {

    #[inline(always)]
    default fn byte_hash(&self, hasher: &mut Hasher) {
        match self {
            None => 0u32.byte_hash(hasher),
            Some(t) => {
                1u32.byte_hash(hasher);
                t.byte_hash(hasher);
            }
        }
    }
}

impl ByteHash for bool {

    #[inline(always)]
    fn byte_hash(&self, hasher: &mut Hasher) {
        (*self as u32).byte_hash(hasher);
    }
}
