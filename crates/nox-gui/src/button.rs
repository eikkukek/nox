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

pub(crate) struct Button<I, FontHash, Style, HoverStyle> {
    title: CompactString,
    title_text: Option<RenderedText>,
    rect: Rect,
    rect_vertex_range: VertexRange,
    outline_vertex_range: VertexRange,
    offset: Vec2,
    flags: u32,
    focused_outline_width: f32,
    _marker: PhantomData<(I, FontHash, Style, HoverStyle)>,
}

impl<I, FontHash, Style, HoverStyle> Button<I, FontHash, Style, HoverStyle> {

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
            focused_outline_width: 0.0,
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

impl<I, FontHash, Style, HoverStyle> Widget<I, FontHash, Style, HoverStyle> for
        Button<I, FontHash, Style, HoverStyle>
    where 
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
        HoverStyle: WindowStyle<FontHash>,
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
    fn calc_height(
        &mut self,
        style: &Style,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
    ) -> f32 {
        let title_text = self.title_text.get_or_insert(text_renderer
                .render(&[text_segment(&self.title, &style.font_regular())], false, 0.0
            ).unwrap_or_default()
        );
        style.calc_text_box_height(title_text)
    }

    fn is_active(
        &self,
        _nox: &Nox<I>,
        _style: &Style,
        _hover_style: &HoverStyle,
        _window_pos: Vec2,
        _cursor_pos: Vec2
    ) -> bool
    {
        self.held()
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style,
        _hover_style: &HoverStyle,
        _text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        _window_width: f32,
        window_pos: Vec2,
        cursor_pos: Vec2,
        _delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        _window_moving: bool,
    ) -> UpdateResult
    {
        self.flags &= !Self::PRESSED;
        let title_text = self.title_text.as_ref().unwrap();
        let rect_size = style.calc_text_box_size(title_text);
        let rect = rect(Default::default(), rect_size, style.rounding());
        let requires_triangulation =
            self.rect != rect ||
            self.focused_outline_width != style.focused_outline_width();
        self.rect = rect;
        self.focused_outline_width = style.focused_outline_width();
        let mut cursor_in_widget = false;
        self.flags &= !Self::CURSOR_IN_BUTTON;
        if self.held() {
            cursor_in_widget = true;
            if nox.was_mouse_button_released(MouseButton::Left) {
                let bounding_rect = BoundingRect::from_position_size(window_pos + self.offset, self.rect.max);
                self.flags |= Self::PRESSED * bounding_rect.is_point_inside(cursor_pos) as u32;
                self.flags &= !Self::HELD;
            }
        } else if cursor_in_this_window && !other_widget_active {
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
        nox_geom::shapes::outline_points(points,
            self.focused_outline_width, false, &mut |p| { outline_points.push(p.into()); }
        );
        self.outline_vertex_range = tri(&outline_points);
        self.rect_vertex_range = tri(&points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        _hover_style: &HoverStyle,
        vertices: &mut [Vertex],
    )
    {
        let offset = self.offset;
        let vertex_sample = vertices[self.outline_vertex_range.start()];
        if self.cursor_in_button() || self.held() {
            let target_color = if self.held() {
                style.active_widget_outline_col()
            } else {
                style.focused_widget_outline_col()
            };
            if vertex_sample.offset != offset || vertex_sample.color != target_color {
                for vertex in &mut vertices[self.outline_vertex_range.range()] {
                    vertex.offset = offset;
                    vertex.color = target_color;
                }
            }
        }
        else if vertex_sample.color.alpha != 0.0 {
            for vertex in &mut vertices[self.outline_vertex_range.range()] {
                vertex.color = ColorSRGBA::black(0.0);
            }
        }
        let vertex_sample = vertices[self.rect_vertex_range.start()];
        if vertex_sample.offset != offset || vertex_sample.color != style.widget_bg_col() {
            let target_color = style.widget_bg_col();
            for vertex in &mut vertices[self.rect_vertex_range.range()] {
                vertex.offset = offset;
                vertex.color = target_color;
            }
        }
    }

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style,
        _base_pipeline: GraphicsPipelineId,
        text_pipeline: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, HoverStyle>>, Error>
    {
        let title_text = unsafe {
            self.title_text.as_ref().unwrap_unchecked()
        };
        render_commands.bind_pipeline(text_pipeline)?;
        let pc_vertex = push_constants_vertex(
            window_pos + self.offset + style.item_pad_inner(),
            vec2(style.font_scale(), style.font_scale()),
            inv_aspect_ratio,
            unit_scale,
        );
        let pc_fragment = text_push_constants_fragment(style.text_col());
        render_text(render_commands, title_text, pc_vertex, pc_fragment, vertex_buffer, index_buffer)?;
        Ok(None)
    }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    ) {
        let vertex_sample = vertices[self.rect_vertex_range.start()];
        if vertex_sample.color.alpha != 0.0 {
            for vertex in &mut vertices[self.rect_vertex_range.range()] {
                vertex.color = ColorSRGBA::black(0.0);
            }
        }
        let vertex_sample = vertices[self.outline_vertex_range.start()];
        if vertex_sample.color.alpha != 0.0 {
            for vertex in &mut vertices[self.outline_vertex_range.range()] {
                vertex.color = ColorSRGBA::black(0.0);
            }
        }
    }
}
