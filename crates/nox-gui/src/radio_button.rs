use core::marker::PhantomData;

use nox::{
    alloc::arena_alloc::ArenaGuard,
    mem::vec_types::{GlobalVec, Vector},
    *,
};

use nox_font::{text_segment, RenderedText};

use nox_geom::{
    shapes::*,
    *,
};

use crate::*;

pub struct RadioButton<I, Style> {
    offset: Vec2,
    diameter: f32,
    size: Vec2,
    focused_stroke_thickness: f32,
    active_stroke_thickness: f32,
    handle_vertex_range: Option<VertexRange>,
    handle_inner_vertex_range: Option<VertexRange>,
    focused_handle_stroke_vertex_range: Option<VertexRange>,
    active_handle_stroke_vertex_range: Option<VertexRange>,
    label: CompactString,
    label_text: RenderedText,
    font: CompactString,
    flags: u32,
    _marker: PhantomData<(I, Style)>,
}

impl<I, Style> RadioButton<I, Style>
    where 
        Style: WindowStyle,
{

    const HELD: u32 = 0x1;
    const HOVERED: u32 = 0x2;
    const SELECTED: u32 = 0x4;
    const CLICKED: u32 = 0x8;

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            offset: Default::default(),
            diameter: 0.0,
            size: Default::default(),
            focused_stroke_thickness: 0.0,
            active_stroke_thickness: 0.0,
            handle_vertex_range: None,
            handle_inner_vertex_range: None,
            focused_handle_stroke_vertex_range: None,
            active_handle_stroke_vertex_range: None,
            label: Default::default(),
            label_text: Default::default(),
            font: Default::default(),
            flags: 0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn set_label(
        &mut self,
        label: &str,
        text_renderer: &mut TextRenderer,
        style: &Style,
    ) {
        let font_changed = &self.font != style.font_regular();
        if font_changed {
            self.font = style.font_regular().clone();
        }
        if font_changed || self.label != label {
            self.label = CompactString::new(label);
            self.label_text = text_renderer.render(
                &[text_segment(label, &self.font)],
                false,
                0.0,
            ).unwrap_or_default();
        }
    }

    #[inline(always)]
    pub fn update_value<T: Eq>(
        &mut self,
        value: &mut T,
        radio_value: T
    ) {
        self.flags &= !Self::SELECTED;
        if self.clicked() {
            *value = radio_value;
            self.flags |= Self::SELECTED;
        } else if value == &radio_value {
            self.flags |= Self::SELECTED;
        }
    }

    #[inline(always)]
    pub fn hide(&mut self, vertices: &mut [Vertex]) {
        hide_vertices(vertices, self.handle_vertex_range);
        hide_vertices(vertices, self.handle_inner_vertex_range);
        hide_vertices(vertices, self.focused_handle_stroke_vertex_range);
        hide_vertices(vertices, self.active_handle_stroke_vertex_range);
    }

    #[inline(always)]
    fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    fn hovered(&self) -> bool {
        self.flags & Self::HOVERED == Self::HOVERED
    }

    #[inline(always)]
    fn selected(&self) -> bool {
        self.flags & Self::SELECTED == Self::SELECTED
    }

    #[inline(always)]
    fn clicked(&self) -> bool {
        self.flags & Self::CLICKED == Self::CLICKED
    }
}

impl<I, Style> Widget<I, Style> for RadioButton<I, Style>
    where 
        I: Interface,
        Style: WindowStyle,
{

    #[inline(always)]
    fn get_offset(&self) -> nox_geom::Vec2 {
        self.offset
    }

    #[inline(always)]
    fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }

    fn set_scroll_offset(&mut self, offset: Vec2) {
        self.offset += offset;
    }

    #[inline(always)]
    fn calc_size(
        &mut self,
        style: &Style,
        _text_renderer: &mut TextRenderer,
    ) -> Vec2
    {
        let text_size = style.calc_text_size(&self.label_text);
        let diameter = style.default_handle_radius() * 2.0;
        let max_y = text_size.y.max(diameter);
        self.size = vec2(diameter + style.item_pad_inner().x + text_size.x, max_y);
        self.size
    }

    fn status<'a>(
        &'a self,
        _nox: &Nox<I>,
        _style: &Style,
        _window_pos: Vec2,
        _cursor_pos: Vec2,
    ) -> WidgetStatus<'a>
    {
        if self.held() {
            WidgetStatus::Active
        } else if self.hovered() {
            WidgetStatus::Hovered(None)
        } else {
            WidgetStatus::Inactive
        }
    }

    fn update(
        &mut self,
        nox: &mut Nox<I>,
        style: &Style,
        _text_renderer: &mut TextRenderer,
        window_size: Vec2,
        window_pos: Vec2,
        content_offset: Vec2,
        cursor_pos: Vec2,
        _delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        cursor_in_other_widget: bool,
        _window_moving: bool,
        hover_blocked: bool,
        collect_text: &mut dyn FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult
    {
        let diameter = style.default_handle_radius() * 2.0;
        let requires_triangulation =
            self.diameter != diameter ||
            self.focused_stroke_thickness != style.focused_widget_stroke_thickness() ||
            self.active_stroke_thickness != style.active_widget_stroke_thickness();
        self.diameter = diameter;
        self.focused_stroke_thickness = style.focused_widget_stroke_thickness();
        self.active_stroke_thickness = style.active_widget_stroke_thickness();
        let size = self.size;
        let error_margin = style.cursor_error_margin();
        let error_margin_2 = error_margin + error_margin;
        let bounding_rect = BoundingRect::from_position_size(
            window_pos + self.offset - vec2(error_margin, error_margin),
            size + vec2(error_margin_2, error_margin_2),
        );
        let cursor_in_widget =
            cursor_in_this_window && !other_widget_active && !hover_blocked &&
            !cursor_in_other_widget && bounding_rect.is_point_inside(cursor_pos);
        self.flags &= !(Self::CLICKED | Self::HOVERED);
        if self.held() {
            if nox.was_mouse_button_released(MouseButton::Left) {
                self.flags &= !Self::HELD;
                if cursor_in_widget {
                    self.flags |= Self::CLICKED;
                }
            }
        } else if cursor_in_widget {
            self.flags |= Self::HOVERED;
            if nox.was_mouse_button_pressed(MouseButton::Left) {
                self.flags |= Self::HELD;
            }
        }
        let (min_bounds, max_bounds) = calc_bounds(
            window_pos, content_offset,
            self.offset, window_size
        );
        let bounded_instance = BoundedTextInstance {
            add_scale: vec2(1.0, 1.0),
            min_bounds,
            max_bounds,
            color:
                if self.held() {
                    style.active_text_col()
                } else if self.hovered() {
                    style.focused_text_col()
                } else {
                    style.inactive_text_col()
                },
        };
        let text_height = style.calc_text_height(&self.label_text);
        collect_text(
            &self.label_text,
            self.offset + vec2(diameter + style.item_pad_inner().x, size.y * 0.5 - text_height * 0.5),
            bounded_instance,
        );
        UpdateResult {
            requires_triangulation,
            requires_transfer_commands: false,
            cursor_in_widget
        }
    }

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        helper_points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> Option<VertexRange>,
    )
    {
        let radius = self.diameter * 0.5;
        points.clear();
        circle(vec2(radius, radius), radius)
            .to_points(16, &mut |p| { points.push(p.into()); });
        outline_points(points, self.focused_stroke_thickness, false,
            &mut |p| { helper_points.push(p.into()); });
        self.focused_handle_stroke_vertex_range = tri(&helper_points);
        helper_points.clear();
        outline_points(points, self.active_stroke_thickness, false,
            &mut |p| { helper_points.push(p.into()); });
        self.active_handle_stroke_vertex_range = tri(&helper_points);
        self.handle_vertex_range = tri(&points);
        points.clear();
        let inner_radius = radius * 0.4;
        circle(vec2(radius, radius), inner_radius)
            .to_points(16, &mut |p| { points.push(p.into()); });
        self.handle_inner_vertex_range = tri(&points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        vertices: &mut [Vertex],
    )
    {
        let offset = self.offset;
        set_vertex_params(vertices, self.handle_vertex_range, offset, style.widget_bg_col());
        if self.held() {
            set_vertex_params(vertices, self.active_handle_stroke_vertex_range, offset, style.active_widget_stroke_col());
        } else {
            hide_vertices(vertices, self.active_handle_stroke_vertex_range);
        }
        if self.hovered() {
            set_vertex_params(vertices, self.focused_handle_stroke_vertex_range, offset, style.focused_widget_stroke_col());
        } else {
            hide_vertices(vertices, self.focused_handle_stroke_vertex_range);
        }
        if self.selected() {
            let target_color =
                if self.held() {
                    style.active_text_col()
                } else if self.hovered() {
                    style.focused_text_col()
                } else {
                    style.inactive_text_col()
                };
            set_vertex_params(vertices, self.handle_inner_vertex_range, offset, target_color);
        } else {
            hide_vertices(vertices, self.handle_inner_vertex_range);
        }
    }

    fn hide(
        &mut self,
        vertices: &mut [Vertex],
        _window_semaphore: (TimelineSemaphoreId, u64),
        _global_resources: &mut GlobalResources,
        _tmp_alloc: &ArenaGuard,
    ) -> Result<(), Error> {
        self.hide(vertices); 
        Ok(())
    }
}
