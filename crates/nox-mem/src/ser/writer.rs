use std::io;

pub trait Writer {

    fn write(&mut self, buf: &[u8], align: usize) -> io::Result<()>;
}
