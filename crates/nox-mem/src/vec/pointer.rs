use core::{
    mem::needs_drop,
    ptr::NonNull,
    ops::{Deref, DerefMut},
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use crate::{
    impl_traits,
    num::IntoUsize,
};

/// A helper pointer wrapper for common vector operations.
///
/// # Examples
/// ``` rust
/// 
/// use nox_mem::alloc::{StdAlloc, LocalAllocExt};
/// use nox_mem::vec::Pointer;
///
/// unsafe {
///     let ptr: Pointer<String> = StdAlloc
///         .allocate_uninit(4)
///         .unwrap().into();
///     ptr.insert_element("foo".to_string(), 0, 0);
///     ptr.insert_element("bar".to_string(), 1, 1);
///     ptr.insert_element("hello".to_string(), 2, 2);
///     ptr.insert_element("world".to_string(), 3, 3);
///     ptr.drop_in_place(4);
///     StdAlloc.free_uninit(ptr.into(), 4);
/// }
///
/// 
///
/// ```
#[derive(Eq)]
pub struct Pointer<T: Sized, SizeType: IntoUsize = usize>(NonNull<T>, PhantomData<SizeType>);

impl<T, SizeType: IntoUsize> Debug for Pointer<T, SizeType> {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T, SizeType: IntoUsize> Clone for Pointer<T, SizeType> {

    fn clone(&self) -> Self {
        *self
    }
}

impl<T, SizeType: IntoUsize> Copy for Pointer<T, SizeType> {}

impl<T, SizeType: IntoUsize> PartialEq for Pointer<T, SizeType> {

    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Sized, SizeType: IntoUsize> Pointer<T, SizeType> {

    #[inline(always)]
    pub fn new(ptr: *mut T) -> Option<Self>
    {
        Some(Self(NonNull::new(ptr)?, PhantomData))
    }

    #[inline(always)]
    pub fn dangling() -> Self {
        Self(NonNull::dangling(), PhantomData)
    }

    /// # Safety
    /// This is unsafe as it can create invalid pointers.
    #[inline(always)]
    pub unsafe fn add(self, count: usize) -> Self {
        unsafe {
            self.0.add(count.into_usize()).into()
        }
    }

    /// # Safety
    /// This is unsafe as it can create invalid pointers.
    #[inline(always)]
    pub unsafe fn sub(self, count: usize) -> Self {
        unsafe {
            self.0.sub(count.into_usize()).into()
        }
    }

    #[inline(always)]
    pub fn cast<U>(self) -> Pointer<U, SizeType> {
        Pointer(self.0.cast(), PhantomData)
    }
   
    /// Moves elements from the pointee of [`self`] to the pointee of `dst`.
    /// # Safety
    /// The source and destination *must not* overlap.
    #[inline(always)] 
    pub unsafe fn move_elements(self, dst: Self, len: SizeType) {
        unsafe {
            self.copy_to_nonoverlapping(*dst, len.into_usize());
        }
    }

    /// Inserts a value of [`T`] to `index` pushing elements on subsequent indices back up to
    /// `len`.
    /// # Safety
    /// [`Self`] needs to point to an aligned array of [`T`]s up to `len` + 1.
    #[inline(always)]
    pub unsafe fn insert_element(
        self, 
        value: T,
        index: SizeType,
        len: SizeType,
    ) -> Pointer<T, SizeType>
    {
        unsafe {
            let res = self.add(index.into_usize());
            res.copy_to((*res).add(1), len.into_usize() - index.into_usize());
            res.write(value);
            res
        }
    }

    /// Drops values up to `len` if [`T`] requires [`Drop`].
    /// # Safety
    /// [`Self`] needs to point to a valid, aligned array of [`T`] which contains *initialized*
    /// values of [`T`] up to `len`.
    #[inline(always)]
    pub unsafe fn drop_in_place(self, len: SizeType) {
        if needs_drop::<T>() {
            unsafe {
                for i in 0..len.into_usize() {
                    self.deref().add(i).drop_in_place();
                }
            }
        }
    }

    /// Clones elements from [`self`] to `dst` up to `len`.
    /// # Safety
    /// [`self`] and `dst` need to be valid, aligned pointers to arrays of [`T`] up to `len` and
    /// [`self`] needs to hold valid, initialized values of [`T`] up to `len`. The source and
    /// destination *must* not overlap.
    #[inline(always)]
    pub unsafe fn clone_elements(self, dst: Self, len: SizeType)
        where T: Clone
    {
        unsafe {
            for i in 0..len.into_usize() {
                dst.deref().add(i).write(self.deref().add(i).as_ref().clone());
            }
        }
    }

    /// Clones elements from [`self`] to `dst` up to `len`.
    /// # Safety
    /// [`self`] and `dst` need to be valid, aligned pointers to arrays of [`T`] up to `len` and
    /// [`self`] needs to hold valid, initialized values of [`T`] up to `len`. The source and
    /// destination *must* not overlap
    pub unsafe fn fast_clone_elements(self, dst: Self, len: SizeType)
        where T: Copy
    {
        unsafe {
            self.copy_to_nonoverlapping(*dst, len.into_usize());
        }
    }
}

impl_traits!{
    for Pointer<T, SizeType: [IntoUsize]>
    Deref =>

        type Target = NonNull<T>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    ,
    DerefMut =>

        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    ,
    From<NonNull<T>> =>
        
        fn from(value: NonNull<T>) -> Self {
            Self(value, PhantomData)
        }
    ,
}

impl<T, SizeType> From<Pointer<T, SizeType>> for NonNull<T>
    where SizeType: IntoUsize
{

    fn from(value: Pointer<T, SizeType>) -> Self {
        *value
    }
}
