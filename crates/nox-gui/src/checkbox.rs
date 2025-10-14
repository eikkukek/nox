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

pub(crate) struct Checkbox<I, FontHash> {
    title: CompactString,
    title_text: Option<RenderedText>,
    checkbox_text: Option<RenderedText>,
    rect: Rect,
    offset: Vec2,
    rect_vertex_range: VertexRange,
    outline_vertex_range: VertexRange,
    outline_width: f32,
    flags: u32,
    _marker: PhantomData<(I, FontHash)>
}

impl<I, FontHash> Checkbox<I, FontHash> {

    const HELD: u32 = 0x1;
    const PRESSED: u32 = 0x2;
    const CURSOR_IN_CHECKBOX: u32 = 0x4;
    const CHECKED: u32 = 0x8;

    #[inline(always)]
    pub fn new(
        title: &str,
    ) -> Self
    {
        Self {
            title: title.into(),
            title_text: Default::default(),
            checkbox_text: Default::default(),
            rect: Default::default(),
            offset: Default::default(),
            rect_vertex_range: Default::default(),
            outline_vertex_range: Default::default(),
            outline_width: 0.0,
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

impl<I, FontHash> Widget<I, FontHash> for Checkbox<I, FontHash>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash
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
    ) -> Vec2
    {
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(&self.title, &style.font_regular)], false, 0.0).unwrap_or_default()
        );
        let checkbox_text = self.checkbox_text.get_or_insert(text_renderer
            .render(&[text_segment(&style.checkbox_symbol.to_string(), &style.font_regular)], false, 0.0).unwrap_or_default()
        );
        let title_size = style.calc_text_size(vec2(title_text.text_width, title_text.row_height));
        let checkbox_size = style.calc_text_box_size(vec2(checkbox_text.text_width, checkbox_text.row_height));
        let max = checkbox_size.x.max(checkbox_size.y);
        let checkbox_size = vec2(max, max);
        checkbox_size + vec2(title_size.x + style.item_pad_outer.x, 0.0)
    }

    fn is_active(&self, _style: &Style<FontHash>, _window_pos: Vec2, _cursor_pos: Vec2) -> bool {
        self.held()
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        _window_width: f32,
        window_pos: Vec2,
        cursor_pos: Vec2,
        _delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
    ) -> UpdateResult
    {
        self.flags &= !Self::PRESSED;
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(self.title.as_str(), &style.font_regular)], false, 0.0).unwrap_or_default()
        );
        let checkbox_text = self.checkbox_text.get_or_insert(text_renderer
            .render(&[text_segment(&style.checkbox_symbol.to_string(), &style.font_regular)], false, 0.0).unwrap_or_default()
        );
        let rect_size = style.calc_text_box_size(vec2(checkbox_text.text_width, checkbox_text.row_height));
        let rect_max_size = rect_size.x.max(rect_size.y);
        let rect_size = vec2(rect_max_size, rect_max_size);
        let rect = rect(Default::default(), rect_size, style.rounding);
        let requires_triangulation = self.rect != rect || self.outline_width != style.outline_width;
        self.rect = rect;
        self.outline_width = style.outline_width;
        let mut cursor_in_widget = false;
        self.flags &= !Self::CURSOR_IN_CHECKBOX;
        let pos = window_pos + self.offset + vec2(style.calc_text_width(title_text.text_width) + style.item_pad_outer.x, 0.0);
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
        UpdateResult {
            min_widget_width: self.calc_size(style, text_renderer).x,
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
        let title_text = self.title_text.as_ref().unwrap();
        let offset = self.offset + vec2(style.calc_text_width(title_text.text_width) + style.item_pad_outer.x, 0.0);
        let vertex_sample = vertices[self.outline_vertex_range.start()];
        if self.cursor_in_checkbox() || self.held() {
            let target_color = if self.held() {
                style.widget_outline_col_hl
            } else {
                style.widget_outline_col
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
        if vertex_sample.offset != offset || vertex_sample.color != style.widget_bg_col {
            let target_color = style.widget_bg_col;
            for vertex in &mut vertices[self.rect_vertex_range.range()] {
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
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn OnTopContents<I, FontHash>>, Error>
    {
        let title_text = self.title_text.as_ref().unwrap();
        render_commands.bind_pipeline(text_pipeline)?;
        let pc_vertex = push_constants_vertex(
            window_pos + self.offset + vec2(0.0, style.item_pad_inner.y),
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
        if self.checked() {
            let checkbox_text = self.checkbox_text.as_ref().unwrap();
            let checkbox_pos = window_pos + self.offset + vec2(style.calc_text_width(title_text.text_width) + style.item_pad_outer.x, 0.0);
            let size = style.calc_text_size(vec2(checkbox_text.text_width, checkbox_text.row_height));
            let pc_vertex = push_constants_vertex(
                checkbox_pos + self.rect.max * 0.5 - size * 0.5,
                vec2(style.font_scale, style.font_scale),
                inv_aspect_ratio,
            );
            render_commands.push_constants(|pc| unsafe {
                if pc.stage == ShaderStage::Vertex {
                    pc_vertex.as_bytes()
                } else {
                    pc_fragment.as_bytes()
                }
            })?;
            render_text(checkbox_text, render_commands, vertex_buffer, index_buffer)?;
        }
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
