use core::marker::PhantomData;

use compact_str::CompactString;

use nox::{alloc::arena_alloc::ArenaGuard, mem::vec_types::Vector, *};

use nox_font::{RenderedText, text_segment};

use nox_geom::{
    shapes::*,
    *
};

use crate::*;

pub struct SelectableTag<Style> {
    offset: Vec2,
    size: Vec2,
    rounding: f32,
    width_override: f32,
    label: CompactString,
    label_text: RenderedText,
    focused_stroke_thickness: f32,
    active_stroke_thickness: f32,
    text_box_vertex_range: Option<VertexRange>,
    focused_stroke_vertex_range: Option<VertexRange>,
    active_stroke_vertex_range: Option<VertexRange>,
    font: CompactString,
    flags: u32,
    _marker: PhantomData<Style>,
}

impl<Style> SelectableTag<Style>
    where 
        Style: WindowStyle,
{

    const HELD: u32 = 0x1;
    const HOVERED: u32 = 0x2;
    const SELECTED: u32 = 0x4;
    const CLICKED: u32 = 0x8;
    const WIDTH_OVERRIDE: u32 = 0x10;

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            offset: Default::default(),
            size: Default::default(),
            rounding: 0.0,
            width_override: 0.0,
            label: Default::default(),
            label_text: Default::default(),
            focused_stroke_thickness: 0.0,
            active_stroke_thickness: 0.0,
            text_box_vertex_range: None,
            focused_stroke_vertex_range: None,
            active_stroke_vertex_range: None,
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
        target: T
    ) -> bool {
        self.flags &= !Self::SELECTED;
        if self.clicked() {
            *value = target;
            self.flags |= Self::SELECTED;
        } else if value == &target {
            self.flags |= Self::SELECTED;
        }
        self.flags &= !Self::WIDTH_OVERRIDE;
        self.selected()
    }

    #[inline(always)]
    pub fn override_width(&mut self, width: f32) {
        self.width_override = width;
        self.flags |= Self::WIDTH_OVERRIDE;
    }

    #[inline(always)]
    pub fn label_text(&self) -> &RenderedText {
        &self.label_text
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
    pub fn clicked(&self) -> bool {
        self.flags & Self::CLICKED == Self::CLICKED
    }

    #[inline(always)]
    fn width_override(&self) -> bool {
        self.flags & Self::WIDTH_OVERRIDE == Self::WIDTH_OVERRIDE
    }

    #[inline(always)]
    pub fn hide(&mut self, vertices: &mut [Vertex]) {
        hide_vertices(vertices, self.text_box_vertex_range);
        hide_vertices(vertices, self.active_stroke_vertex_range);
        hide_vertices(vertices, self.focused_stroke_vertex_range);
    }
}

impl<Style> Widget<Style> for SelectableTag<Style>
    where
        Style: WindowStyle,
{

    #[inline(always)]
    fn get_offset(&self) -> Vec2 {
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
        _text_renderer: &mut TextRenderer,
    ) -> Vec2 {
        style.calc_text_box_size(&self.label_text)
    }

    fn status<'a>(
        &'a self,
        _ctx: &WindowCtx,
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
        ctx: &mut WindowCtx,
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
        let mut size = style.calc_text_box_size(&self.label_text);
        if self.width_override() {
            size.x = size.x.max(self.width_override);
        }
        let requires_triangulation =
            self.size != size ||
            self.rounding != style.rounding() ||
            self.focused_stroke_thickness != style.focused_widget_stroke_thickness() ||
            self.active_stroke_thickness != style.active_widget_stroke_thickness();
        self.size = size;
        self.rounding = style.rounding();
        self.focused_stroke_thickness = style.focused_widget_stroke_thickness();
        self.active_stroke_thickness = style.active_widget_stroke_thickness();
        let error_margin = style.cursor_error_margin();
        let error_margin_2 = error_margin + error_margin;
        let bounding_rect = BoundingRect::from_position_size(
            window_pos + self.offset - vec2(error_margin, 0.0),
            size + vec2(error_margin_2, 0.0),
        );
        let cursor_in_widget =
            cursor_in_this_window && !hover_blocked &&
            !other_widget_active && !cursor_in_other_widget &&
            bounding_rect.is_point_inside(cursor_pos);
        self.flags &= !(Self::CLICKED | Self::HOVERED);
        let mouse_left_state = ctx.mouse_button_state(MouseButton::Left);
        if self.held() {
            if mouse_left_state.released() {
                self.flags &= !Self::HELD;
                if cursor_in_widget {
                    self.flags |= Self::CLICKED;
                }
            }
        } else if cursor_in_widget {
            self.flags |= Self::HOVERED;
            if mouse_left_state.pressed() {
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
        collect_text(
            &self.label_text,
            self.offset + style.item_pad_inner(),
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
        points: &mut mem::vec_types::GlobalVec<[f32; 2]>,
        helper_points: &mut mem::vec_types::GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> Option<VertexRange>,
    )
    {
        let text_box = rect(Default::default(), self.size, self.rounding);
        text_box.to_points(&mut |p| { points.push(p.into()); });
        outline_points(points, self.focused_stroke_thickness,
            false, &mut |p| { helper_points.push(p.into()); });
        self.focused_stroke_vertex_range = tri(&helper_points);
        helper_points.clear();
        outline_points(points, self.active_stroke_thickness,
            false, &mut |p| { helper_points.push(p.into()); });
        self.active_stroke_vertex_range = tri(&helper_points);
        self.text_box_vertex_range = tri(&points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        vertices: &mut [Vertex],
    ) {
        let offset = self.offset;
        if self.selected() {
            set_vertex_params(vertices, self.text_box_vertex_range, offset, style.selection_col());
        } else if self.held() || self.hovered() {
            set_vertex_params(vertices, self.text_box_vertex_range, offset, style.widget_bg_col());
        } else {
            hide_vertices(vertices, self.text_box_vertex_range);
        }
        if self.held() {
            set_vertex_params(vertices, self.active_stroke_vertex_range, offset, style.active_widget_stroke_col());
        } else {
            hide_vertices(vertices, self.active_stroke_vertex_range);
        }
        if self.hovered() {
            set_vertex_params(vertices, self.focused_stroke_vertex_range, offset, style.focused_widget_stroke_col());
        } else {
            hide_vertices(vertices, self.focused_stroke_vertex_range);
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
