use core::hash::Hash;

use nox_geom::*;

use nox_font::{VertexTextRenderer, RenderedText, text_segment};

use crate::*;

pub trait Text {

    fn new(text: &str) -> Self;

    fn update<FontHash>(
        &mut self,
        text_renderer: &mut VertexTextRenderer<FontHash>,
        font: &FontHash,
    ) -> Option<&RenderedText>
        where
            FontHash: Clone + Eq + Hash;

    fn get_text_size(&self) -> Vec2;

    fn get_text_width(&self) -> f32;

    fn get_text_height(&self) -> f32;
}

pub struct DefaultText {
    text: CompactString,
    rendered_text: Option<RenderedText>,
}

impl Text for DefaultText {

    #[inline(always)]
    fn new(text: &str) -> Self {
        Self {
            text: text.into(),
            rendered_text: None,
        }
    }

    #[inline(always)]
    fn update<FontHash>(
        &mut self,
        text_renderer: &mut VertexTextRenderer<FontHash>,
        font: &FontHash,
    ) -> Option<&RenderedText>
        where
            FontHash: Clone + Eq + Hash,
    {
        Some(self.rendered_text.get_or_insert(
            text_renderer.render(
                &[text_segment(&self.text, font)],
                false, 0.0
            ).unwrap_or_default()
        ))
    }

    #[inline(always)]
    fn get_text_size(&self) -> Vec2 {
        self.rendered_text
            .as_ref()
            .map(|v| vec2(v.text_width, v.row_height * v.text_rows as f32))
            .unwrap_or_default()
    }

    #[inline(always)]
    fn get_text_width(&self) -> f32 {
        self.rendered_text
            .as_ref()
            .map(|v| v.text_width)
            .unwrap_or_default()
    }

    #[inline(always)]
    fn get_text_height(&self) -> f32 {
        self.rendered_text
            .as_ref()
            .map(|v| v.row_height * v.text_rows as f32)
            .unwrap_or_default()
    }
}

pub struct EmptyText;

impl Text for EmptyText {

    #[inline(always)]
    fn new(_text: &str) -> Self {
        Self
    }
    
    #[inline(always)]
    fn update<FontHash>(
        &mut self,
        _text_renderer: &mut VertexTextRenderer<FontHash>,
        _font: &FontHash,
    ) -> Option<&RenderedText>
        where 
            FontHash: Clone + Eq + Hash
    {
        None
    }

    #[inline(always)]
    fn get_text_size(&self) -> Vec2 {
        Default::default()
    }

    #[inline(always)]
    fn get_text_width(&self) -> f32 {
        0.0
    }

    #[inline(always)]
    fn get_text_height(&self) -> f32 {
        0.0
    }
}

