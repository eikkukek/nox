use std::sync::Arc;

use core::slice;

use nox::mem::vec_types::GlobalVec;

use super::*;

#[derive(Clone)]
pub struct VertexTextRenderer<'a> {
    trigs: GlobalVec<Option<Arc<GlyphTriangles>>>,
    offsets: GlobalVec<Option<GlobalVec<VertexOffset>>>,
    face: Face<'a>,
    curve_depth: u32,
}

#[derive(Default, Clone)]
pub struct RenderedText {
    pub text: GlobalVec<InstancedText>,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone)]
pub struct InstancedText {
    pub trigs: Arc<GlyphTriangles>,
    pub offsets: GlobalVec<VertexOffset>,
}

impl<'a> VertexTextRenderer<'a> {

    pub fn new(face: Face<'a>, curve_depth: u32) -> Self {
        Self {
            trigs: GlobalVec::with_len(0x110000, None),
            offsets: GlobalVec::with_len(0x110000, Default::default()),
            face,
            curve_depth,
        }
    }

    pub fn render(
        &mut self,
        text: &str,
    ) -> Option<RenderedText>
    {
        self.offsets.fill(Default::default());
        let mut result = GlobalVec::with_capacity(16);
        let mut pen_x = 0.0;
        let face = &self.face;
        let trigs = &mut self.trigs;
        let offsets = &mut self.offsets;
        let units_per_em = face.units_per_em() as f32;
        for c in text.chars() {
            let index = c as usize;
            if trigs[index].is_none() {
                if let Some(trig) = triangulate(c, &self.face, self.curve_depth) {
                    trigs[index] = Some(Arc::new(trig));
                }
            }
            let glyph_id = face.glyph_index(c)?;
            if trigs[index].is_some() {
                let lsb = face.glyph_hor_side_bearing(glyph_id)? as f32 / units_per_em;
                let glyph_x = pen_x + lsb;
                let offsets = &mut offsets[index];
                if offsets.is_none() {
                    *offsets = Some(GlobalVec::with_capacity(4));
                }
                unsafe {
                    offsets.as_mut().unwrap_unchecked().push(VertexOffset { offset: glyph_x });
                }
            }
            let advance = face.glyph_hor_advance(glyph_id)? as f32 / units_per_em;
            pen_x += advance;
        }
        for (i, offsets) in self.offsets.iter_mut().enumerate() {
            if let Some(offsets) = offsets.take() {
                let text = InstancedText {
                    trigs: self.trigs[i].as_ref().unwrap().clone(),
                    offsets,
                };
                result.push(text);
            }
        }
        Some(RenderedText {
            text: result,
            width: pen_x,
            height: (face.ascender() - face.descender() + face.line_gap()) as f32 / units_per_em,
        })
    }
}

impl RenderedText {

    pub fn iter(&self) -> slice::Iter<'_, InstancedText> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a RenderedText {
    
    type Item = &'a InstancedText;
    type IntoIter = slice::Iter<'a, InstancedText>;

    fn into_iter(self) -> Self::IntoIter {
        self.text.iter()
    }
}
