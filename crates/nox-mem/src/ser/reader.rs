use std::io;

pub trait Reader {

    fn read(&mut self, buf: &mut [u8], align: usize) -> io::Result<()>;
}
