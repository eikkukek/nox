use std::sync::Arc;

use core::slice;

use unicode_segmentation::UnicodeSegmentation;

use nox::mem::{
    slice,
    vec_types::{GlobalVec, Vector},
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
        line_center: bool,
        normalized_width: f32,
    ) -> Option<RenderedText>
    {
        self.offsets.fill(Default::default());
        let face = &self.face;
        let trigs = &mut self.trigs;
        let offsets = &mut self.offsets;
        let curve_depth = self.curve_depth;
        let units_per_em = face.units_per_em() as f32;
        let height = (face.ascender() - face.descender() + face.line_gap()) as f32 / units_per_em;
        let space = face.glyph_hor_advance(face.glyph_index(' ')?)? as f32 / units_per_em;
        let mut pen_x = 0.0;
        let mut shapes = GlobalVec::<(Option<f32>, &str, harfbuzz_rs::GlyphBuffer)>
            ::with_capacity(text.split_word_bounds().count());
        let mut line_start = 0;
        let width_div_2 = normalized_width / 2.0;
        for word in text.split_word_bounds() {
            let buffer = harfbuzz_rs::UnicodeBuffer
                ::new()
                .add_str(word);
            let shape = harfbuzz_rs::shape(&face.hb_font, buffer, &[]);
            let positions = shape.get_glyph_positions();
            let mut word_width = 0.0;
            for position in positions {
                word_width += position.x_advance as f32 / units_per_em;
            }
            if word_width > normalized_width {
                continue
            }
            if pen_x + word_width > normalized_width {
                if word == " " {
                    continue
                }
                if shapes.back().unwrap().1 == " " {
                    pen_x -= space;
                }
                shapes[line_start].0 =
                    if line_center {
                        Some(width_div_2 - pen_x / 2.0)
                    } else {
                        Some(0.0)
                    };
                pen_x = 0.0;
                line_start = shapes.len();
            }
            shapes.push((None, word, shape));
            pen_x += word_width;
        }
        if shapes.len() == 0 {
            return None
        }
        shapes[line_start].0 =
            if line_center {
                Some(width_div_2 - pen_x / 2.0)
            } else {
                Some(0.0)
            };
        let mut pen_y = -height;
        for (start, word, shape) in &shapes {
            if let &Some(start) = start {
                pen_x = start;
                pen_y += height;
            }
            let positions = shape.get_glyph_positions();
            for (i, c) in word.chars().enumerate() {
                let glyph_index = c as usize;
                let trigs = &mut trigs[glyph_index];
                if trigs.is_none() {
                    if let Some(trig) = triangulate(c, face, curve_depth) {
                        *trigs = Some(Arc::new(trig));
                    }
                }
                let position = positions[i];
                let glyph_x = pen_x + position.x_offset as f32 / units_per_em;
                let glyph_y = pen_y + position.y_offset as f32 / units_per_em;
                let offsets = &mut offsets[glyph_index];
                if let Some(offsets) = offsets.as_mut() {
                    offsets.push(VertexOffset {
                        offset: [glyph_x, glyph_y],
                    });
                }
                else if trigs.is_some() {
                    *offsets = Some(slice![VertexOffset {
                        offset: [glyph_x, glyph_y],
                    }].into());
                }
                pen_x += position.x_advance as f32 / units_per_em;
            }
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
            width: normalized_width,
            height: height,
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
