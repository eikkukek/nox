use core::{
    hash::Hash,
    marker::PhantomData,
};

use compact_str::CompactString;

use nox::{
    mem::vec_types::GlobalVec,
    *,
};

use nox_font::{RenderedText, VertexTextRenderer, text_segment};

use crate::*;

use nox_geom::{
    shapes::*,
    *,
};

pub(crate) struct Button<I, FontHash> {
    title: CompactString,
    title_text: Option<RenderedText>,
    rect: Rect,
    rect_vertex_range: VertexRange,
    outline_vertex_range: VertexRange,
    offset: Vec2,
    flags: u32,
    outline_width: f32,
    _marker: PhantomData<(I, FontHash)>,
}

impl<I, FontHash> Button<I, FontHash> {

    const HELD: u32 = 0x1;
    const PRESSED: u32 = 0x2;
    const CURSOR_IN_BUTTON: u32 = 0x4;

    pub fn new(
        title: &str
    ) -> Self
    {
        Self {
            title: title.into(),
            title_text: Default::default(),
            rect: Default::default(),
            rect_vertex_range: Default::default(),
            outline_vertex_range: Default::default(),
            offset: Default::default(),
            flags: 0,
            outline_width: 0.0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    pub fn pressed(&self) -> bool {
        self.flags & Self::PRESSED == Self::PRESSED
    }

    #[inline(always)]
    pub fn cursor_in_button(&self) -> bool {
        self.flags & Self::CURSOR_IN_BUTTON == Self::CURSOR_IN_BUTTON
    }
}

impl<I, FontHash> Widget<I, FontHash> for Button<I, FontHash>
    where 
        I: Interface,
        FontHash: Clone + Eq + Hash,
{

    #[inline(always)]
    fn hover_text(&self) -> Option<&str> {
        None
    }

    #[inline(always)]
    fn set_offset(
        &mut self,
        offset: Vec2,
    )
    {
        self.offset = offset;
    }
    
    #[inline(always)]
    fn calc_size(
        &mut self,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
    ) -> Vec2 {
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(&self.title, &style.font_regular)], false, 0.0).unwrap_or_default()
        );
        style.calc_text_box_size(vec2(title_text.text_width, title_text.row_height))
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        _window_width: f32,
        window_pos: Vec2,
        cursor_pos: Vec2,
        cursor_in_this_window: bool,
    ) -> UpdateResult
    {
        self.flags &= !Self::PRESSED;
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(&self.title, &style.font_regular)], false, 0.0).unwrap_or_default()
        );
        let rect_size = style.calc_text_box_size(vec2(title_text.text_width, title_text.row_height));
        let rect = rect(Default::default(), rect_size, style.rounding);
        let requires_triangulation = self.rect != rect || self.outline_width != style.outline_width;
        self.rect = rect;
        self.outline_width = style.outline_width;
        let mut cursor_in_widget = false;
        self.flags &= !Self::CURSOR_IN_BUTTON;
        if self.held() {
            cursor_in_widget = true;
            if nox.was_mouse_button_released(MouseButton::Left) {
                let bounding_rect = BoundingRect::from_position_size(window_pos + self.offset, self.rect.max);
                self.flags |= Self::PRESSED * bounding_rect.is_point_inside(cursor_pos) as u32;
                self.flags &= !Self::HELD;
            }
        } else if cursor_in_this_window {
            let bounding_rect = BoundingRect::from_position_size(
                window_pos + self.offset,
                self.rect.max
            );
            cursor_in_widget = bounding_rect.is_point_inside(cursor_pos);
            if cursor_in_widget {
                self.flags |= Self::CURSOR_IN_BUTTON;
                if nox.was_mouse_button_pressed(MouseButton::Left) {
                    self.flags |= Self::HELD;
                }
            }
        }
        UpdateResult {
            min_widget_width: rect_size.x,
            requires_triangulation,
            cursor_in_widget,
        }
    }

    #[inline(always)]
    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    )
    {
        let mut outline_points = GlobalVec::new();
        self.rect.to_points(&mut |p| { points.push(p.into()); });
        nox_geom::shapes::outline_points(points, self.outline_width, false, &mut |p| { outline_points.push(p.into()); });
        self.outline_vertex_range = tri(&outline_points);
        self.rect_vertex_range = tri(&points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style<FontHash>,
        vertices: &mut [Vertex],
    )
    {
        let offset = self.offset;
        let vertex_sample = vertices[self.outline_vertex_range.start];
        if self.cursor_in_button() || self.held() {
            let target_color = if self.held() {
                style.outline_col_hl
            } else {
                style.outline_col
            };
            if vertex_sample.offset != offset || vertex_sample.color != target_color {
                for vertex in &mut vertices[self.outline_vertex_range.clone()] {
                    vertex.offset = offset;
                    vertex.color = target_color;
                }
            }
        }
        else if vertex_sample.color.a != 0.0 {
            for vertex in &mut vertices[self.outline_vertex_range.clone()] {
                vertex.color = ColorRGBA::transparent_black();
            }
        }
        let vertex_sample = vertices[self.rect_vertex_range.start];
        if vertex_sample.offset != offset || vertex_sample.color != style.widget_bg_col {
            let target_color = style.widget_bg_col;
            for vertex in &mut vertices[self.rect_vertex_range.clone()] {
                vertex.offset = offset;
                vertex.color = target_color;
            }
        }
    }

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        _base_pipeline: GraphicsPipelineId,
        text_pipeline: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
    ) -> Result<(), Error>
    {
        let title_text = unsafe {
            self.title_text.as_ref().unwrap_unchecked()
        };
        render_commands.bind_pipeline(text_pipeline)?;
        let pc_vertex = push_constants_vertex(
            window_pos + self.offset + style.item_pad_inner,
            vec2(style.font_scale, style.font_scale),
            inv_aspect_ratio
        );
        let pc_fragment = text_push_constants_fragment(style.text_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_text(title_text, render_commands, vertex_buffer, index_buffer)?;
        Ok(())
    }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    ) {
        let vertex_sample = vertices[self.rect_vertex_range.start];
        if vertex_sample.color.a != 0.0 {
            for vertex in &mut vertices[self.rect_vertex_range.clone()] {
                vertex.color = ColorRGBA::transparent_black();
            }
        }
        let vertex_sample = vertices[self.outline_vertex_range.start];
        if vertex_sample.color.a != 0.0 {
            for vertex in &mut vertices[self.outline_vertex_range.clone()] {
                vertex.color = ColorRGBA::transparent_black();
            }
        }
    }
}
