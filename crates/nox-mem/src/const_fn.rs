#[inline(always)]
pub const fn max_usize(a: usize, b: usize) -> usize {
    if a > b { a }
    else { b }
}

#[inline(always)]
pub const fn align_up(offset: usize, align: usize) -> usize {
    (offset + align - 1) & !(align - 1)
}
