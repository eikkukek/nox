use core::marker::PhantomData;

use nox::{
    mem::vec_types::{GlobalVec, Vector},
    *,
};

use nox_font::{VertexTextRenderer, text_segment, RenderedText};

use nox_geom::{
    shapes::*,
    *,
};

use crate::*;

pub struct RadioButton<I, FontHash, Style> {
    offset: Vec2,
    diameter: f32,
    size: Vec2,
    focused_outline_width: f32,
    active_outline_width: f32,
    handle_vertex_range: VertexRange,
    handle_inner_vertex_range: VertexRange,
    focused_handle_outline_vertex_range: VertexRange,
    active_handle_outline_vertex_range: VertexRange,
    label: CompactString,
    label_text: RenderedText,
    font: FontHash,
    flags: u32,
    _marker: PhantomData<(I, Style)>,
}

impl<I, FontHash, Style> RadioButton<I, FontHash, Style>
    where 
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
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
            focused_outline_width: 0.0,
            active_outline_width: 0.0,
            handle_vertex_range: Default::default(),
            handle_inner_vertex_range: Default::default(),
            focused_handle_outline_vertex_range: Default::default(),
            active_handle_outline_vertex_range: Default::default(),
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
        text_renderer: &mut VertexTextRenderer<FontHash>,
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

impl<I, FontHash, Style> Widget<I, FontHash, Style> for RadioButton<I, FontHash, Style>
    where 
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
{

    fn get_offset(&self) -> nox_geom::Vec2 {
        self.offset
    }

    fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }

    fn calc_size(
        &mut self,
        style: &Style,
        _text_renderer: &mut VertexTextRenderer<FontHash>,
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
        _text_renderer: &mut VertexTextRenderer<'_, FontHash>,
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
            self.focused_outline_width != style.focused_widget_outline_width() ||
            self.active_outline_width != style.active_widget_outline_width();
        self.diameter = diameter;
        self.focused_outline_width = style.focused_widget_outline_width();
        self.active_outline_width = style.active_widget_outline_width();
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
            cursor_in_widget
        }
    }

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        helper_points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    )
    {
        let radius = self.diameter * 0.5;
        points.clear();
        circle(vec2(radius, radius), radius)
            .to_points(16, &mut |p| { points.push(p.into()); });
        outline_points(points, self.focused_outline_width, false,
            &mut |p| { helper_points.push(p.into()); });
        self.focused_handle_outline_vertex_range = tri(&helper_points);
        helper_points.clear();
        outline_points(points, self.active_outline_width, false,
            &mut |p| { helper_points.push(p.into()); });
        self.active_handle_outline_vertex_range = tri(&helper_points);
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
            set_vertex_params(vertices, self.active_handle_outline_vertex_range, offset, style.active_widget_outline_col());
        } else {
            hide_vertices(vertices, self.active_handle_outline_vertex_range);
        }
        if self.hovered() {
            set_vertex_params(vertices, self.focused_handle_outline_vertex_range, offset, style.focused_widget_outline_col());
        } else {
            hide_vertices(vertices, self.focused_handle_outline_vertex_range);
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

    fn render_commands(
        &self,
        _render_commands: &mut RenderCommands,
        _style: &Style,
        _base_pipeline_id: GraphicsPipelineId,
        _text_pipeline_id: GraphicsPipelineId,
        _vertex_buffer: &mut RingBuf,
        _index_buffer: &mut RingBuf,
        _window_pos: Vec2,
        _inv_aspect_ratio: f32,
        _unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, Style>>, Error> { Ok(None) }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    ) {
        hide_vertices(vertices, self.handle_vertex_range);
        hide_vertices(vertices, self.handle_inner_vertex_range);
        hide_vertices(vertices, self.focused_handle_outline_vertex_range);
        hide_vertices(vertices, self.active_handle_outline_vertex_range);
    }
}
