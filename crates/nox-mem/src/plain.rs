/// A marker trait for plain data.
/// A trait for types that can be used as plain data.
///
/// # Safety
/// This trait is unsafe, because it allows optimizations where values may be left uninitialized.
pub unsafe trait Plain: Default + Copy {}

unsafe impl Plain for u8 {}
unsafe impl Plain for u16 {}
unsafe impl Plain for u32 {}
unsafe impl Plain for u64 {}
unsafe impl Plain for u128 {}
unsafe impl Plain for usize {}

unsafe impl Plain for i8 {}
unsafe impl Plain for i16 {}
unsafe impl Plain for i32 {}
unsafe impl Plain for i64 {}
unsafe impl Plain for i128 {}
unsafe impl Plain for isize {}

unsafe impl Plain for f32 {}
unsafe impl Plain for f64 {}
