use core::hash::Hash;

use compact_str::CompactString;

use nox::{
    mem::{value_as_bytes, vec_types::{GlobalVec, Vector}},
    *,
};

use nox_font::{RenderedText, VertexTextRenderer, text_segment};

use crate::*;

use nox_geom::{
    shapes::*,
    *,
};

pub(crate) struct Button {
    title: CompactString,
    title_text: Option<RenderedText>,
    rect: Rect,
    position: Vec2,
    rect_draw_info: DrawInfo,
    flags: u32,
}

impl Button {

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
            position: Default::default(),
            rect_draw_info: Default::default(),
            flags: 0,
        }
    }

    #[inline(always)]
    pub fn set_position(
        &mut self,
        position: Vec2,
    )
    {
        self.position = position;
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

    #[inline(always)]
    pub fn calc_size<FontHash>(
        &mut self,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<FontHash>,
    ) -> Vec2
        where 
            FontHash: Clone + Eq + Hash
    {
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(self.title.as_str(), &style.font_regular)], false, 0.0).unwrap_or_default());
        style.calc_text_size(vec2(title_text.text_width, title_text.font_height))
    }

    #[inline(always)]
    pub fn update<I, FontHash>(
        &mut self,
        nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<FontHash>,
        _window_width: f32,
        cursor_pos: Vec2,
        cursor_in_this_window: bool,
    ) -> (bool, bool, f32)
        where
            I: Interface,
            FontHash: Clone + Eq + Hash
    {
        self.flags &= !Self::PRESSED;
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(self.title.as_str(), &style.font_regular)], false, 0.0).unwrap_or_default());
        let rect_size = style.calc_text_box_size(vec2(title_text.text_width, title_text.font_height));
        let rect = rect(Default::default(), rect_size, style.rounding);
        let requires_triangulation = self.rect != rect;
        if requires_triangulation {
            self.rect = rect;
        }
        let mut cursor_in_button = false;
        self.flags &= !Self::CURSOR_IN_BUTTON;
        if self.held() {
            cursor_in_button = true;
            if nox.was_mouse_button_released(MouseButton::Left) {
                let bounding_rect = BoundingRect::from_position_size(self.position, self.rect.max);
                self.flags |= Self::PRESSED * bounding_rect.is_point_inside(cursor_pos) as u32;
                self.flags &= !Self::HELD;
            }
        } else if cursor_in_this_window {
            let bounding_rect = BoundingRect::from_position_size(
                self.position,
                self.rect.max
            );
            cursor_in_button = bounding_rect.is_point_inside(cursor_pos);
            if cursor_in_button {
                self.flags |= Self::CURSOR_IN_BUTTON;
                if nox.was_mouse_button_pressed(MouseButton::Left) {
                    self.flags |= Self::HELD;
                }
            }
        }
        (requires_triangulation, cursor_in_button, rect_size.x)
    }

    #[inline(always)]
    pub fn triangulate<F>(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        tri: &mut F,
    )
        where
            F: FnMut(&[[f32; 2]]) -> DrawInfo
    {
        points.clear();
        self.rect.to_points(&mut |p| { points.push(p.into()); });
        self.rect_draw_info = tri(&points);
    }

    #[inline(always)]
    pub fn render_commands<FontHash>(
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
        let pos = self.position;
        if self.cursor_in_button() || self.held() {
            let pc_vertex = style.calc_outline_push_constant(pos, self.rect.max, inv_aspect_ratio);
            let pc_fragment = push_constants_fragment(
                if self.held() {
                    style.outline_col_hl
                } else {
                    style.outline_col
                }
            );
            render_commands.push_constants(|pc| unsafe {
                if pc.stage == ShaderStage::Vertex {
                    value_as_bytes(&pc_vertex).unwrap()
                } else {
                    value_as_bytes(&pc_fragment).unwrap()
                }
            })?;
            render_commands.draw_indexed(
                self.rect_draw_info,
                [
                    DrawBufferInfo {
                        id: vertex_buffer_id,
                        offset: window_vertex_offset,
                    },
                    no_offset,
                ],
                DrawBufferInfo {
                    id: index_buffer_id,
                    offset: window_index_offset,
                },
            )?;
        }
        let pc_vertex = push_constants_vertex(pos, vec2(1.0, 1.0), inv_aspect_ratio);
        let pc_fragment = push_constants_fragment(style.widget_bg_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                value_as_bytes(&pc_vertex).unwrap()
            } else {
                value_as_bytes(&pc_fragment).unwrap()
            }
        })?;
        render_commands.draw_indexed(
            self.rect_draw_info,
            [
                DrawBufferInfo {
                    id: vertex_buffer_id,
                    offset: window_vertex_offset,
                },
                no_offset,
            ],
            DrawBufferInfo {
                id: index_buffer_id,
                offset: window_index_offset,
            },
        )?;
        let pc_vertex = push_constants_vertex(
            self.position + style.item_pad_inner,
            vec2(style.font_scale, style.font_scale),
            inv_aspect_ratio
        );
        let pc_fragment = push_constants_fragment(style.text_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                value_as_bytes(&pc_vertex).unwrap()
            } else {
                value_as_bytes(&pc_fragment).unwrap()
            }
        })?;
        render_text(title_text, render_commands, vertex_buffer, index_buffer)?;
        Ok(())
    }
}
