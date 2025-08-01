pub trait ByteHasher {

    fn update(&mut self, input: &[u8]);
}

pub trait ByteHash {

    fn byte_hash<H: ByteHasher>(&self, state: &mut H);
}

macro_rules! impl_le_hash {
    ($($t:ty),+) => {
        $(
            impl ByteHash for $t {

                #[inline(always)]
                fn byte_hash<H: ByteHasher>(&self, state: &mut H) {
                    state.update(self.to_le_bytes().as_slice());
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
    fn byte_hash<H: ByteHasher>(&self, state: &mut H) {
        state.update(self.as_bytes());
    }
}

impl<T: ByteHash> ByteHash for [T] {

    #[inline(always)]
    fn byte_hash<H: ByteHasher>(&self, state: &mut H) {
        (self.len() as u128).byte_hash(state);
        for t in self {
            t.byte_hash(state);
        }
    }
}

impl<T: ByteHash> ByteHash for Option<T> {

    #[inline(always)]
    default fn byte_hash<H: ByteHasher>(&self, hasher: &mut H) {
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
    fn byte_hash<H: ByteHasher>(&self, hasher: &mut H) {
        (*self as u32).byte_hash(hasher);
    }
}
