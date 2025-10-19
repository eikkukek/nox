use core::{
    fmt::Write,
    hash::Hash,
    marker::PhantomData,
    str::FromStr,
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

pub trait Sliderable: Copy + FromStr {

    fn slide_and_quantize_t(&mut self, min: Self, max: Self, t: f32) -> f32;

    fn drag(&mut self, min: Self, max: Self, amount: f32);

    fn calc_t(&self, min: Self, max: Self) -> f32;

    fn display<FontHash>(
        &self,
        style: &impl WindowStyle<FontHash>,
        to: &mut impl Write, 
    ) -> core::fmt::Result;
}

pub(crate) struct Slider<I, FontHash, Style, HoverStyle>
{
    title: CompactString,
    title_text: Option<RenderedText>,
    slider_rect: Rect,
    slider_rect_vertex_range: VertexRange,
    handle_rect: Rect,
    handle_rect_vertex_range: VertexRange,
    outline_rect_vertex_range: VertexRange,
    offset: Vec2,
    pub t: f32,
    pub quantized_t: f32,
    pub hover_text: CompactString,
    falgs: u32,
    focused_outline_width: f32,
    _marker: PhantomData<(I, FontHash, Style, HoverStyle)>,
}

impl<I, FontHash, Style, HoverStyle> Slider<I, FontHash, Style, HoverStyle>
    where
        Style: WindowStyle<FontHash>,
{

    const HELD: u32 = 0x1;
    const CURSOR_IN_SLIDER: u32 = 0x2;

    #[inline(always)]
    pub fn new(
        title: &str,
    ) -> Self
    {
        Self {
            title: CompactString::new(title),
            title_text: Default::default(),
            slider_rect: Default::default(),
            slider_rect_vertex_range: Default::default(),
            handle_rect: Default::default(),
            handle_rect_vertex_range: Default::default(),
            outline_rect_vertex_range: Default::default(),
            offset: Default::default(),
            t: 1.0,
            quantized_t: 1.0,
            hover_text: Default::default(),
            falgs: 0,
            focused_outline_width: 0.0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    fn slider_off(
        &self,
        style: &Style,
        text_width: f32,
    ) -> Vec2
    {
        let mut pos = self.offset;
        pos.x += text_width + style.item_pad_outer().x;
        //pos.y += text_box_height / 2.0 - text_box_height / 4.0;
        pos
    }

    #[inline(always)]
    fn handle_off(
        &self,
        slider_off: Vec2,
    ) -> Vec2
    {
        let mut pos = slider_off;
        pos.x += (self.slider_rect.max.x - self.handle_rect.max.x) * self.quantized_t;
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

impl<I, FontHash, Style, HoverStyle> Widget<I, FontHash, Style, HoverStyle> for
        Slider<I, FontHash, Style, HoverStyle>
    where 
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
        HoverStyle: WindowStyle<FontHash>,
{

    #[inline(always)]
    fn hover_text(&self) -> Option<&str> {
        Some(self.hover_text.as_str())
    }

    #[inline(always)]
    fn set_offset(
        &mut self,
        offset: Vec2,
    )
    {
        self.offset = offset;
    }

    fn calc_height(
        &mut self,
        style: &Style,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
    ) -> f32
    {
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(
                &[text_segment(self.title.as_str(), &style.font_regular())], false, 0.0
        ).unwrap_or_default());
        style.calc_text_height(title_text)
    }

    fn is_active(
        &self,
        _nox: &Nox<I>,
        _style: &Style,
        _hover_style: &HoverStyle,
        _window_pos: Vec2,
        _cursor_pos: Vec2
    ) -> bool
    {
        self.held()
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style,
        _hover_style: &HoverStyle,
        _text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        window_width: f32,
        window_pos: Vec2,
        cursor_pos: Vec2,
        _delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        _window_moving: bool,
    ) -> UpdateResult
        where
            I: Interface,
            FontHash: Clone + Eq + Hash
    {
        let title_text = self.title_text.as_ref().unwrap();
        let text_width = style.calc_text_width(title_text);
        let text_box_height = style.calc_text_box_height(title_text);
        let mut width = text_width +
            style.item_pad_outer().x + style.item_pad_outer().x + style.item_pad_outer().x;
        let min_window_width = width + style.min_slider_width();
        if window_width < min_window_width {
            width = style.min_slider_width();
        } else {
            width = window_width - width;
        }
        let slider_rect = rect(
            Default::default(),
            vec2(
                width,
                text_box_height / 2.0,
            ),
            style.rounding(),
        );
        let handle_rect = rect(
            Default::default(),
            vec2(
                style.item_pad_inner().x * 2.0,
                slider_rect.max.y,
            ),
            style.rounding(),
        );
        let requires_triangulation =
            slider_rect != self.slider_rect ||
            handle_rect != self.handle_rect ||
            self.focused_outline_width != style.focused_outline_width();
        self.focused_outline_width = style.focused_outline_width();
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
                self.t = self.calc_t(cursor_pos, window_pos + self.slider_off(style, text_width));
            }
        } else if cursor_in_this_window && !other_widget_active {
            let bounding_rect = BoundingRect::from_position_size(
                window_pos + self.slider_off(style, text_width),
                self.slider_rect.max,
            );
            cursor_in_widget = bounding_rect.is_point_inside(cursor_pos);
            if cursor_in_widget {
                self.falgs |= Self::CURSOR_IN_SLIDER;
                if nox.was_mouse_button_pressed(MouseButton::Left) {
                    self.falgs |= Self::HELD;
                    self.t = self.calc_t(cursor_pos, window_pos + self.slider_off(style, text_width));
                }
            }
        }
        UpdateResult {
            requires_triangulation,
            cursor_in_widget,
            min_widget_width: min_window_width - style.item_pad_outer().x - style.item_pad_outer().x,
        }
    }

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    )
    {
        self.slider_rect.to_points(&mut |p| { points.push(p.into()); });
        let mut outline_points = GlobalVec::<[f32; 2]>::new();
        nox_geom::shapes::outline_points(points,
            self.focused_outline_width, false, &mut |p| { outline_points.push(p.into()); }
        );
        self.outline_rect_vertex_range = tri(&outline_points);
        self.slider_rect_vertex_range = tri(&points);
        points.clear();
        self.handle_rect.to_points(&mut |p| { points.push(p.into()); });
        self.handle_rect_vertex_range = tri(&points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        _hover_style: &HoverStyle,
        vertices: &mut [Vertex],
    )
    {
        let title_text = self.title_text.as_ref().unwrap();
        let text_width = style.calc_text_width(title_text);
        let slider_off = self.slider_off(style, text_width);
        let vertex_sample = vertices[self.outline_rect_vertex_range.start()];
        if self.cursor_in_slider() || self.held() {
            let offset = slider_off;
            let target_color = if self.held() {
                style.active_widget_outline_col()
            } else {
                style.focused_widget_outline_col()
            };
            if vertex_sample.offset != offset || vertex_sample.color != target_color {
                for vertex in &mut vertices[self.outline_rect_vertex_range.range()] {
                    vertex.offset = offset;
                    vertex.color = target_color;
                }
            }
        }
        else if vertex_sample.color.alpha != 0.0 {
            for vertex in &mut vertices[self.outline_rect_vertex_range.range()] {
                vertex.color = ColorSRGBA::black(0.0);
            }
        }
        let vertex_sample = vertices[self.slider_rect_vertex_range.start()];
        if vertex_sample.offset != slider_off || vertex_sample.color != style.widget_bg_col() {
            let target_color = style.widget_bg_col();
            for vertex in &mut vertices[self.slider_rect_vertex_range.range()] {
                vertex.offset = slider_off;
                vertex.color = target_color;
            }
        }
        let vertex_sample = vertices[self.handle_rect_vertex_range.start()];
        let handle_off = self.handle_off(slider_off);
        if vertex_sample.offset != handle_off || vertex_sample.color != style.handle_col() {
            let target_color = style.handle_col();
            for vertex in &mut vertices[self.handle_rect_vertex_range.range()] {
                vertex.offset = handle_off;
                vertex.color = target_color;
            }
        }
    }

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style,
        _base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, HoverStyle>>, Error>
    {
        let title_text = unsafe {
            self.title_text.as_ref().unwrap_unchecked()
        };
        render_commands.bind_pipeline(text_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            window_pos + vec2(
                self.offset.x,
                self.offset.y +
                    (self.slider_rect.max.y - style.calc_text_height(title_text)) / 2.0
            ),
            vec2(style.font_scale(), style.font_scale()),
            inv_aspect_ratio,
            unit_scale,
        );
        let pc_fragment = text_push_constants_fragment(style.text_col());
        render_text(render_commands, title_text, pc_vertex, pc_fragment, vertex_buffer, index_buffer)?;
        Ok(None)
    }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    ) {
        let vertex_sample = vertices[self.slider_rect_vertex_range.start()];
        if vertex_sample.color.alpha != 0.0 {
            for vertex in &mut vertices[self.slider_rect_vertex_range.range()] {
                vertex.color = ColorSRGBA::black(0.0);
            }
        }
        let vertex_sample = vertices[self.handle_rect_vertex_range.start()];
        if vertex_sample.color.alpha != 0.0 {
            for vertex in &mut vertices[self.handle_rect_vertex_range.range()] {
                vertex.color = ColorSRGBA::black(0.0);
            }
        }
        let vertex_sample = vertices[self.outline_rect_vertex_range.start()];
        if vertex_sample.color.alpha != 0.0 {
            for vertex in &mut vertices[self.outline_rect_vertex_range.range()] {
                vertex.color = ColorSRGBA::black(0.0);
            }
        }
    }
}

impl Sliderable for f32 {

    #[inline(always)]
    fn slide_and_quantize_t(&mut self, min: Self, max: Self, t: f32) -> f32 {
        *self = (1.0 - t) * min + t * max;
        t
    }

    #[inline(always)]
    fn drag(&mut self, min: Self, max: Self, amount: f32) {
        *self += amount;
        *self = self.clamp(min, max);
    }

    #[inline(always)]
    fn calc_t(&self, min: Self, max: Self) -> f32 {
        if *self >= max { return 1.0 }
        if *self <= min { return 0.0 }
        let d0 = max - min;
        let d1 = self - min;
        d1 / d0
    }

    #[inline(always)]
    fn display<FontHash>(
        &self,
        style: &impl WindowStyle<FontHash>,
        to: &mut impl Write,
    ) -> core::fmt::Result
    {
        style.f32_format(*self, to)
    }
}


impl Sliderable for f64 {

    #[inline(always)]
    fn slide_and_quantize_t(&mut self, min: Self, max: Self, t: f32) -> f32 {
        *self = ((1.0 - t as f64) * min + t as f64 * max) as f64;
        t
    }

    #[inline(always)]
    fn drag(&mut self, min: Self, max: Self, amount: f32) {
        *self += amount as f64;
        *self = self.clamp(min, max);
    }

    #[inline(always)]
    fn calc_t(&self, min: Self, max: Self) -> f32 {
        if *self >= max { return 1.0 }
        if *self <= min { return 0.0 }
        let d0 = max - min;
        let d1 = self - min;
        (d1 / d0) as f32
    }

    #[inline(always)]
    fn display<FontHash>(
        &self,
        style: &impl WindowStyle<FontHash>,
        to: &mut impl Write,
    ) -> core::fmt::Result
    {
        style.f64_format(*self, to)
    }
}

macro_rules! impl_sliderable_int {
    ($($t:ty),+ $(,)?) => {
        $(
            impl Sliderable for $t {

                #[inline(always)]
                fn slide_and_quantize_t(&mut self, min: Self, max: Self, t: f32) -> f32 {
                    let mut as_float = 0.0;
                    as_float.slide_and_quantize_t(min as f32, max as f32, t);
                    let fract = as_float.fract();
                    *self = 
                        if fract >= 0.5 {
                            as_float.ceil() as Self
                        } else {
                            as_float.floor() as Self
                        };
                    self.calc_t(min, max)
                }

                #[inline(always)]
                fn drag(&mut self, min: Self, max: Self, mut amount: f32) {
                    if amount.abs() < f32::EPSILON {
                        return
                    }
                    if amount.is_sign_negative() {
                        amount = amount.min(-1.0);
                        let amount = amount as Self;
                        if (*self > 0 && amount >= Self::MIN) || Self::MIN - *self <= amount {
                            *self += amount;
                        }
                    } else {
                        amount = amount.max(1.0);
                        let amount = amount as Self;
                        if (*self < 0 && amount <= Self::MAX) || Self::MAX - *self >= amount {
                            *self += amount;
                        }
                    }
                    *self = (*self).clamp(min, max);
                }

                #[inline(always)]
                fn calc_t(&self, min: Self, max: Self) -> f32 {
                    if *self >= max { return 1.0 }
                    if *self <= min { return 0.0 }
                    let d0 = max - min;
                    let d1 = self - min;
                    d1 as f32 / d0 as f32
                }

                #[inline(always)]
                fn display<FontHash>(
                    &self,
                    _style: &impl WindowStyle<FontHash>,
                    to: &mut impl Write,
                ) -> core::fmt::Result
                {
                    write!(to, "{}", *self)
                }
            }
        )+
    };
}

macro_rules! impl_sliderable_uint {
    ($($t:ty),+ $(,)?) => {
        $(
            impl Sliderable for $t {

                #[inline(always)]
                fn slide_and_quantize_t(&mut self, min: Self, max: Self, t: f32) -> f32 {
                    let mut as_float = 0.0;
                    as_float.slide_and_quantize_t(min as f32, max as f32, t);
                    let fract = as_float.fract();
                    *self = 
                        if fract >= 0.5 {
                            as_float.ceil() as $t
                        } else {
                            as_float.floor() as $t
                        };
                    self.calc_t(min, max)
                }

                #[inline(always)]
                fn drag(&mut self, min: Self, max: Self, mut amount: f32) {
                    if amount.abs() < f32::EPSILON {
                        return
                    }
                    if amount.is_sign_negative() {
                        amount = amount.min(-1.0);
                        let amount = amount.abs() as Self;
                        if amount <= *self {
                            *self -= amount;
                        }
                    } else {
                        amount = amount.max(1.0);
                        let amount = amount as Self;
                        if Self::MAX - *self >= amount {
                            *self += amount;
                        }
                    }
                    *self = (*self).clamp(min, max);
                }

                #[inline(always)]
                fn calc_t(&self, min: Self, max: Self) -> f32 {
                    if *self >= max { return 1.0 }
                    if *self <= min { return 0.0 }
                    let d0 = max - min;
                    let d1 = self - min;
                    d1 as f32 / d0 as f32
                }

                #[inline(always)]
                fn display<FontHash>(
                    &self,
                    _style: &impl WindowStyle<FontHash>,
                    to: &mut impl Write,
                ) -> core::fmt::Result
                {
                    write!(to, "{}", *self)
                }
            }
        )+
    };
}

impl_sliderable_int!(
    i8, i16, i32, i64, i128,
);

impl_sliderable_uint!(
    u8, u16, u32, u64, u128,
);
