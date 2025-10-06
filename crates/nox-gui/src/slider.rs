use core::{
    fmt::Display,
    hash::Hash,
    marker::PhantomData,
};

use compact_str::CompactString;

use nox::{
    mem::vec_types::{GlobalVec, Vector},
    *,
};

use nox_font::{VertexTextRenderer, text_segment, RenderedText};

use nox_geom::{
    *,
    shapes::*,
};

use crate::*;

pub trait Sliderable: Copy + Display {

    fn slide(&mut self, min: Self, max: Self, t: f32);

    fn calc_t(&self, min: Self, max: Self) -> f32;
}

pub(crate) struct Slider<I, FontHash>
{
    title: CompactString,
    title_text: Option<RenderedText>,
    slider_rect: Rect,
    handle_rect: Rect,
    position: Vec2,
    pub t: f32,
    slider_rect_draw_info: DrawInfo,
    handle_rect_draw_info: DrawInfo,
    falgs: u32,
    _marker: PhantomData<(I, FontHash)>,
}

impl<I, FontHash> Slider<I, FontHash>
{

    const HELD: u32 = 0x1;
    const CURSOR_IN_SLIDER: u32 = 0x2;

    #[inline(always)]
    pub fn new(
        t: f32,
        title: &str,
    ) -> Self
    {
        Self {
            title: CompactString::new(title),
            title_text: Default::default(),
            slider_rect: Default::default(),
            handle_rect: Default::default(),
            position: Default::default(),
            t,
            slider_rect_draw_info: Default::default(),
            handle_rect_draw_info: Default::default(),
            falgs: 0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    fn slider_pos(
        &self,
        style: &Style<FontHash>,
        text_width: f32,
        text_box_height: f32,
    ) -> Vec2
    {
        let mut pos = self.position;
        pos.x += text_width + style.item_pad_outer.x;
        pos.y += text_box_height / 2.0 - text_box_height / 4.0;
        pos
    }

    #[inline(always)]
    fn handle_pos(
        &self,
        slider_pos: Vec2,
    ) -> Vec2
    {
        let mut pos = slider_pos;
        pos.x += (self.slider_rect.max.x - self.handle_rect.max.x) * self.t;
        pos
    }

    #[inline(always)]
    fn calc_t(
        &self,
        mut cursor_position: Vec2,
        slider_pos: Vec2,
    ) -> f32
    {
        cursor_position.x -= self.handle_rect.max.x * 0.5;
        // handle_pos solved for t
        let t = (cursor_position.x - slider_pos.x) / (self.slider_rect.max.x - self.handle_rect.max.x);
        t.clamp(0.0, 1.0)
    }

    #[inline(always)]
    pub fn held(&self) -> bool {
        self.falgs & Self::HELD == Self::HELD
    }

    #[inline(always)]
    pub fn cursor_in_slider(&self) -> bool {
        self.falgs & Self::CURSOR_IN_SLIDER == Self::CURSOR_IN_SLIDER
    }  
}

impl<I, FontHash> Widget<I, FontHash> for Slider<I, FontHash>
    where 
        I: Interface,
        FontHash: Clone + Eq + Hash,
{

    #[inline(always)]
    fn set_position(
        &mut self,
        position: Vec2,
    )
    {
        self.position = position;
    }

    fn calc_size(
        &mut self,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
    ) -> Vec2
    {
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(self.title.as_str(), &style.font_regular)], false, 0.0).unwrap_or_default());
        style.calc_text_size(vec2(title_text.text_width, title_text.font_height))
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        window_width: f32,
        cursor_pos: Vec2,
        cursor_in_this_window: bool,
    ) -> UpdateResult
        where
            I: Interface,
            FontHash: Clone + Eq + Hash
    {
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(self.title.as_str(), &style.font_regular)], false, 0.0).unwrap_or_default());
        let text_width = style.calc_text_width(title_text.text_width);
        let text_box_height = style.calc_text_box_height(title_text.font_height);
        let mut width = text_width + style.item_pad_outer.x + style.item_pad_outer.x + style.item_pad_outer.x;
        let min_window_width = width + style.slider_min_width;
        if window_width < min_window_width {
            width = style.slider_min_width;
        } else {
            width = window_width - width;
        }
        let slider_rect = rect(
            Default::default(),
            vec2(
                width,
                text_box_height / 2.0,
            ),
            style.rounding,
        );
        let handle_rect = rect(
            Default::default(),
            vec2(
                style.item_pad_inner.x * 2.0,
                slider_rect.max.y,
            ),
            style.rounding,
        );
        let requires_triangulation = slider_rect != self.slider_rect || handle_rect != self.handle_rect;
        if requires_triangulation {
            self.slider_rect = slider_rect;
            self.handle_rect = handle_rect;
        }
        let mut cursor_in_widget = false;
        self.falgs &= !Self::CURSOR_IN_SLIDER;
        if self.held() {
            cursor_in_widget = true;
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.falgs &= !Self::HELD;
            } else {
                self.t = self.calc_t(cursor_pos, self.slider_pos(style, text_width, text_box_height));
            }
        } else if cursor_in_this_window {
            let bounding_rect = BoundingRect::from_position_size(
                self.slider_pos(style, text_width, text_box_height),
                self.slider_rect.max,
            );
            cursor_in_widget = bounding_rect.is_point_inside(cursor_pos);
            if cursor_in_widget {
                self.falgs |= Self::CURSOR_IN_SLIDER;
                if nox.was_mouse_button_pressed(MouseButton::Left) {
                    self.falgs |= Self::HELD;
                    self.t = self.calc_t(cursor_pos, self.slider_pos(style, text_width, text_box_height));
                }
            }
        }
        UpdateResult {
            requires_triangulation,
            cursor_in_widget,
            min_widget_width: min_window_width - style.item_pad_outer.x - style.item_pad_outer.x,
        }
    }

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> DrawInfo,
    )
    {
        points.clear();
        self.slider_rect.to_points(&mut |p| { points.push(p.into()); });
        self.slider_rect_draw_info = tri(&points);
        points.clear();
        self.handle_rect.to_points(&mut |p| { points.push(p.into()); });
        self.handle_rect_draw_info = tri(&points);
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
        let text_width = style.calc_text_width(title_text.text_width);
        let text_box_height = style.calc_text_box_height(title_text.font_height);
        let slider_pos = self.slider_pos(style, text_width, text_box_height);
        if self.cursor_in_slider() || self.held() {
            let pc_vertex = style.calc_outline_push_constant(slider_pos, self.slider_rect.max, inv_aspect_ratio);
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
                self.slider_rect_draw_info,
                [
                    DrawBufferInfo::new(vertex_buffer_id, window_vertex_offset),
                    no_offset,
                ],
                DrawBufferInfo::new(index_buffer_id, window_index_offset),
            )?;
        }
        let mut pc_vertex = push_constants_vertex(slider_pos, vec2(1.0, 1.0), inv_aspect_ratio);
        let mut pc_fragment = push_constants_fragment(style.widget_bg_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            self.slider_rect_draw_info,
            [
                DrawBufferInfo::new(vertex_buffer_id, window_vertex_offset),
                no_offset,
            ],
            DrawBufferInfo {
                id: index_buffer_id,
                offset: window_index_offset,
            },
        )?;
        pc_fragment.color = style.handle_col;
        pc_vertex.vert_off = self.handle_pos(slider_pos);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            self.handle_rect_draw_info,
            [
                DrawBufferInfo::new(vertex_buffer_id, window_vertex_offset),
                no_offset,
            ],
            DrawBufferInfo::new(index_buffer_id, window_index_offset),
        )?;
        let pc_vertex = push_constants_vertex(
            vec2(self.position.x, self.position.y + (text_box_height - title_text.font_height * style.font_scale) / 2.0),
            vec2(style.font_scale, style.font_scale),
            inv_aspect_ratio,
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
        Ok(())
    }
}

macro_rules! impl_sliderable {
    ($($t:ty),+ $(,)?) => {
        $(
            impl Sliderable for $t {

                fn slide(&mut self, min: Self, max: Self, t: f32) {
                    *self = ((1.0 - t) * min as f32 + t * max as f32) as $t
                }

                fn calc_t(&self, min: Self, max: Self) -> f32 {
                    if *self >= max { return 1.0 }
                    if *self <= min { return 0.0 }
                    let d0 = max - min;
                    let d1 = self - min;
                    d1 as f32 / d0 as f32
                }
            }
        )+
    };
}

impl_sliderable!(
    i8, i16, i32, i64, i128,
    u8, u16, u32, u64, u128,
    f32, f64,
);
