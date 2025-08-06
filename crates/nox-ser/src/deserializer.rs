use nox_mem::{Vector, vec_types::GlobalVec};

use super::{Reader, Primitive};

pub struct Deserializer<R>
    where
        R: Reader
{
    reader: R,
}

impl<R: Reader> Deserializer<R> {

    #[inline(always)]
    pub fn deserialize_primitive<const BYTES: usize, P>(
        &mut self,
    ) -> Result<P, R::Error>
        where
            P: Primitive<{BYTES}>
    {
        let mut buf = [0u8; BYTES];
        self.reader.read(&mut buf)?;
        Ok(P::from_le_bytes(buf))
    }

    pub fn deserialize_slice_primitive<const BYTES: usize, P>(
        &mut self,
    ) -> Result<GlobalVec<P>, R::Error>
        where
            P: Primitive<{BYTES}>
    {
        let len: u32 = self.deserialize_primitive()?;
        let mut vec = GlobalVec::<P>
            ::with_capacity(len as usize)
            .unwrap();
        for _ in 0..len {
            vec.push(self.deserialize_primitive()?).unwrap();
        }
        Ok(vec)
    }

    pub fn deserialize_str(
        &mut self
    ) -> Result<String, R::Error>
    {
        let len: u32 = self.deserialize_primitive()?;
        let mut string = String::with_capacity(len as usize);
        let vec = unsafe {
            string.as_mut_vec()
        };
        for _ in 0..len {
            vec.push(self.deserialize_primitive()?);
        }
        Ok(string)
    }
}
