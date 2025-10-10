use core::{
    hash::Hash,
    marker::PhantomData,
};

use nox::*;

use nox_geom::{
    shapes::*,
    *
};

use crate::*;

pub(crate) struct ColorPicker<I, FontHash> {
    _title: CompactString,
    offset: Vec2,
    rect: Rect,
    rect_vertex_range: VertexRange,
    _marker: PhantomData<(I, FontHash)>,
}

impl<I, FontHash> ColorPicker<I, FontHash> {

    pub fn new(title: &str) -> Self {
        Self {
            _title: CompactString::new(title),
            offset: Default::default(),
            rect: Default::default(),
            rect_vertex_range: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<I, FontHash> Widget<I, FontHash> for ColorPicker<I, FontHash>
    where 
        FontHash: Clone + Eq + Hash,
        I: Interface,
{

    #[inline(always)]
    fn hover_text(&self) -> Option<&str> {
        None
    }

    #[inline(always)]
    fn set_offset(
        &mut self,
        offset: nox_geom::Vec2,
    ) {
        self.offset = offset;
    }

    #[inline(always)]
    fn calc_size(
        &mut self,
        style: &Style<FontHash>,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
    ) -> Vec2 {
        let font_height = text_renderer.font_height(&style.font_regular).unwrap();
        //style.calc_text_size(vec2(font_height, font_height))
        vec2(font_height, font_height)
    }

    fn update(
        &mut self,
        _nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
        _window_width: f32,
        _window_pos: Vec2,
        _cursor_pos: Vec2,
        _cursor_in_this_window: bool,
    ) -> UpdateResult {
        let size = self.calc_size(style, text_renderer);
        let requires_triangulation = self.rect.max != size;
        self.rect.max = size;
        UpdateResult {
            min_widget_width: size.x,
            requires_triangulation,
            cursor_in_widget: false,
        }
    }

    fn triangulate(
        &mut self,
        points: &mut mem::vec_types::GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    )
    {
        self.rect.to_points(&mut |p| { points.push(p.into()); });
        self.rect_vertex_range = tri(points);
    }

    fn set_vertex_params(
        &mut self,
        _style: &Style<FontHash>,
        vertices: &mut [Vertex],
    ) {
        let start = self.rect_vertex_range.start;
        vertices[start].color = ColorRGBA::white();
        vertices[start + 1].color = ColorRGBA::from_rgba(1.0, 0.0, 0.0, 1.0);
        vertices[start + 2].color = ColorRGBA::black();
        vertices[start + 3].color = ColorRGBA::black();

        vertices[start].offset = self.offset;
        vertices[start + 1].offset = self.offset;
        vertices[start + 2].offset = self.offset;
        vertices[start + 3].offset = self.offset;
    }

    fn render_commands(
        &self,
        _render_commands: &mut RenderCommands,
        _style: &Style<FontHash>,
        _base_pipeline_id: GraphicsPipelineId,
        _text_pipeline_id: GraphicsPipelineId,
        _vertex_buffer: &mut RingBuf,
        _index_buffer: &mut RingBuf,
        _window_pos: Vec2,
        _inv_aspect_ratio: f32,
    ) -> Result<(), Error>
    {
        Ok(())
    }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    )
    {
        for vertex in &mut vertices[self.rect_vertex_range.clone()] {
            vertex.color = ColorRGBA::black();
        }
    }
}
