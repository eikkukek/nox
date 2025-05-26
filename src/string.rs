use std::fmt::Write;
use std::ffi::CStr;
use std::cmp::PartialEq;

#[derive(Copy, Clone)]
pub struct String<const N: usize> {
string: [u8; N],
len: usize,
}

impl<const N: usize> String<N> {

pub fn new() -> Self {
    Self { string: [0u8; N], len: 0 }
}

pub fn from_str(s: &str) -> Self {
    let mut string = [0u8; N];
    let mut end = s.len().min(N);
    while !s.is_char_boundary(end) && end > 0 {
        end -= 1;
    }
    let slice = &s[..end];
    string[..end].copy_from_slice(slice.as_bytes());
    Self {
        string,
        len: end,
    }
}

pub fn from_ascii<const N_BUF: usize>(s: &[i8; N_BUF]) -> Result<Self, SmallError> {
    let mut string = [0u8; N];
    let mut len = 0usize;
    for c in *s {
        if c < 0 {
            return Err(
                String::from_str("invalid ascii"),
            );
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

pub unsafe fn from_ascii_ptr(s: *const i8) -> Result<Self, SmallError> {
    let mut string = [0u8; N];
    let mut len = 0usize;
    let mut c = unsafe { *s };
    while c != b'\0' as i8  && len != N {
        if c < 0 {
            return Err(
                String::from_str("invalid ascii"),
            );
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

pub fn as_str(&self) -> &str {
    std::str::from_utf8(&self.string[..self.len]).unwrap_or("<invalid utf-8>")
}

pub fn as_bytes(&self) -> &[u8] {
    &self.string[..self.len]
}

pub fn format(args: std::fmt::Arguments<'_>) -> Self {
    let mut s = Self { string: [0u8; N], len: 0 };
    s.write_fmt(args).expect("this shouldn't happen");
        s
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<const N: usize, const M: usize> PartialEq<String::<M>> for String::<N> {

    fn eq(&self, other: &String<M>) -> bool {
        return self.as_bytes() == other.as_bytes();
    }
}

impl<const N: usize> std::fmt::Debug for String<N> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "String<{}>({:?})", N, self.as_str())
    }
}

impl<const N: usize> std::fmt::Display for String<N> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> std::fmt::Write for String<N> {

    fn write_str(&mut self, s: &str) -> std::fmt::Result {
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

pub fn cstr_eq(a: &CStr, b: &CStr) -> bool {
    a.to_bytes() == b.to_bytes()
}

pub type SmallError = String<64>;
pub type LargeError = String<256>;
