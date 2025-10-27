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

#[derive(Clone, Copy, Debug)]
pub struct TextOffset {
    pub offset: [f32; 2],
    pub char: char,
    pub row: u32,
    pub row_height: f32,
    pub x_advance: f32,
    pub offset_index: Option<u32>,
    pub first_word: bool,
}

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

    pub fn font_height(&mut self, font: &H) -> Option<f32> {
        let FaceCache { face, trigs: _, offsets: _ } = self.faces.get(font)?;
        Some((face.ascender() - face.descender() + face.line_gap()) as f32 / face.units_per_em() as f32)
    }

    pub fn render_and_collect_offsets(
        &mut self,
        text: &[impl TextSegment<H>],
        line_center: bool,
        mut max_normalized_width: f32,
        pen_x_start: f32,
        mut collect_offsets: impl FnMut(TextOffset),
    ) -> Option<RenderedText>
    {
        if max_normalized_width == 0.0 {
            max_normalized_width = f32::MAX;
        }
        let faces = &mut self.faces;
        let curve_depth = self.curve_tolerance;
        let width_div_2 = max_normalized_width / 2.0;
        let mut pen_x = pen_x_start;
        let mut shapes = GlobalVec::<(Option<f32>, &str, H, harfbuzz_rs::GlyphBuffer)>::new();
        let mut line_start = 0;
        let mut first_line = true;
        let mut height: f32 = 0.0;
        let mut text_width: f32 = 0.0;
        let mut skip_row = false;
        for segment in text {
            let FaceCache { face, trigs: _, offsets: _ } = faces.get(&segment.font())?;
            let units_per_em = face.units_per_em() as f32;
            height = height.max((face.ascender() - face.descender() + face.line_gap()) as f32 / units_per_em);
            let space = face.glyph_hor_advance(face.glyph_index(' ')?)? as f32 / units_per_em;
            for (i, word) in segment.text().split_word_bounds().enumerate() {
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
                shapes.push((None, word, segment.font().clone(), shape));
                if pen_x + word_width > max_normalized_width {
                    if word == " " {
                        continue
                    }
                    if shapes.last().unwrap().1 == " " {
                        pen_x -= space;
                    }
                    shapes[line_start].0 =
                        if first_line {
                            first_line = false;
                            if i == 0 {
                                skip_row = true;
                                Some(0.0)
                            } else {
                                Some(pen_x_start)
                            }
                        }
                        else if line_center {
                            Some(width_div_2 - pen_x / 2.0)
                        } else {
                            Some(0.0)
                        };
                    text_width = text_width.max(pen_x);
                    pen_x = 0.0;
                    line_start = shapes.len() - 1;
                }
                pen_x += word_width;
            }
        }
        text_width = text_width.max(pen_x);
        if shapes.len() == 0 {
            return None
        }
        shapes[line_start].0 =
                if first_line {
                    Some(pen_x_start)
                }
                else if line_center {
                    Some(width_div_2 - pen_x / 2.0)
                } else {
                    Some(0.0)
                };
        let mut pen_y = 0.0;
        let mut rows =
            if skip_row {
                1
            } else {
                0
            };
        for (i, (start, word, font, shape)) in shapes.iter().enumerate() {
            let FaceCache { face, trigs, offsets } = faces.get_mut(&font).unwrap();
            let units_per_em = face.units_per_em() as f32;
            if let Some(start) = start {
                pen_x = *start;
                pen_y = height * rows as f32;
                rows += 1;
            }
            let positions = shape.get_glyph_positions();
            for (j, c) in word.chars().enumerate() {
                let trigs = trigs.entry(c).or_default();
                if trigs.is_none() {
                    if let Some(trig) = triangulate(c, face, curve_depth) {
                        *trigs = Some(Arc::new(trig));
                    }
                }
                let position = positions[j];
                let glyph_x = pen_x + position.x_offset as f32 / units_per_em;
                let glyph_y = pen_y + position.y_offset as f32 / units_per_em;
                let mut offset_index = None;
                if trigs.is_some() {
                    let offsets = offsets.entry(c).or_insert(Some(Default::default())).as_mut().unwrap();
                    offset_index = Some(offsets.len() as u32);
                    offsets.push(VertexOffset { offset: [glyph_x, glyph_y] });
                }
                let x_advance = position.x_advance as f32 / units_per_em;
                collect_offsets(TextOffset {
                    offset: [glyph_x, glyph_y],
                    char: c,
                    row: rows - 1 as u32,
                    offset_index,
                    row_height: height,
                    x_advance,
                    first_word: i == 0,
                });
                pen_x += x_advance;
            }
        }
        let mut result = GlobalVec::new();
        for segment in text {
            let FaceCache { face: _, trigs, offsets } = faces.get_mut(segment.font()).unwrap();
            for (&c, off) in &mut *offsets {
                result.push((c, InstancedText {
                    trigs: trigs[&c].clone().unwrap(),
                    offsets: off.take().unwrap(),
                }));
            }
            offsets.clear();
        }
        Some(RenderedText {
            text: result,
            text_width,
            row_height: height,
            text_rows: rows,
            last_row_width: pen_x,
        })
    }

    #[inline(always)]
    pub fn render_with_start_offset(
        &mut self,
        text: &[impl TextSegment<H>],
        line_center: bool,
        max_normalized_width: f32,
        pen_x_start: f32,
    ) -> Option<RenderedText> {
        self.render_and_collect_offsets(text, line_center, max_normalized_width, pen_x_start, |_| {})
    }

    #[inline(always)]
    pub fn render(
        &mut self,
        text: &[impl TextSegment<H>],
        line_center: bool,
        max_normalized_width: f32,
    ) -> Option<RenderedText> {
        self.render_and_collect_offsets(text, line_center, max_normalized_width, 0.0, |_| {})
    }
}
