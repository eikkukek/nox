use super::*;

/// A trait for structures, which *can* be a part of `p_next` chains.
///
/// # Safety
/// A structure implementing this trait *must* adhere to the memory layout of [`BaseOutStructure`].
pub unsafe trait Chainable<'a> {

    /// Casts self to [`BaseOutStructure`].
    fn base_out(&mut self) -> &mut BaseOutStructure<'a>;
}

/// Creates an iterator over a `p_next` chain.
pub fn chain_iter<'a, T>(
    first: &'a mut T,
) -> impl Iterator<Item = &'a mut BaseOutStructure<'a>>
    where T: ?Sized + Chainable<'a>
{
    let out: *mut BaseOutStructure<'a> = first.base_out();
    (0..).scan(out, |out, _| unsafe {
        let this = *out;
        if out.is_null() { return None };
        let next = (*this).p_next;
        *out = next;
        Some(&mut *this)
    })
}
