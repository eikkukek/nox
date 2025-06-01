use core::marker::PhantomData;

pub struct DynVec<'mem, T> {
    data: *mut T,
    size: usize,
    len: usize,
    _marker: PhantomData<&'mem ()>,
}
