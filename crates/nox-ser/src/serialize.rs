use crate::{Writer, Serializer};

pub trait Serialize {

    fn serialize<W: Writer>(serializer: &mut Serializer<W>);
}
