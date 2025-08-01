pub trait Writer {

    type Error;

    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}
