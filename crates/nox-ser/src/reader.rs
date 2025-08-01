pub trait Reader {

    type Error;

    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error>;
}
