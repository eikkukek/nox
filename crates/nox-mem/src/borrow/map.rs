use core::fmt::{self, Debug, Display};

pub enum CowMutBy<'a, B, O>
    where
        B: ?Sized,
{
    Borrowed(&'a mut B),
    Owned(O),
}

impl<B, O> CowMutBy<'_, B, O>
    where
        B: ?Sized,
{

    /// Returns whether the data is borrowed.
    #[inline(always)]
    pub fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed(_))
    }

    /// Returns whether the data is owned.
    #[inline(always)]
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    #[inline(always)]
    pub fn to_by<T, Fb, Fo>(
        &self,
        map_borrowed: Fb,
        map_owned: Fo,
    ) -> T
        where
            Fb: FnOnce(&B) -> T,
            Fo: FnOnce(&O) -> T,
    {
        match self {
            Self::Borrowed(b) => map_borrowed(b),
            Self::Owned(o) => map_owned(o),
        }
    }

    #[inline(always)]
    pub fn deref_by<T, Fb, Fo>(
        &self,
        map_borrowed: Fb,
        map_owned: Fo,
    ) -> &T
        where
            Fb: FnOnce(&B) -> &T,
            Fo: FnOnce(&O) -> &T,
    {
        match self {
            Self::Borrowed(b) => map_borrowed(b),
            Self::Owned(o) => map_owned(o),
        }
    }

    #[inline(always)]
    pub fn deref_mut_by<T, Fb, Fo>(
        &mut self,
        map_borrowed: Fb,
        map_owned: Fo,
    ) -> &mut T
        where
            Fb: FnOnce(&mut B) -> &mut T,
            Fo: FnOnce(&mut O) -> &mut T,
    {
        match self {
            Self::Borrowed(b) => map_borrowed(b),
            Self::Owned(o) => map_owned(o),
        }
    }
    
    #[inline(always)]
    pub fn to_owned_by<F>(
        &mut self,
        f: F,
    ) -> &mut O
        where F: FnOnce(&mut B) -> O,
    {
        match self {
            Self::Borrowed(b) => {
                *self = Self::Owned(f(b));
                match self {
                    Self::Owned(o) => o,
                    Self::Borrowed(_) => unreachable!(),
                }
            },
            Self::Owned(o) => o,
        }
    }

    #[inline(always)]
    pub fn into_owned_by<F>(
        self,
        f: F,
    ) -> O
        where F: FnOnce(&mut B) -> O,
    {
        match self {
            Self::Borrowed(b) => f(b),
            Self::Owned(o) => o,
        }
    }
}

impl<B, O> Debug for CowMutBy<'_, B, O>
    where
        B: Debug + ?Sized,
        O: Debug,
{
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrowed(b) =>
                f.debug_tuple("Borrowed")
                .field(b)
                .finish(),
            Self::Owned(o) =>
                f.debug_tuple("Owned")
                .field(o)
                .finish(),
        }
    }
}

impl<B, O> Display for CowMutBy<'_, B, O>
    where
        B: Display + ?Sized,
        O: Display,
{

    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrowed(b) => b.fmt(f),
            Self::Owned(o) => o.fmt(f)
        }
    }
}
