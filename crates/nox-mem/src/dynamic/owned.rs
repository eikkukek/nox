use core::{
    ptr::{self, NonNull},
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
};

use crate::{
    alloc::{Layout, LocalAlloc, LocalAllocExt, StdAlloc},
    const_assert,
};

use super::{Dyn, DynRawParts};

pub trait InitKind {

    fn new(size: usize, align: usize) -> Self;

    fn uninit_size_align(&self) -> Option<(usize, usize)>;
}

pub struct Init;

impl InitKind for Init {

    fn new(_size: usize, _align: usize) -> Self {
        Self
    }

    fn uninit_size_align(&self) -> Option<(usize, usize)> {
        None
    }
}

pub struct Uninit(usize, usize);

impl InitKind for Uninit {

    fn new(size: usize, align: usize) -> Self {
        Self(size, align)
    }

    fn uninit_size_align(&self) -> Option<(usize, usize)> {
        Some((self.0, self.1))
    }
}

union Data<T: ?Sized> {
    non_null: NonNull<T>,
    raw_parts: (*const (), *const ()),
}

impl<T: ?Sized> Data<T> {

    pub fn non_null(non_null: NonNull<T>) -> Self {
        Self {
            non_null,
        }
    }

    pub fn just_meta(meta: *const ()) -> Self {
        Self {
            raw_parts: (ptr::null(), meta),
        }
    }
}

pub struct OwnedBase<T, Kind>
    where 
        T: ?Sized,
        Kind: InitKind,
{
    data: Data<T>,
    kind: Kind,
}

const_assert!(size_of::<Owned<dyn core::any::Any>>() == 16);
const_assert!(size_of::<OwnedMaybeUninit<dyn core::any::Any>>() == 32);

impl<T, Kind> OwnedBase<T, Kind>
    where 
        T: ?Sized,
        Kind: InitKind,
{

    pub fn new<Source>(t: Source) -> Self
        where 
            Source: Dyn<T, Target = Source>,
    {
        unsafe {
            let meta = Source::raw_parts(&t).meta;
            let size = size_of::<Source>();
            if size == 0 {
                let _ = MaybeUninit::new(t);
                return Self {
                    data: Data::just_meta(meta),
                    kind: InitKind::new(0, 0),
                }
            }
            let data = StdAlloc
                .allocate_uninit(1)
                .expect("global alloc failed");
            data.write(t);
            Self {
                data: Data::non_null(NonNull::new(Source::from_raw_parts(
                    DynRawParts { data: data.as_ptr(), meta }
                ).cast_mut()).unwrap()),
                kind: InitKind::new(size, align_of::<Source>()),
            }
        }
    }

    #[inline(always)]
    fn as_ptr_internal(&self) -> *const T {
        unsafe {
            if self.data.raw_parts.0.is_null() {
                let addr = &self.data;
                let data = Data {
                    raw_parts: (addr as *const _ as *const (), self.data.raw_parts.1),
                };
                data.non_null.as_ptr()
            } else {
                self.data.non_null.as_ptr()
            }
        }
    }

    #[inline(always)]
    fn as_ref_internal(&self) -> &T {
        unsafe {
            self.as_ptr_internal()
                .as_ref()
                .unwrap()
        }
    }

    #[inline(always)]
    fn as_mut_internal(&mut self) -> &mut T {
        unsafe {
            self.as_ptr_internal()
                .cast_mut()
                .as_mut()
                .unwrap()
        }
    }

    #[inline(always)]
    fn as_source_ptr_internal<Source>(&self) -> Option<*const Source>
        where 
            T: 'static,
            Source: Dyn<T>,
    {
        unsafe {
            let meta = Source::meta();
            if self.data.raw_parts.1 != meta {
                return None
            }
            if self.data.raw_parts.0.is_null() {
                let addr = &self.data;
                Some(mem::transmute::<*const Data<T>, *const Source>(addr))
            } else {
                let non_null = self.data.non_null;
                Some(Source
                    ::get_self(non_null.as_ptr())
                )
            }
        }
    }

    #[inline(always)]
    fn as_source_ref_internal<Source>(&self) -> Option<&Source>
        where
            T: 'static,
            Source: Dyn<T>,
    {
        unsafe {
            self.as_source_ptr_internal::<Source>()
                .map(|v| v.as_ref().unwrap())
        }
    }

    #[inline(always)]
    fn as_source_mut_internal<Source>(&mut self) -> Option<&mut Source>
        where
            T: 'static,
            Source: Dyn<T>,
    {
        unsafe {
            self.as_source_ptr_internal::<Source>()
                .map(|v| v.cast_mut().as_mut().unwrap())
        }
    }

    #[inline(always)]
    pub fn take_source_internal<Source>(self) -> Option<Source>
        where
            T: 'static,
            Source: Dyn<T>,
    {
        unsafe {
            let meta = Source::meta();
            if self.data.raw_parts.1 != meta {
                return None
            }
            if self.data.raw_parts.0.is_null() {
                let mut unit = ();
                let value = mem
                    ::transmute::<*mut (), *const Source>(&mut unit)
                    .read();
                let _ = MaybeUninit::new(self);
                Some(value)
            } else {
                let non_null = self.data.non_null;
                let value = Source
                    ::get_self(non_null.as_ptr())
                    .read();
                let size = size_of_val(&value);
                let align = align_of_val(&value);
                StdAlloc.free_raw(
                    non_null.cast(),
                    Layout::from_size_align(size, align).unwrap(),
                );
                let _ = MaybeUninit::new(self);
                Some(value)
            }
        }
    }
}

pub type Owned<T> = OwnedBase<T, Init>;

impl<T: ?Sized> Owned<T> { 

    /// If the meta of [`Source`] matches the meta of [`T`], this function takes a reference
    /// of the value and returns [`&Source`], otherwise it returns [`None`].
    pub fn as_source_ref<Source>(&self) -> Option<&Source>
        where
            T: 'static,
            Source: Dyn<T>,
    {
        self.as_source_ref_internal()
    }


    /// If the meta of [`Source`] matches the meta of [`T`], this function takes a mutable
    /// reference of the value
    /// and returns [`&mut Source`], otherwise it returns [`None`].
    pub fn as_mut<Source>(&mut self) -> Option<&mut Source>
        where
            T: 'static,
            Source: Dyn<T>,
    {
        self.as_source_mut_internal()
    }

    /// If the meta of [`Source`] matches the meta of [`T`], this function moves the value,
    /// deallocates memory and returns [`Source`], otherwise it returns [`None`].
    pub fn take_source<Source>(self) -> Option<Source>
        where
            T: 'static,
            Source: Dyn<T>,
    {
        self.take_source_internal()
    }
}

pub type OwnedMaybeUninit<T> = OwnedBase<T, Uninit>;

impl<T: ?Sized> OwnedMaybeUninit<T> {

    pub fn uninit<Source>() -> Self
        where 
            Source: Dyn<T>
    {
        unsafe {
            let meta = Source::meta();
            let size = size_of::<Source>();
            if size == 0 {
                return Self {
                    data: Data::just_meta(meta),
                    kind: InitKind::new(0, 0),
                }
            }
            let data = StdAlloc
                .allocate_uninit(1)
                .expect("global alloc failed");
            Self {
                data: Data::non_null(NonNull::new(Source::from_raw_parts(
                    DynRawParts { data: data.as_ptr(), meta, }
                ).cast_mut()).unwrap()),
                kind: InitKind::new(size, align_of::<Source>()),
            }
        }
    }

    pub fn write<Source>(
        &mut self,
        value: Source,
    ) -> Option<&mut Source>
        where 
            T: 'static,
            Source: Dyn<T>,
    {
        unsafe {
            let ptr = self
                .as_source_ptr_internal::<Source>()?
                .cast_mut();
            ptr.write(value);
            ptr.as_mut()
        }
    }

    pub unsafe fn assume_init_drop(
        &mut self,
    ) {
        unsafe {
            self.as_ptr_internal()
                .cast_mut()
                .drop_in_place();
        }
    }

    pub unsafe fn assume_init_ref(
        &mut self,
    ) -> &T
    {
        self.as_ref_internal()
    }

    pub unsafe fn assume_init_mut(
        &mut self
    ) -> &mut T
    {
        self.as_mut_internal()
    }

    pub unsafe fn assume_init_source<Source>(self) -> Option<Source>
        where
            T: 'static,
            Source: Dyn<T>,
    {
        self.take_source_internal()
    }

    pub unsafe fn assume_init_source_ref<Source>(&self) -> Option<&Source>
        where 
            T: 'static,
            Source: Dyn<T>,
    {
        self.as_source_ref_internal()
    }

    pub unsafe fn assume_init_source_mut<Source>(&mut self) -> Option<&mut Source>
        where 
            T: 'static,
            Source: Dyn<T>,
    {
        self.as_source_mut_internal()
    }
}

unsafe impl<T, Kind> Send for OwnedBase<T, Kind>
    where 
        T: ?Sized + Send,
        Kind: InitKind,
{}

unsafe impl<T, Kind> Sync for OwnedBase<T, Kind>
    where 
        T: ?Sized + Sync,
        Kind: InitKind,
{}

impl<T: ?Sized> Deref for Owned<T> {

    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref_internal()
    }
}

impl<T: ?Sized> DerefMut for Owned<T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_internal()
    }
}

impl<T: ?Sized> AsRef<T> for Owned<T> {

    fn as_ref(&self) -> &T {
        self.as_ref_internal()
    }
}

impl<T: ?Sized> AsMut<T> for Owned<T> {

    fn as_mut(&mut self) -> &mut T {
        self.as_mut_internal()
    }
}

impl<T, Kind> Drop for OwnedBase<T, Kind>
    where 
        T: ?Sized,
        Kind: InitKind,
{

    fn drop(&mut self) {
        unsafe {
            if self.data.raw_parts.0.is_null() {
                if self.kind.uninit_size_align().is_none() {
                    let addr = &self.data;
                    let data = Data::<T> {
                        raw_parts: (addr as *const _ as *const (), self.data.raw_parts.1),
                    };
                    data.non_null.drop_in_place();
                }
            } else {
                let (size, align) =
                if let Some((size, align)) = self.kind.uninit_size_align() {
                    (size, align)
                } else {
                    let data_mut = self.as_mut_internal();
                    let size = size_of_val(data_mut);
                    let align = align_of_val(data_mut);
                    self.data.non_null.drop_in_place();
                    (size, align)
                };
                let ptr = self.data.non_null.cast();
                StdAlloc.free_raw(
                    ptr,
                    Layout::from_size_align(size, align).unwrap(),
                );
            }
        }
    }
}
