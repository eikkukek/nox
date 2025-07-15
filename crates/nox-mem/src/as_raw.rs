pub trait AsRaw {

    type Repr;

    fn as_raw(self) -> Self::Repr;
}
