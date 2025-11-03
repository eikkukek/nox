use core::marker::PhantomData;

use nox::{
    alloc::arena_alloc::ArenaGuard,
    mem::vec_types::{GlobalVec, Vector},
    *,
};

use nox_font::{RenderedText, text_segment};

use crate::*;

use nox_geom::{
    shapes::*,
    *,
};

pub struct Checkbox<I, Style> {
    offset: Vec2,
    rect: Rect,
    size: Vec2,
    label: CompactString,
    label_text: RenderedText,
    focused_outline_width: f32,
    active_outline_width: f32,
    checkmark_points: GlobalVec<[f32; 2]>,
    rect_vertex_range: VertexRange,
    checkmark_vertex_range: VertexRange,
    focused_outline_vertex_range: VertexRange,
    active_outline_vertex_range: VertexRange,
    font: CompactString,
    flags: u32,
    _marker: PhantomData<(I, Style)>
}

impl<I, Style> Checkbox<I, Style>
    where
        Style: WindowStyle,
{

    const HELD: u32 = 0x1;
    const CLICKED: u32 = 0x2;
    const CURSOR_IN_CHECKBOX: u32 = 0x4;
    const CHECKED: u32 = 0x8;

    #[inline(always)]
    pub fn new() -> Self
    {
        Self {
            rect: Default::default(),
            size: Default::default(),
            offset: Default::default(),
            label: Default::default(),
            label_text: Default::default(),
            focused_outline_width: 0.0,
            active_outline_width: 0.0,
            checkmark_points: Default::default(),
            checkmark_vertex_range: Default::default(),
            rect_vertex_range: Default::default(),
            focused_outline_vertex_range: Default::default(),
            active_outline_vertex_range: Default::default(),
            font: Default::default(),
            flags: 0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    fn clicked(&self) -> bool {
        self.flags & Self::CLICKED == Self::CLICKED
    }

    #[inline(always)]
    fn hovered(&self) -> bool {
        self.flags & Self::CURSOR_IN_CHECKBOX == Self::CURSOR_IN_CHECKBOX
    }

    #[inline(always)]
    fn checked(&self) -> bool {
        self.flags & Self::CHECKED == Self::CHECKED
    }

    #[inline(always)]
    pub fn update_value(&mut self, value: &mut bool) {
        if self.clicked() {
            *value = !*value;
        }
        self.flags &= !Self::CHECKED;
        or_flag!(self.flags, Self::CHECKED, *value);
    }

    #[inline(always)]
    pub fn set_label(
        &mut self,
        label: &str,
        text_renderer: &mut TextRenderer,
        style: &Style,
    ) where 
        Style: WindowStyle,
    {
        let font_changed = &self.font != style.font_regular();
        if font_changed {
            self.font = style.font_regular().clone();
        }
        if font_changed || self.label != label {
            self.label = CompactString::new(label);
            self.label_text = text_renderer.render(
                &[text_segment(label, &self.font)],
                false,
                0.0
            ).unwrap_or_default();
        }
    }

    #[inline(always)]
    pub fn hide(&mut self, vertices: &mut [Vertex]) {
        hide_vertices(vertices, self.rect_vertex_range);
        hide_vertices(vertices, self.focused_outline_vertex_range);
        hide_vertices(vertices, self.active_outline_vertex_range);
        hide_vertices(vertices, self.checkmark_vertex_range);
    }
}

impl<I, Style> Widget<I, Style> for Checkbox<I, Style>
    where
        I: Interface,
        Style: WindowStyle,
{

    #[inline(always)]
    fn get_offset(&self) -> Vec2
    {
        self.offset
    }

    #[inline(always)]
    fn set_offset(&mut self, offset: Vec2)
    {
        self.offset = offset;
    }

    #[inline(always)]
    fn set_scroll_offset(&mut self, offset: Vec2) {
        self.offset += offset;
    }

    #[inline(always)]
    fn calc_size(
        &mut self,
        style: &Style,
        text_renderer: &mut TextRenderer,
    ) -> Vec2
    {
        let checkmark_box_max = style.calc_font_height(text_renderer);
        let checkbox_size = vec2(checkmark_box_max, checkmark_box_max);
        let label_width = style.calc_text_width(&self.label_text);
        self.size = checkbox_size + vec2(style.item_pad_inner().x + label_width, 0.0);
        self.size
    }

    fn status<'a>(
        &'a self,
        _nox: &Nox<I>,
        _style: &Style,
        _window_pos: Vec2, _cursor_pos: Vec2
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
        text_renderer: &mut TextRenderer,
        window_size: Vec2,
        window_pos: Vec2,
        content_offset: Vec2,
        cursor_pos: Vec2,
        _delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        _cursor_in_other_widget: bool,
        _window_moving: bool,
        hover_blocked: bool,
        collect_text: &mut dyn FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult
    {
        let offset = self.offset;
        let rect_max = style.calc_font_height(text_renderer);
        let rect_size = vec2(rect_max, rect_max);
        let rect = rect(Default::default(), rect_size, style.rounding());
        let requires_triangulation =
            self.rect != rect ||
            self.focused_outline_width != style.focused_widget_outline_width() ||
            self.active_outline_width != style.active_widget_outline_width();
        self.rect = rect;
        self.focused_outline_width = style.focused_widget_outline_width();
        self.active_outline_width = style.active_widget_outline_width();
        let mut cursor_in_widget = false;
        self.flags &= !(Self::CLICKED | Self::CURSOR_IN_CHECKBOX);
        let pos = window_pos + offset;
        let bounding_rect = BoundingRect::from_position_size(
            pos, self.size
        );
        if self.held() {
            cursor_in_widget = true;
            if nox.was_mouse_button_released(MouseButton::Left) {
                or_flag!(self.flags, Self::CLICKED, bounding_rect.is_point_inside(cursor_pos));
                self.flags &= !Self::HELD;
            }
        } else if cursor_in_this_window && !other_widget_active && !hover_blocked {
            cursor_in_widget = bounding_rect.is_point_inside(cursor_pos);
            if cursor_in_widget {
                self.flags |= Self::CURSOR_IN_CHECKBOX;
                if nox.was_mouse_button_pressed(MouseButton::Left) {
                    self.flags |= Self::HELD;
                }
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
        collect_text(
            &self.label_text,
            offset + vec2(rect.max.x + style.item_pad_inner().x, 0.0),
            bounded_instance,
        );
        if requires_triangulation {
            self.checkmark_points.clear();
            style.get_checkmark_points(text_renderer, &mut self.checkmark_points);
        }
        UpdateResult {
            requires_triangulation,
            requires_transfer_commands: false,
            cursor_in_widget
        }
    }

    #[inline(always)]
    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        helper_points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    )
    {
        self.rect.to_points(&mut |p| { points.push(p.into()); });
        outline_points(points,
            self.focused_outline_width, false, &mut |p| { helper_points.push(p.into()); }
        );
        self.focused_outline_vertex_range = tri(&helper_points);
        helper_points.clear();
        outline_points(points,
            self.active_outline_width, false, &mut |p| { helper_points.push(p.into()); }
        );
        self.active_outline_vertex_range = tri(&helper_points);
        self.rect_vertex_range = tri(&points);
        points.clear();
        points.append(&self.checkmark_points);
        self.checkmark_vertex_range = tri(&points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        vertices: &mut [Vertex],
    )
    {
        let offset = self.offset;
        if self.held() {
            let target_color = style.active_widget_outline_col();
            set_vertex_params(vertices, self.active_outline_vertex_range, offset, target_color);
        }
        else {
            hide_vertices(vertices, self.active_outline_vertex_range);
        }
        if self.hovered() {
            let target_color = style.focused_widget_outline_col();
            set_vertex_params(vertices, self.focused_outline_vertex_range, offset, target_color);
        } else {
            hide_vertices(vertices, self.focused_outline_vertex_range);
        }
        if self.checked() {
            let offset = offset + self.rect.max * 0.5;
            let target_color =
                if self.held() {
                    style.active_text_col()
                } else if self.hovered() {
                    style.focused_text_col()
                } else {
                    style.inactive_text_col()
                };
            set_vertex_params(vertices, self.checkmark_vertex_range, offset, target_color);
        } else {
            hide_vertices(vertices, self.checkmark_vertex_range);
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
