mod structs;

use std::sync::Arc;

use core::{
    slice,
    hash::Hash,
};

use rustc_hash::FxHashMap;

use unicode_segmentation::UnicodeSegmentation;

use nox::mem::vec_types::{GlobalVec, Vector};

use super::*;

pub use structs::*;

pub struct VertexTextRenderer<'a, H: Clone + PartialEq + Eq + Hash> {
    faces: FxHashMap<H, FaceCache<'a>>,
    curve_tolerance: f32,
}

impl<'a, H: Clone + PartialEq + Eq + Hash> VertexTextRenderer<'a, H> {

    pub fn new(fonts: impl IntoIterator<Item = (H, Face<'a>)>, curve_tolerance: f32) -> Self {
        let mut faces = FxHashMap::default();
        for face in fonts {
            faces.insert(face.0, FaceCache { face: face.1, trigs: Default::default(), offsets: Default::default() });
        }
        Self {
            faces,
            curve_tolerance,
        }
    }

    pub fn render(
        &mut self,
        text: &[TextSegment<H>],
        line_center: bool,
        mut max_normalized_width: f32,
    ) -> Option<RenderedText>
    {
        if max_normalized_width == 0.0 {
            max_normalized_width = f32::MAX;
        }
        let faces = &mut self.faces;
        let curve_depth = self.curve_tolerance;
        let width_div_2 = max_normalized_width / 2.0;
        let mut pen_x = 0.0;
        let mut shapes = GlobalVec::<(Option<f32>, &str, H, harfbuzz_rs::GlyphBuffer)>::new();
        let mut line_start = 0;
        let mut height: f32 = 0.0;
        let mut text_width: f32 = 0.0;
        for segment in text {
            let FaceCache { face, trigs: _, offsets: _ } = faces.get(&segment.font.clone())?;
            let units_per_em = face.units_per_em() as f32;
            height = height.max((face.ascender() - face.descender() + face.line_gap()) as f32 / units_per_em);
            let space = face.glyph_hor_advance(face.glyph_index(' ')?)? as f32 / units_per_em;
            for word in segment.text.split_word_bounds() {
                let buffer = harfbuzz_rs::UnicodeBuffer
                    ::new()
                    .add_str(word);
                let shape = harfbuzz_rs::shape(&face.hb_font, buffer, &[]);
                let positions = shape.get_glyph_positions();
                let mut word_width = 0.0;
                for position in positions {
                    word_width += position.x_advance as f32 / units_per_em;
                }
                if word_width > max_normalized_width {
                    continue
                }
                if pen_x + word_width > max_normalized_width {
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
                    text_width = text_width.max(pen_x);
                    pen_x = 0.0;
                    line_start = shapes.len();
                }
                shapes.push((None, word, segment.font.clone(), shape));
                pen_x += word_width;
            }
        }
        text_width = text_width.max(pen_x);
        if shapes.len() == 0 {
            return None
        }
        shapes[line_start].0 =
            if line_center {
                Some(width_div_2 - pen_x / 2.0)
            } else {
                Some(0.0)
            };
        let mut pen_y = 0.0;
        let mut rows = 0;
        let mut result = GlobalVec::new();
        for (start, word, font, shape) in &shapes {
            let FaceCache { face, trigs, offsets } = faces.get_mut(&font.clone()).unwrap();
            let units_per_em = face.units_per_em() as f32;
            if let Some(start) = start {
                pen_x = *start;
                pen_y = height * rows as f32;
                rows += 1;
            }
            let positions = shape.get_glyph_positions();
            for (i, c) in word.chars().enumerate() {
                let trigs = trigs.entry(c).or_default();
                if trigs.is_none() {
                    if let Some(trig) = triangulate(c, face, curve_depth) {
                        *trigs = Some(Arc::new(trig));
                    }
                }
                let position = positions[i];
                let glyph_x = pen_x + position.x_offset as f32 / units_per_em;
                let glyph_y = pen_y + position.y_offset as f32 / units_per_em;
                if trigs.is_some() {
                    let offsets = offsets.entry(c).or_insert(Some(Default::default())).as_mut().unwrap();
                    offsets.push(VertexOffset { offset: [glyph_x, glyph_y] });
                }
                pen_x += position.x_advance as f32 / units_per_em;
            }
        }
        for TextSegment { text: _, font } in text {
            let FaceCache { face: _, trigs, offsets } = faces.get_mut(font).unwrap();
            for (c, off) in &mut *offsets {
                result.push(InstancedText {
                    trigs: trigs[c].clone().unwrap(),
                    offsets: off.take().unwrap(),
                });
            }
            offsets.clear();
        }
        Some(RenderedText {
            text: result,
            text_width,
            font_height: height,
            text_rows: rows,
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
