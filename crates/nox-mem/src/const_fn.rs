#[inline(always)]
pub const fn max_usize(a: usize, b: usize) -> usize {
    if a > b { a }
    else { b }
}

/// (offset + align - 1) & !(align - 1)
#[inline(always)]
pub const fn align_up(offset: usize, align: usize) -> usize {
    (offset + align - 1) & !(align - 1)
}

#[inline(always)]
pub const fn align_up_u64(offset: u64, align: u64) -> u64 {
    (offset + align - 1) & !(align - 1)
}
