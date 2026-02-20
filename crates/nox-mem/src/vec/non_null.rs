use core::{
    marker::PhantomData,
    ptr::NonNull,
    ops::{Deref, DerefMut},
    mem::ManuallyDrop,
    fmt::{self, Display, Formatter}
};

use crate::{
    alloc::{Layout, LocalAlloc},
    collections::{ReservePolicy, TryReserveError},
    impl_traits,
    num::{FromUsize, IntoUsize},
    slice,
    conditional::{Conditional, True, False},
};

use super::{AllocVecBase, Vector, Pointer};

pub struct NonNullPolicy<SizeType>(PhantomData<SizeType>);

pub struct NonNullAlloc<'a>(PhantomData<&'a ()>);

unsafe impl<'a> LocalAlloc for NonNullAlloc<'a> {

    type Error = TryReserveError<()>;

    #[inline(always)]
    unsafe fn allocate_raw(&self, _layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        Err(TryReserveError::max_capacity_exceeded(0, 0, ()))
    }

    #[inline(always)]
    unsafe fn free_raw(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

unsafe impl<SizeType> ReservePolicy<SizeType> for NonNullPolicy<SizeType>
    where SizeType: IntoUsize,
{

    fn can_grow() -> bool {
        false
    }

    fn grow(
        current: SizeType, 
        required: usize,
    ) -> Result<SizeType, TryReserveError<()>> {
        if required > current.into_usize() {
            Err(TryReserveError::max_capacity_exceeded(current, required, ()))
        } else {
            Ok(current)
        }
    }

    fn grow_infallible(current: SizeType, required: usize) -> SizeType {
        if required > current.into_usize() {
            panic!("maximum capacity of {current} exceeded")
        } else {
            current
        }
    }
}

#[derive(Debug, Hash)]
pub struct NonNullVecBase<'a, T, SizeType, Clonable = False>
    where 
        SizeType: IntoUsize + FromUsize,
        Clonable: Conditional,
{
    inner: ManuallyDrop<AllocVecBase<T, SizeType, NonNullAlloc<'a>, NonNullPolicy<SizeType>>>,
    _marker: PhantomData<(Clonable, *const ())>,
}

impl<'a, T, SizeType> NonNullVecBase<'a, T, SizeType>
    where SizeType: IntoUsize + FromUsize
{

    /// Creates a new [`NonNullVec`].
    ///
    /// # Safety
    /// This is unsafe because you need to ensure that the pointer is valid for vector operations
    /// up to `capacity` for the duration the vector is used.
    #[inline(always)]
    pub unsafe fn new(
        ptr: NonNull<T>,
        capacity: SizeType,
    ) -> Self
    {
        unsafe {
            Self {
                inner: ManuallyDrop::new(
                    AllocVecBase::from_raw_parts(
                        ptr.into(),
                        SizeType::ZERO,
                        capacity,
                        NonNullAlloc(PhantomData)
                    )
                ),
                _marker: PhantomData,
            }
        }
    }

    /// Creates a [`NonNullVec`] with the given capacity, allocting its memory from [`Alloc`]. The
    /// memory can be freed by getting the pointer back with [`NonNullVec::as_non_null()`].
    #[inline(always)]
    pub fn with_capacity<Alloc: LocalAlloc + ?Sized>(
        capacity: SizeType,
        alloc: &'a Alloc,
    ) -> Result<Self, Alloc::Error>
    {
        unsafe {
            let ptr = alloc.allocate_raw(
                Layout::from_size_align_unchecked(
                    capacity.into_usize() * size_of::<T>(),
                    align_of::<T>(),
                )
            )?;
            Ok(Self::new(ptr.cast(), capacity))
        }
    }
}

impl<'a, T, SizeType, Clonable> NonNullVecBase<'a, T, SizeType, Clonable>
    where
        SizeType: IntoUsize + FromUsize,
        Clonable: Conditional,
{

    /// Consumes self and returns [`NonNullVec`] with the given length.
    ///
    /// # Safety
    /// `len` needs to be less than or equal to capacity and values up to `len` need to be valid
    /// values of [`T`].
    pub unsafe fn with_len(mut self, len: SizeType) -> Self {
        unsafe {
            self.set_len(len);
            self
        }
    }

    #[inline(always)]
    pub fn into_static(self) -> NonNullVecBase<'static, T, SizeType> {
        unsafe {
            NonNullVecBase {
                inner: ManuallyDrop::new(
                    ManuallyDrop::into_inner(self.inner).with_alloc(NonNullAlloc(PhantomData))
                ),
                _marker: PhantomData,
            }
        }
    }
    
    /// Makes the [`NonNullVec`] trivially clonable.
    ///
    /// # Safety
    /// It needs to be guaranteed that only one of the resulting vectors is ever used if vector
    /// operations are required.
    #[inline(always)]
    pub unsafe fn into_clonable(self) -> NonNullVecBase<'a, T, SizeType, True> {
        unsafe {
            NonNullVecBase {
                inner: ManuallyDrop::new(
                    ManuallyDrop::into_inner(self.inner).with_alloc(NonNullAlloc(PhantomData))
                ),
                _marker: PhantomData,
            }

        }
    }

    /// Drops all elements of the vector and sets len and capacity to zero.
    ///
    /// Since [`NonNullVec`] doesn't implement [`Drop`], this can be used when you would drop the
    /// vector.
    ///
    /// # Safety
    /// The inner pointer needs to be valid up to `len`.
    pub unsafe fn drop_in_place(&mut self) {
        self.clear();
        unsafe {
            self.set_capacity(SizeType::ZERO);
        }
    }

    /// Drops all elements and frees the memory with the given allocator.
    ///
    /// # Safety
    /// It has to be ensured that the inner pointer resulted from an allocation from the same
    /// allocator, with the size of [`size_of<T>`] * capacity and align of `T`.
    pub unsafe fn drop_and_free<Alloc>(&mut self, alloc: &Alloc)
        where Alloc: LocalAlloc + ?Sized
    {
        self.clear();
        unsafe {
            alloc.free_raw(
                NonNull::new(self.as_mut_ptr()).unwrap().cast(),
                Layout::from_size_align_unchecked(
                    self.capacity().into_usize() * size_of::<T>(),
                    align_of::<T>(),
                ),
            );
            self.set_capacity(SizeType::ZERO);
        }
    }

    /// Consumes self and returns the inner pointer.
    pub fn into_inner(self) -> NonNull<T> {
        ManuallyDrop::into_inner(self.inner).into_inner().into()
    }
}

/// A wrapper around a raw pointer, which is interpreted as a vector.
///
/// This can be useful when you have a raw pointer which you want to use like a vector, for example
/// with custom allocators.
///
/// It has the property [`size_of<Vec> == size_of<NonNullVec>`].
///
/// # Examples
/// ``` rust
/// use nox_mem::alloc::{StdAlloc, LocalAllocExt};
/// use nox_mem::vec::{NonNullVec, Vector};
/// unsafe {
///     let ptr = StdAlloc.allocate_uninit(4).unwrap();
///     let mut vec = NonNullVec::new(ptr, 4);
///     vec.push("foo".to_string());
///     vec.push("bar".to_string());
///     vec.drop_in_place();
///     assert_eq!(
///         vec.try_push("hello".to_string()).unwrap_err().recover_value().0,
///         "hello",
///     );
///     StdAlloc.free_uninit(ptr, 4);
/// }
/// ```
pub type NonNullVec<'a, T, Clonable = False> = NonNullVecBase<'a, T, usize, Clonable>;

/// A wrapper around a raw pointer, which is interpreted as a vector.
///
/// Unlike [`NonNullVec`], this stores its capacity and length as [`u32`].
///
/// It has the property [`size_of<Vec32> == size_of<NonNullVec32>`].
///
/// See [`NonNullVec`] for examples and full description.
pub type NonNullVec32<'a, T, Clonable = False> = NonNullVecBase<'a, T, u32, Clonable>;

impl_traits! {
    for NonNullVecBase<'a, T, SizeType: [IntoUsize + FromUsize], Clonable: [Conditional]>
    Default =>

        fn default() -> Self {
            unsafe {
                Self {
                    inner: ManuallyDrop::new(AllocVecBase::from_raw_parts(
                        Pointer::dangling(),
                        SizeType::ZERO,
                        SizeType::ZERO,
                        NonNullAlloc(PhantomData),
                    )),
                    _marker: PhantomData,
                }
            }
        }
    ,
    Deref =>
        type Target = AllocVecBase<T, SizeType, NonNullAlloc<'a>, NonNullPolicy<SizeType>>;

        #[inline(always)]
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    ,
    DerefMut =>

        #[inline(always)]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    ,
    AsRef<[T]> =>

        fn as_ref(&self) -> &[T] {
            self.as_slice()
        }
    ,
    AsMut<[T]> =>

        fn as_mut(&mut self) -> &mut [T] {
            self.as_mut_slice()
        }
    ,
    IntoIterator for &'vec => 

        type Item = &'vec T;
        type IntoIter = slice::Iter<'vec, T>;

        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    ,
    IntoIterator for mut &'vec => 

        type Item = &'vec mut T;
        type IntoIter = slice::IterMut<'vec, T>;

        fn into_iter(self) -> Self::IntoIter {
            self.iter_mut()
        }
    ,
}

unsafe impl<T, SizeType> Send for NonNullVecBase<'static, T, SizeType, False> 
    where
        SizeType: FromUsize + IntoUsize,
        T: Send,
{}

unsafe impl<T, SizeType> Sync for NonNullVecBase<'static, T, SizeType, False> 
    where
        SizeType: FromUsize + IntoUsize,
        T: Sync,
{}

impl<'a, T, SizeType> Clone for NonNullVecBase<'a, T, SizeType, True>
    where SizeType: FromUsize + IntoUsize
{

    fn clone(&self) -> Self {
        unsafe { Self {
            inner: ManuallyDrop::new(AllocVecBase::from_raw_parts(
                Pointer::new(self.as_ptr().cast_mut()).unwrap(),
                self.len(),
                self.capacity(),
                NonNullAlloc(PhantomData),
            )),
            _marker: PhantomData,
        } }
    }
}

impl<'a, T, SizeType> Display for NonNullVecBase<'a, T, SizeType>
    where
        SizeType: IntoUsize + FromUsize,
        AllocVecBase<T, SizeType, NonNullAlloc<'a>, NonNullPolicy<SizeType>>: Display,
{

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
