use super::{Writer, Primitive};

pub struct Serializer<W>
    where
        W: Writer
{
    writer: W,
}

impl<W: Writer> Serializer<W> {

    #[inline(always)]
    pub fn serialize_primitive<const BYTES: usize, P>(
        &mut self,
        value: P
    ) -> Result<(), W::Error>
        where
            P: Primitive<{BYTES}>
    {
        self.writer.write(&value.to_le_bytes())
    }

    pub fn serialize_slice_primitive<const BYTES: usize, P>(
        &mut self,
        slice: &[P]
    ) -> Result<(), W::Error>
        where
            P: Primitive<{BYTES}>
    {
        self.serialize_primitive(slice.len() as u32)?;
        for p in slice {
            self.writer.write(&p.to_le_bytes())?;
        }
        Ok(())
    }

    #[inline(always)]
    pub fn serialize_str(
        &mut self,
        str: &str
    ) -> Result<(), W::Error>
    {
        self.serialize_slice_primitive(str.as_bytes())
    }
}
