use std::io;
use super::{Reader, Writer};

pub trait Ser {

    fn ser<W: Writer>(&self, writer: &mut W) -> io::Result<()>;
}

pub trait Deser: Sized + Ser {

    type Error: From<io::Error>;

    fn deser<R: Reader>(reader: &mut R) -> Result<Self, Self::Error>;
}

macro_rules! impl_ser_le {
    ($($t:ty),*) => {
       $(
            impl Ser for $t {
                #[inline(always)]
                fn ser<W: Writer>(&self, writer: &mut W) -> std::io::Result<()> {
                    writer.write(&self.to_le_bytes(), align_of::<Self>())
                }
            }
       )*
    };
}

macro_rules! impl_deser_le {
    ($($t:ty),*) => {
       $(
            impl Deser for $t {
                type Error = std::io::Error;
                #[inline(always)]
                fn deser<R: Reader>(reader: &mut R) -> Result<Self, Self::Error> {
                    let mut buf = [0u8; size_of::<Self>()];
                    reader.read(&mut buf, align_of::<Self>())?;
                    Ok(Self::from_le_bytes(buf))
                }
            }
       )*
    };
}

impl_ser_le!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
impl_deser_le!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

impl Ser for &str {

    fn ser<W: Writer>(&self, writer: &mut W) -> io::Result<()> {
        let len = self.len() as u32;
        len.ser(writer)?;
        writer.write(self.as_bytes(), 1)
    }
}

impl<T: Ser> Ser for [T] {

    fn ser<W: Writer>(&self, writer: &mut W) -> io::Result<()> {
        let len = self.len() as u32;
        len.ser(writer)?;
        for value in self {
            value.ser(writer)?;
        }
        Ok(())
    }
}

fn test<W: Writer, V: crate::vec_types::Vector<u32>>(writer: &mut W) {

    let a = [""; 10];
    a.ser(writer).unwrap();
    let vec = vec![10, 10, 10];
    vec.ser(writer).unwrap();
    let vec2 = crate::vec_types::GlobalVec::<&str>::new().unwrap();
    vec2.ser(writer).unwrap();
}
