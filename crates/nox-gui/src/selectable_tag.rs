use core::marker::PhantomData;

use compact_str::CompactString;

use nox::{mem::vec_types::Vector, *};

use nox_font::{VertexTextRenderer, RenderedText, text_segment};

use nox_geom::{
    shapes::*,
    *
};

use crate::*;

pub struct SelectableTag<I, FontHash, Style> {
    offset: Vec2,
    size: Vec2,
    rounding: f32,
    width_override: f32,
    label: CompactString,
    label_text: RenderedText,
    focused_outline_width: f32,
    active_outline_width: f32,
    text_box_vertex_range: VertexRange,
    focused_outline_vertex_range: VertexRange,
    active_outline_vertex_range: VertexRange,
    font: FontHash,
    flags: u32,
    _marker: PhantomData<(I, Style)>,
}

impl<I, FontHash, Style> SelectableTag<I, FontHash, Style>
    where 
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
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
            focused_outline_width: 0.0,
            active_outline_width: 0.0,
            text_box_vertex_range: Default::default(),
            focused_outline_vertex_range: Default::default(),
            active_outline_vertex_range: Default::default(),
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
}

impl<I, FontHash, Style> Widget<I, FontHash, Style> for SelectableTag<I, FontHash, Style>
    where
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
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
        _text_renderer: &mut VertexTextRenderer<FontHash>,
    ) -> Vec2 {
        style.calc_text_box_size(&self.label_text)
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
        let mut size = style.calc_text_box_size(&self.label_text);
        if self.width_override() {
            size.x = size.x.max(self.width_override);
        }
        let requires_triangulation =
            self.size != size ||
            self.rounding != style.rounding() ||
            self.focused_outline_width != style.focused_widget_outline_width() ||
            self.active_outline_width != style.active_widget_outline_width();
        self.size = size;
        self.rounding = style.rounding();
        self.focused_outline_width = style.focused_widget_outline_width();
        self.active_outline_width = style.active_widget_outline_width();
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
        collect_text(
            &self.label_text,
            self.offset + style.item_pad_inner(),
            bounded_instance,
        );
        UpdateResult {
            requires_triangulation,
            cursor_in_widget
        }
    }

    fn triangulate(
        &mut self,
        points: &mut mem::vec_types::GlobalVec<[f32; 2]>,
        helper_points: &mut mem::vec_types::GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    )
    {
        let text_box = rect(Default::default(), self.size, self.rounding);
        text_box.to_points(&mut |p| { points.push(p.into()); });
        outline_points(points, self.focused_outline_width,
            false, &mut |p| { helper_points.push(p.into()); });
        self.focused_outline_vertex_range = tri(&helper_points);
        helper_points.clear();
        outline_points(points, self.active_outline_width,
            false, &mut |p| { helper_points.push(p.into()); });
        self.active_outline_vertex_range = tri(&helper_points);
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
            set_vertex_params(vertices, self.active_outline_vertex_range, offset, style.active_widget_outline_col());
        } else {
            hide_vertices(vertices, self.active_outline_vertex_range);
        }
        if self.hovered() {
            set_vertex_params(vertices, self.focused_outline_vertex_range, offset, style.focused_widget_outline_col());
        } else {
            hide_vertices(vertices, self.focused_outline_vertex_range);
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
        _content_area: BoundingRect,
        _inv_aspect_ratio: f32,
        _unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, Style>>, Error> { Ok(None) }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    ) {
        hide_vertices(vertices, self.text_box_vertex_range);
        hide_vertices(vertices, self.active_outline_vertex_range);
        hide_vertices(vertices, self.focused_outline_vertex_range);
    }
}
