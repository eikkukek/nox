use core::hash::Hash;

use nox::*;

use nox_geom::*;

use nox_font::{VertexTextRenderer, RenderedText, text_segment};

use crate::*;

pub trait Text {

    fn new(text: &str) -> Self;

    fn get_text<FontHash>(
        &mut self,
        text_renderer: &mut VertexTextRenderer<FontHash>,
        font: &FontHash,
    ) -> Option<&RenderedText>
        where
            FontHash: Clone + Eq + Hash;

    fn get_text_size(&self) -> Vec2;

    fn get_text_width(&self) -> f32;

    fn get_text_height(&self) -> f32;
    
    fn render(
        &self,
        render_commands: &mut RenderCommands,
        offset: Vec2,
        color: ColorSRGBA,
        font_scale: Vec2,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
    ) -> Result<(), Error>;
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
    fn get_text<FontHash>(
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

    #[inline(always)]
    fn render(
        &self,
        render_commands: &mut RenderCommands,
        offset: Vec2,
        color: ColorSRGBA,
        font_scale: Vec2,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
    ) -> Result<(), Error>
    {
        let pc_vertex = push_constants_vertex(
            offset,
            font_scale,
            inv_aspect_ratio,
            unit_scale,
        );
        let pc_fragment = text_push_constants_fragment(color);
        render_text(
            render_commands,
            self.rendered_text.as_ref().unwrap(),
            pc_vertex,
            pc_fragment, vertex_buffer, index_buffer
        )
    }
}

pub struct EmptyText;

impl Text for EmptyText {

    #[inline(always)]
    fn new(_text: &str) -> Self {
        Self
    }
    
    #[inline(always)]
    fn get_text<FontHash>(
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

    #[inline(always)]
    fn render(
        &self,
        _render_commands: &mut RenderCommands,
        _offset: Vec2,
        _color: ColorSRGBA,
        _font_scale: Vec2,
        _inv_aspect_ratio: f32,
        _unit_scale: f32,
        _vertex_buffer: &mut RingBuf,
        _index_buffer: &mut RingBuf,
    ) -> Result<(), Error>
    {
        Ok(())
    }
}

