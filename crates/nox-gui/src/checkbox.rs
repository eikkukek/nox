use core::{
    hash::Hash,
    marker::PhantomData,
};

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

pub(crate) struct Checkbox<I, FontHash> {
    title: CompactString,
    title_text: Option<RenderedText>,
    checkbox_text: Option<RenderedText>,
    rect: Rect,
    position: Vec2,
    rect_draw_info: DrawInfo,
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
        checked: bool,
    ) -> Self
    {
        Self {
            title: title.into(),
            title_text: Default::default(),
            checkbox_text: Default::default(),
            rect: Default::default(),
            position: Default::default(),
            rect_draw_info: Default::default(),
            flags: Self::CHECKED * checked as u32,
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
    fn set_position(
        &mut self,
        position: Vec2,
    )
    {
        self.position = position;
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
        let title_size = style.calc_text_size(vec2(title_text.text_width, title_text.font_height));
        let checkbox_size = style.calc_text_box_size(vec2(checkbox_text.text_width, checkbox_text.font_height));
        let max = checkbox_size.x.max(checkbox_size.y);
        let checkbox_size = vec2(max, max);
        checkbox_size + vec2(title_size.x + style.item_pad_outer.x, 0.0)
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        _window_width: f32,
        cursor_pos: Vec2,
        cursor_in_this_window: bool,
    ) -> UpdateResult
    {
        self.flags &= !Self::PRESSED;
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(self.title.as_str(), &style.font_regular)], false, 0.0).unwrap_or_default()
        );
        let checkbox_text = self.checkbox_text.get_or_insert(text_renderer
            .render(&[text_segment(&style.checkbox_symbol.to_string(), &style.font_regular)], false, 0.0).unwrap_or_default()
        );
        let rect_size = style.calc_text_box_size(vec2(checkbox_text.text_width, checkbox_text.font_height));
        let rect_max_size = rect_size.x.max(rect_size.y);
        let rect_size = vec2(rect_max_size, rect_max_size);
        let rect = rect(Default::default(), rect_size, style.rounding);
        let requires_triangulation = self.rect != rect;
        self.rect = rect;
        let mut cursor_in_widget = false;
        self.flags &= !Self::CURSOR_IN_CHECKBOX;
        let pos = self.position + vec2(style.calc_text_width(title_text.text_width) + style.item_pad_outer.x, 0.0);
        if self.held() {
            cursor_in_widget = true;
            if nox.was_mouse_button_released(MouseButton::Left) {
                let bounding_rect = BoundingRect::from_position_size(pos, self.rect.max);
                self.flags |= Self::PRESSED * bounding_rect.is_point_inside(cursor_pos) as u32;
                self.flags &= !Self::HELD;
            }
        } else if cursor_in_this_window {
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
        tri: &mut dyn FnMut(&[[f32; 2]]) -> DrawInfo,
    )
    {
        points.clear();
        self.rect.to_points(&mut |p| { points.push(p.into()); });
        self.rect_draw_info = tri(&points);
    }

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        inv_aspect_ratio: f32,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_vertex_offset: u64,
        window_index_offset: u64,
        no_offset: DrawBufferInfo,
    ) -> Result<(), Error>
    {
        let vertex_buffer_id = vertex_buffer.id();
        let index_buffer_id = index_buffer.id();
        let title_text = self.title_text.as_ref().unwrap();
        let checkbox_text = self.checkbox_text.as_ref().unwrap();
        let checkbox_pos = self.position + vec2(style.calc_text_width(title_text.text_width) + style.item_pad_outer.x, 0.0);
        if self.cursor_in_checkbox() || self.held() {
            let pc_vertex = style.calc_outline_push_constant(
                checkbox_pos,
                self.rect.max,
                inv_aspect_ratio
            );
            let pc_fragment = push_constants_fragment(
                if self.held() {
                    style.outline_col_hl
                } else {
                    style.outline_col
                }
            );
            render_commands.push_constants(|pc| unsafe {
                if pc.stage == ShaderStage::Vertex {
                    pc_vertex.as_bytes()
                } else {
                    pc_fragment.as_bytes()
                }
            })?;
            render_commands.draw_indexed(
                self.rect_draw_info,
                [
                    DrawBufferInfo::new(vertex_buffer_id, window_vertex_offset),
                    no_offset,
                ],
                DrawBufferInfo::new(index_buffer_id, window_index_offset),
            )?;
        }
        let pc_vertex = push_constants_vertex(checkbox_pos, vec2(1.0, 1.0), inv_aspect_ratio);
        let pc_fragment = push_constants_fragment(style.widget_bg_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            self.rect_draw_info,
            [
                DrawBufferInfo::new(vertex_buffer_id, window_vertex_offset),
                no_offset
            ],
            DrawBufferInfo::new(index_buffer_id, window_index_offset),
        )?;
        let pc_vertex = push_constants_vertex(
            self.position + vec2(0.0, style.item_pad_inner.y),
            vec2(style.font_scale, style.font_scale),
            inv_aspect_ratio
        );
        let pc_fragment = push_constants_fragment(style.text_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_text(title_text, render_commands, vertex_buffer, index_buffer)?;
        if self.checked() {
            let size = style.calc_text_size(vec2(checkbox_text.text_width, checkbox_text.font_height));
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
        Ok(())
    }
}
