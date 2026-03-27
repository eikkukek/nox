use core::{
    fmt::{self, Write, Display, Debug, Formatter},
    cmp::PartialEq,
    hash::Hash,
    ffi::c_char,
};

use super::StringError;

#[derive(Copy, Clone)]
pub struct ArrayString<const N: usize> {
    string: [u8; N],
    len: usize,
}

impl<const N: usize> ArrayString<N> {

    pub fn new<T>(text: T) -> Self
        where 
            T: AsRef<str>,
    {
        let text = text.as_ref();
        let mut string = [0u8; N];
        let mut end = text.len().min(N);
        while !text.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        let slice = &text[..end];
        string[..end].copy_from_slice(slice.as_bytes());
        Self {
            string,
            len: end,
        }
    }
    
    /// Constructs an [`ArrayString`] from a [`c_char`] slice.
    pub fn from_c_char_slice<const N_BUF: usize>(
        text: &[c_char; N_BUF]
    ) -> Result<Self, StringError>
    {
        let mut string = [0u8; N];
        let mut len = 0usize;
        for (i, &c) in text.iter().enumerate() {
            if c < 0 {
                return Err(StringError::Utf8Error(i));
            }
            if c == b'\0' as i8 || len == N {
                break;
            }
            string[len] = c as u8;
            len += 1;
        }
        Ok(
            Self {
                string,
                len,
            }
        )
    }
   
    /// Constructs an [`ArrayString`] from a null terminated c_char pointer.
    /// # Safety
    /// The pointer point to a valid null terminated array of [`c_char`]s.
    pub unsafe fn from_c_char_ptr(s: *const c_char) -> Result<Self, StringError> {
        let mut string = [0u8; N];
        let mut len = 0usize;
        let mut c = unsafe { *s };
        while c != b'\0' as i8  && len != N {
            if c < 0 {
                return Err(StringError::Utf8Error(len));
            }
            string[len] = c as u8;
            len += 1;
            c = unsafe { *s.add(len) }
        }
        Ok(
            Self {
                string,
                len,
            }
        )
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.string[..self.len]).unwrap()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.string[..self.len]
    }

    #[inline]
    pub fn format(args: fmt::Arguments<'_>) -> Self {
        let mut s = Self { string: [0u8; N], len: 0 };
        s.write_fmt(args).expect("this shouldn't happen");
        s
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const N: usize> Default for ArrayString<N> {

    fn default() -> Self {
        Self {
            string: [0u8; N],
            len: 0,
        }
    }
}

impl<const N: usize> AsRef<str> for ArrayString<N> {

    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize, const M: usize> PartialEq<ArrayString::<M>> for ArrayString::<N> {

    #[inline]
    fn eq(&self, other: &ArrayString<M>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const N: usize> Eq for ArrayString<N> {}

impl<const N: usize> Hash for ArrayString<N> {

    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<const N: usize> Debug for ArrayString<N> {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl<const N: usize> Display for ArrayString<N> {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> Write for ArrayString<N> {

    fn write_str(&mut self, s: &str) -> fmt::Result {
        let available = N - self.len;
        let mut end = s.len().min(available);
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        let slice = &s[..end];
        self.string[self.len..self.len + end].copy_from_slice(slice.as_bytes());
        self.len += end;
        Ok(())
    }
}
