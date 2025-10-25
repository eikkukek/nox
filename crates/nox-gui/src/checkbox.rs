use core::{
    hash::Hash,
    marker::PhantomData,
};

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

pub struct Checkbox<I, FontHash, Style> {
    checkbox_text: Option<RenderedText>,
    rect: Rect,
    offset: Vec2,
    rect_vertex_range: VertexRange,
    outline_vertex_range: VertexRange,
    focused_outline_width: f32,
    flags: u32,
    _marker: PhantomData<(I, FontHash, Style)>
}

impl<I, FontHash, Style> Checkbox<I, FontHash, Style>
    where
        Style: WindowStyle<FontHash>,
{

    const HELD: u32 = 0x1;
    const PRESSED: u32 = 0x2;
    const CURSOR_IN_CHECKBOX: u32 = 0x4;
    const CHECKED: u32 = 0x8;

    #[inline(always)]
    pub fn new() -> Self
    {
        Self {
            checkbox_text: Default::default(),
            rect: Default::default(),
            offset: Default::default(),
            rect_vertex_range: Default::default(),
            outline_vertex_range: Default::default(),
            focused_outline_width: 0.0,
            flags: 0,
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
    pub fn cursor_in_checkbox(&self) -> bool {
        self.flags & Self::CURSOR_IN_CHECKBOX == Self::CURSOR_IN_CHECKBOX
    }

    #[inline(always)]
    pub fn checked(&self) -> bool {
        self.flags & Self::CHECKED == Self::CHECKED
    }

    #[inline(always)]
    pub fn set_checked(&mut self, value: bool) {
        self.flags &= !Self::CHECKED;
        self.flags |= Self::CHECKED * value as u32;
    }
}

impl<I, FontHash, Style> Widget<I, FontHash, Style> for Checkbox<I, FontHash, Style>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
{

    #[inline(always)]
    fn hover_text(&self) -> Option<&str> {
        None
    }

    #[inline(always)]
    fn get_offset(&self) -> Vec2
    {
        self.offset
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
        style: &Style,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
    ) -> Vec2
    {
        let checkbox_text = self.checkbox_text.get_or_insert(text_renderer
            .render(
                &[text_segment(&style.checkbox_symbol().to_string(), style.font_regular())], false, 0.0
            )
            .unwrap_or_default()
        );
        let checkbox_size = style.calc_text_box_size(checkbox_text);
        let max = checkbox_size.x.max(checkbox_size.y);
        vec2(max, max)
    }

    fn status(
        &self,
        _nox: &Nox<I>,
        _style: &Style,
        _window_pos: Vec2, _cursor_pos: Vec2
    ) -> WidgetStatus
    {
        if self.held() {
            WidgetStatus::Active
        } else if self.cursor_in_checkbox() {
            WidgetStatus::Hovered
        } else {
            WidgetStatus::Inactive
        }
    }

    fn update(
        &mut self,
        nox: &mut Nox<I>,
        style: &Style,
        _text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        window_size: Vec2,
        window_pos: Vec2,
        cursor_pos: Vec2,
        _delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        _cursor_in_other_widget: bool,
        _window_moving: bool,
        collect_text: &mut dyn FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult
    {
        self.flags &= !Self::PRESSED;
        let offset = self.offset;
        let checkbox_text = self.checkbox_text.as_ref().unwrap();
        let rect_size = style.calc_text_box_size(checkbox_text);
        let rect_max_size = rect_size.x.max(rect_size.y);
        let rect_size = vec2(rect_max_size, rect_max_size);
        let rect = rect(Default::default(), rect_size, style.rounding());
        let requires_triangulation =
            self.rect != rect ||
            self.focused_outline_width != style.focused_outline_width();
        self.rect = rect;
        self.focused_outline_width = style.focused_outline_width();
        let mut cursor_in_widget = false;
        self.flags &= !Self::CURSOR_IN_CHECKBOX;
        let pos = window_pos + offset;
        if self.held() {
            cursor_in_widget = true;
            if nox.was_mouse_button_released(MouseButton::Left) {
                let bounding_rect = BoundingRect::from_position_size(pos, self.rect.max);
                self.flags |= Self::PRESSED * bounding_rect.is_point_inside(cursor_pos) as u32;
                self.flags &= !Self::HELD;
            }
        } else if cursor_in_this_window && !other_widget_active {
            let bounding_rect = BoundingRect::from_position_size(
                pos,
                self.rect.max,
            );
            cursor_in_widget = bounding_rect.is_point_inside(cursor_pos);
            if cursor_in_widget {
                self.flags |= Self::CURSOR_IN_CHECKBOX;
                if nox.was_mouse_button_pressed(MouseButton::Left) {
                    self.flags |= Self::HELD;
                }
            }
        }
        if self.checked() {
            let (min_bounds, max_bounds) = calc_bounds(window_pos, self.offset, window_size);
            let bounded_instance = BoundedTextInstance {
                add_scale: vec2(1.0, 1.0),
                min_bounds,
                max_bounds,
                color:
                    if self.held() || self.cursor_in_checkbox() {
                        style.focused_text_col()
                    } else {
                        style.text_col()
                    },
            };
            let checkbox_text = self.checkbox_text.as_ref().unwrap();
            let size = style.calc_text_size(checkbox_text);
            collect_text(checkbox_text,
                offset + rect.max * 0.5 - size * 0.5,
                bounded_instance,
            );
        }
        UpdateResult {
            requires_triangulation,
            cursor_in_widget
        }
    }

    #[inline(always)]
    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    )
    {
        self.rect.to_points(&mut |p| { points.push(p.into()); });
        let mut outline_points = GlobalVec::new();
        nox_geom::shapes::outline_points(points,
            self.focused_outline_width, false, &mut |p| { outline_points.push(p.into()); }
        );
        self.outline_vertex_range = tri(&outline_points);
        self.rect_vertex_range = tri(&points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        vertices: &mut [Vertex],
    )
    {
        let offset = self.offset;
        let vertex_sample = vertices[self.outline_vertex_range.start()];
        if self.cursor_in_checkbox() || self.held() {
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
        _render_commands: &mut RenderCommands,
        _style: &Style,
        _base_pipeline: GraphicsPipelineId,
        _text_pipeline: GraphicsPipelineId,
        _vertex_buffer: &mut RingBuf,
        _index_buffer: &mut RingBuf,
        _window_pos: Vec2,
        _inv_aspect_ratio: f32,
        _unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, Style>>, Error>
    {
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
