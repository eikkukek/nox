use core::ops::{Deref, DerefMut};

use nox::mem::impl_traits;

#[derive(Clone)]
pub struct Face<'a> {
    face: ttf_parser::Face<'a>,
}

impl<'a> Face<'a> {

    pub fn parse(
        data: &'a [u8],
        index: u32,
    ) -> Result<Self, ttf_parser::FaceParsingError>
    {
        Ok(Self {
            face: ttf_parser::Face::parse(data, index)?,
        })
    }
}

impl_traits!(for Face<'a>
    Deref =>

        type Target = ttf_parser::Face<'a>;

        fn deref(&self) -> &Self::Target {
            &self.face
        }       
    ,
    DerefMut =>
        
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.face
        }
    ,
);
