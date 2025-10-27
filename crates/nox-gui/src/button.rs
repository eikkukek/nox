use core::marker::PhantomData;

use compact_str::CompactString;

use nox::{
    mem::vec_types::{GlobalVec, Vector},
    *,
};

use nox_font::{RenderedText, VertexTextRenderer, text_segment};

use crate::*;

use nox_geom::{
    shapes::*,
    *,
};

pub struct Button<I, FontHash, Style> {
    label: CompactString,
    label_text: RenderedText,
    font: FontHash,
    rect: Rect,
    rect_vertex_range: VertexRange,
    focused_outline_vertex_range: VertexRange,
    active_outline_vertex_range: VertexRange,
    focused_outline_width: f32,
    active_outline_width: f32,
    offset: Vec2,
    flags: u32,
    _marker: PhantomData<(I, FontHash, Style)>,
}

impl<I, FontHash, Style> Button<I, FontHash, Style>
    where 
        FontHash: UiFontHash,
{

    const HELD: u32 = 0x1;
    const PRESSED: u32 = 0x2;
    const CURSOR_IN_BUTTON: u32 = 0x4;

    pub fn new() -> Self
    {
        Self {
            label: Default::default(),
            label_text: Default::default(),
            font: Default::default(),
            rect: Default::default(),
            rect_vertex_range: Default::default(),
            focused_outline_vertex_range: Default::default(),
            active_outline_vertex_range: Default::default(),
            focused_outline_width: 0.0,
            active_outline_width: 0.0,
            offset: Default::default(),
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
    ) where 
        Style: WindowStyle<FontHash>, 
    {
        let font_changed = &self.font != style.font_regular();
        if font_changed {
            self.font = style.font_regular().clone();
        }
        if font_changed || self.label != label {
            self.label = CompactString::new(label);
            self.label_text = text_renderer
                .render(&[text_segment(&self.label, &self.font)],
                    false, 0.0
            ).unwrap_or_default();
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
    pub fn hovered(&self) -> bool {
        self.flags & Self::CURSOR_IN_BUTTON == Self::CURSOR_IN_BUTTON
    }
}

impl<I, FontHash, Style> Widget<I, FontHash, Style> for
        Button<I, FontHash, Style>
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
        _text_renderer: &mut VertexTextRenderer<FontHash>,
    ) -> Vec2 {
        style.calc_text_box_size(&self.label_text)
    }

    fn status<'a>(
        &'a self,
        _nox: &Nox<I>,
        _style: &Style,
        _window_pos: Vec2,
        _cursor_pos: Vec2
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
        let rect_size = style.calc_text_box_size(&self.label_text);
        let rect = rect(Default::default(), rect_size, style.rounding());
        let requires_triangulation =
            self.rect != rect ||
            self.focused_outline_width != style.focused_widget_outline_width() ||
            self.active_outline_width != style.active_widget_outline_width();
        self.rect = rect;
        self.focused_outline_width = style.focused_widget_outline_width();
        self.active_outline_width = style.active_widget_outline_width();
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
        let (min_bounds, max_bounds) = calc_bounds(window_pos, self.offset, window_size);
        collect_text(&self.label_text, self.offset + style.item_pad_inner(), BoundedTextInstance {
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
        });
        UpdateResult {
            requires_triangulation,
            cursor_in_widget,
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
            self.active_outline_width, false, &mut |p| { helper_points.push(p.into()); });
        self.active_outline_vertex_range = tri(&helper_points);
        self.rect_vertex_range = tri(&points);
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
        } else {
            hide_vertices(vertices, self.active_outline_vertex_range);
        }
        if self.hovered() {
            let target_color = style.focused_widget_outline_col();
            set_vertex_params(vertices, self.focused_outline_vertex_range, offset, target_color);
        } else {
            hide_vertices(vertices, self.focused_outline_vertex_range);
        }
        let target_color = style.widget_bg_col();
        set_vertex_params(vertices, self.rect_vertex_range, offset, target_color);
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
        hide_vertices(vertices, self.rect_vertex_range);
        hide_vertices(vertices, self.focused_outline_vertex_range);
        hide_vertices(vertices, self.active_outline_vertex_range);
    }
}
