use std::{
    string::String,
    io::{self, Read, Write},
};

use super::is_pod;

/// Trait for writing types in little-endian format.
/// # Example (Manual Implementation)
/// ```
///
/// use std::io::{self, Write};
/// use nox::serialization::WriteLe;
///
/// struct MyU32(u32);
///
/// impl WriteLe for MyU32 {
///     
///     fn write_le<W: Write>(&self, writer: &mut W) -> io::Result<()> {
///         writer.write_all(&self.0.to_le_bytes())
///     }
/// }
/// ```
pub trait WriteLe {

    /// Writes data in little-endian format.
    /// # Example
    /// ```no_run
    ///
    /// use std::fs::{File};
    /// use nox::serialization::{ReadLe, WriteLe};
    ///
    /// let mut file = File::create("data").unwrap();
    /// 1234u32.write_le(&mut file).unwrap();
    /// ```
    fn write_le<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

/// Trait for reading types in little-endian format.
/// # Contract
/// Assumes data was written using the matching [`WriteLe`] implementation.
/// Behaviour is undefined if the input bytes don't match the expected layout or endianness.
/// # Example (Manual Implementation)
/// ```
///
/// use std::io::{self, Write, Read};
/// use nox::serialization::{WriteLe, ReadLe};
///
/// struct MyU32(u32);
///
/// impl WriteLe for MyU32 {
///
///     fn write_le<W: Write>(self, writer: &mut W) -> io::Result<()> {
///         writer.write_all(&self.0.to_le_bytes())
///     }
/// }
///
/// struct MyReadError(io::Error);
///
/// impl From<io::Error> for MyReadError {
///
///     fn from(value: io::Error) -> Self {
///         Self(value)
///     }
/// }
///
/// impl ReadLe for MyU32 {
///
///     type Error = MyReadError;
///
///     fn read_le<R: Read>(reader: &mut R) -> Result<Self, Self::Error> {
///         let mut buf = [0u8; 4];
///         reader.read_exact(&mut buf)?;
///         Ok(MyU32(u32::from_le_bytes(buf)))
///     }
/// }
/// ```
pub trait ReadLe: Sized + WriteLe {

    type Error: From<io::Error>;

    /// Reads data in little-endian format.
    /// # Contract
    /// Assumes data was written using the matching [`WriteLe`] implementation.
    /// Behaviour is undefined if the input bytes don't match the expected layout or endianness.
    /// # Example
    /// ```no_run
    ///
    /// use std::fs::{File};
    /// use std::io::{Seek, SeekFrom};
    /// use nox::serialization::{ReadLe, WriteLe};
    ///
    /// let mut file = File::create("data").unwrap();
    /// 1234u32.write_le(&mut file).unwrap();
    ///
    /// file.seek(SeekFrom::Start(0)).unwrap();
    /// let value = u32::read_le(&mut file).unwrap();
    /// assert_eq!(value, 1234);
    /// ```
    fn read_le<R: Read>(reader: &mut R) -> Result<Self, Self::Error>;
}

macro_rules! impl_write_le {
    ($($t:ty),*) => {
       $(

            impl WriteLe for $t {

                #[inline(always)]
                fn write_le<W: Write>(&self, writer: &mut W) -> io::Result<()> {
                    writer.write_all(&mut self.to_le_bytes())
                }
            }
       )*
    };
}

macro_rules! impl_read_le {
    ($($t:ty),*) => {
       $(
            impl ReadLe for $t {
                type Error = io::Error;
                #[inline(always)]
                fn read_le<R: Read>(reader: &mut R) -> Result<Self, Self::Error> {
                    let mut buf = [0u8; std::mem::size_of::<$t>()];
                    reader.read_exact(&mut buf)?;
                    Ok(<$t>::from_le_bytes(buf))
                }
            }
       )*
    };
}

impl_write_le!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
impl_read_le!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

impl WriteLe for String {

    fn write_le<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        let bytes = self.as_bytes();
        let len = bytes.len() as u32;
        writer.write_all(&len.to_le_bytes())?;
        writer.write_all(&bytes)?;
        Ok(())
    }
}

impl ReadLe for String {

    type Error = io::Error;

    fn read_le<R: Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut len_buf = [0u8; std::mem::size_of::<u32>()];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut s = String::with_capacity(len);
        unsafe {
            let buf = s.as_mut_vec();
            buf.set_len(len);
            reader.read_exact(buf)?;
        }
        Ok(s)
    }
}

impl<T: WriteLe> WriteLe for Vec<T> {

    default fn write_le<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        let len = self.len() as u32;
        len.write_le(writer)?;
        if is_pod::<T>() {
            unsafe {
                let slice = core::slice::from_raw_parts(
                    self.as_ptr() as *const u8,
                    len as usize * size_of::<T>()
                );
                writer.write_all(slice)?;
            }
        }
        else {
            for value in self {
                value.write_le(writer)?;
            }
        }
        Ok(())
    }
}

impl<T: ReadLe> ReadLe for Vec<T> {

    type Error = T::Error;

    fn read_le<R: Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let len = u32::read_le(reader)? as usize;
        let mut v = Vec::with_capacity(len);
        if is_pod::<T>() {
            unsafe {
                let slice = core::slice::from_raw_parts_mut(
                    v.as_mut_ptr() as *mut u8,
                    len * size_of::<T>(),
                );
                reader.read_exact(slice)?;
                v.set_len(len);
            }
        }
        else {
            for _ in 0..len {
                v.push(T::read_le(reader)?);
            }
        }
        Ok(v)
    }
}

impl WriteLe for blake3::Hash {

    fn write_le<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(self.as_bytes())
    }
}

impl ReadLe for blake3::Hash {

    type Error = io::Error;

    fn read_le<R: Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut bytes = [0u8; 32];
        reader.read_exact(&mut bytes)?;
        // SAFETY: bytes.as_slice will always be exactly 32 bytes long
        unsafe {
            Ok(Self::from_slice(bytes.as_slice()).unwrap_unchecked())
        }
    }
}
