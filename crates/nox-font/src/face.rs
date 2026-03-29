use core::{
    ops::{Deref, DerefMut},
    borrow::Borrow,
};

use nox::{
    mem::{alloc::{StdAlloc, LocalAllocExt}},
    error::*,
};

pub struct Face<'a> {
    pub data: &'a [u8],
    pub face: ttf_parser::Face<'a>,
    pub hb_font: harfbuzz_rs::Owned<harfbuzz_rs::Font<'a>>,
}

pub fn parse_face<'a>(
    data: &'a [u8],
    index: u32,
) -> Result<Face<'a>> {
    Ok(Face {
        data,
        face: ttf_parser::Face::parse(data, index).context("failed to parse face")?,
        hb_font: harfbuzz_rs::Font::new(harfbuzz_rs::Face::from_bytes(data, index)),
    })
}

pub struct OwnedFace {
    face: Box<Face<'static>>,
    _data: Box<[u8]>,
}

pub fn parse_owned_face(
    data: Box<[u8]>,
    index: u32,
) -> Result<OwnedFace> {
    let face = parse_face(&data, index);
    let ptr = unsafe {
        StdAlloc.alloc_uninit(1)
    }.expect("global alloc failed");
    unsafe {
        ptr.write(face);
    }
    let ptr: *mut Face<'static> = ptr.as_ptr().cast();
    let face = unsafe {
        Box::from_raw(ptr)
    };
    Ok(OwnedFace {
        face,
        _data: data,
    })
}

impl<'a> Deref for Face<'a> {

    type Target = ttf_parser::Face<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.face
    }
}

impl<'a> DerefMut for Face<'a> {

    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.face
    }
}

impl Borrow<Face<'static>> for OwnedFace {
 
    #[inline]
    fn borrow(&self) -> &Face<'static> {
        &self.face
    }
}
