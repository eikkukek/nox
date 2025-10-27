use core::ops::{Deref, DerefMut};

use nox::mem::impl_traits;

pub struct Face<'a> {
    pub face: ttf_parser::Face<'a>,
    pub hb_font: harfbuzz_rs::Owned<harfbuzz_rs::Font<'a>>,
}

impl<'a> Face<'a> {

    pub fn parse(
        data: &'a [u8],
        index: u32,
    ) -> Result<Self, ttf_parser::FaceParsingError>
    {
        Ok(Self {
            face: ttf_parser::Face::parse(data, index)?,
            hb_font: harfbuzz_rs::Font::new(harfbuzz_rs::Face::from_bytes(data, index)),
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
