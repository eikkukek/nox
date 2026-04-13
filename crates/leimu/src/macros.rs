/// ``` rust
/// x |= y & (z as u32) << y.trailing_zeros()
/// ```
#[macro_export]
macro_rules! or_flag {
    ($flags:expr, $flag:expr, $value:expr $(,)?) => {
        $flags |= $flag & ($value as u32) << $flag.trailing_zeros();
    };
}
