use std::sync::Arc;

use core::slice;

use nox::mem::{
    vec_types::GlobalVec,
    slice,
};

use super::*;

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
        let buffer = harfbuzz_rs::UnicodeBuffer
            ::new()
            .add_str(text);
        let features = harfbuzz_rs::Feature::new(
            harfbuzz_rs::Tag::new('g', 'b', 'o', 's'), 1, 0..usize::MAX);
        let output = harfbuzz_rs::shape(&self.face.hb_font, buffer, &[features]);
        let positions = output.get_glyph_positions();
        let face = &self.face;
        let trigs = &mut self.trigs;
        let offsets = &mut self.offsets;
        let curve_depth = self.curve_depth;
        let units_per_em = face.units_per_em() as f32;
        let mut pen_x = 0.0;
        for (i, c) in text.chars().enumerate() {
            let glyph_index = c as usize;
            let trigs = &mut trigs[glyph_index];
            if trigs.is_none() {
                if let Some(trig) = triangulate(c, face, curve_depth) {
                    *trigs = Some(Arc::new(trig));
                }
            }
            let position = positions[i];
            let glyph_x = pen_x + position.x_offset as f32 / units_per_em;
            let offsets = &mut offsets[glyph_index];
            if let Some(offsets) = offsets.as_mut() {
                offsets.push(VertexOffset { offset: glyph_x });
            }
            else if trigs.is_some() {
                *offsets = Some(slice![VertexOffset { offset: glyph_x }].into());
            }
            pen_x += position.x_advance as f32 / units_per_em;
        }
        let mut result = GlobalVec::with_capacity(8);
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
