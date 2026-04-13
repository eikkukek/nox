use super::*;

pub unsafe trait Chainable<'a> {

    fn base_out(&mut self) -> &mut BaseOutStructure<'a>;
}

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
