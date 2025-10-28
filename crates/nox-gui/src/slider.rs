use core::{
    fmt::Write,
    hash::Hash,
    marker::PhantomData,
    str::FromStr,
};

use nox::{
    mem::vec_types::{GlobalVec, Vector},
    *,
};

use nox_font::{VertexTextRenderer, RenderedText};

use nox_geom::{
    *,
    shapes::*,
};

use crate::*;

pub trait Sliderable: Copy + FromStr + PartialEq {

    const MIN: Self;
    const MAX: Self;

    fn slide_and_quantize_t(&mut self, min: Self, max: Self, t: f32) -> f32;

    fn drag(&mut self, min: Self, max: Self, amount: f32);

    fn calc_t(&self, min: Self, max: Self) -> f32;

    fn display<FontHash>(
        &self,
        style: &impl WindowStyle<FontHash>,
        to: &mut impl Write, 
    ) -> core::fmt::Result;
}

pub struct Slider<I, FontHash, Style>
{
    slider_rect: Rect,
    slider_rect_vertex_range: VertexRange,
    max_y: f32,
    handle_rect: Rect,
    handle_rect_vertex_range: VertexRange,
    regular_handle_outline_width: f32,
    regular_handle_outline_vertex_range: VertexRange,
    active_handle_outline_width: f32,
    active_handle_outline_vertex_range: VertexRange,
    offset: Vec2,
    drag_value: DragValue<I, FontHash, Style>,
    t: f32,
    quantized_t: f32,
    width: f32,
    falgs: u32,
    _marker: PhantomData<(I, FontHash, Style)>,
}

impl<I, FontHash, Style> Slider<I, FontHash, Style>
    where
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
{

    const HELD: u32 = 0x1;
    const CURSOR_IN_SLIDER: u32 = 0x2;

    #[inline(always)]
    pub fn new() -> Self
    {
        Self {
            slider_rect: Default::default(),
            slider_rect_vertex_range: Default::default(),
            max_y: 0.0,
            handle_rect: Default::default(),
            handle_rect_vertex_range: Default::default(),
            regular_handle_outline_width: 0.0,
            regular_handle_outline_vertex_range: Default::default(),
            active_handle_outline_width: 0.0,
            active_handle_outline_vertex_range: Default::default(),
            offset: Default::default(),
            drag_value: DragValue::new(),
            t: 1.0,
            quantized_t: 1.0,
            width: 0.1,
            falgs: 0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    fn handle_off(
        &self,
        offset: Vec2,
    ) -> Vec2
    {
        let mut pos = offset;
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
    fn held(&self) -> bool {
        self.falgs & Self::HELD == Self::HELD
    }

    #[inline(always)]
    fn cursor_in_slider(&self) -> bool {
        self.falgs & Self::CURSOR_IN_SLIDER == Self::CURSOR_IN_SLIDER
    }  

    #[inline(always)]
    pub fn update_value<T: Sliderable>(
        &mut self,
        style: &Style,
        slider_width: f32,
        value: &mut T,
        min: T,
        max: T,
        drag_speed: f32,
    )
    {
        self.drag_value.set_input_params(style, style.min_input_text_width(), None, false);
        self.width = slider_width;
        self.drag_value.calc_value(style, value, T::MIN, T::MAX, drag_speed);
        if self.held() {
            self.quantized_t = value.slide_and_quantize_t(min, max, self.t);
        } else {
            self.t = value.calc_t(min, max);
            self.quantized_t = self.t;
        }
    }
}

impl<I, FontHash, Style> Widget<I, FontHash, Style> for Slider<I, FontHash, Style>
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

    fn calc_size(
        &mut self,
        style: &Style,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
    ) -> Vec2
    {
        let slider_size = vec2(
            self.width,
            style.default_handle_radius() * 2.0,
        );
        let drag_value_size = self.drag_value.calc_size(style, text_renderer);
        let max_y = slider_size.y.max(drag_value_size.y);
        self.max_y = max_y;
        vec2(
            slider_size.x + style.item_pad_outer().x + drag_value_size.x,
            max_y,
        )
    }

    fn status<'a>(
        &'a self,
        nox: &Nox<I>,
        style: &Style,
        window_pos: Vec2,
        cursor_pos: Vec2
    ) -> WidgetStatus<'a>
    {
        match self.drag_value.status(nox, style, window_pos, cursor_pos) {
            WidgetStatus::Active => {
                return WidgetStatus::Active
            },
            WidgetStatus::Hovered(_) => {
                return WidgetStatus::Hovered(None)
            },
            WidgetStatus::Inactive => {}
        };
        if self.held() {
            WidgetStatus::Active
        } else if self.cursor_in_slider() {
            WidgetStatus::Hovered(None)
        } else {
            WidgetStatus::Inactive
        }
    }

    fn update(
        &mut self,
        nox: &mut Nox<I>,
        style: &Style,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        window_size: Vec2,
        window_pos: Vec2,
        content_offset: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        mut other_widget_active: bool,
        cursor_in_other_widget: bool,
        window_moving: bool,
        hover_blocked: bool,
        collect_text: &mut dyn FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult
        where
            I: Interface,
            FontHash: Clone + Eq + Hash
    {
        let width = self.width;
        let diameter = style.default_handle_radius() * 2.0;
        let offset = self.offset + vec2(0.0, self.max_y * 0.5 - self.slider_rect.max.y * 0.5);
        let slider_rect = rect(
            Default::default(),
            vec2(
                width,
                diameter * 0.8,
            ),
            style.rounding(),
        );
        let handle_height = diameter * 1.1;
        let handle_rect = rect(
            vec2(0.0, slider_rect.max.y * 0.5 - handle_height * 0.5),
            vec2(
                style.default_handle_radius() * 1.5,
                handle_height,
            ),
            style.rounding(),
        );
        let requires_triangulation =
            slider_rect != self.slider_rect ||
            handle_rect != self.handle_rect ||
            self.regular_handle_outline_width != style.focused_widget_outline_width() ||
            self.active_handle_outline_width != style.active_widget_outline_width();
        self.regular_handle_outline_width = style.focused_widget_outline_width();
        self.active_handle_outline_width = style.active_widget_outline_width();
        self.slider_rect = slider_rect;
        self.handle_rect = handle_rect;
        let mut cursor_in_widget = false;
        self.falgs &= !Self::CURSOR_IN_SLIDER;
        self.drag_value.set_offset(self.offset + vec2(width + style.item_pad_outer().x, 0.0));
        let drag_result = self.drag_value.update(
            nox,
            style,
            text_renderer,
            window_size, window_pos, content_offset,
            cursor_pos, delta_cursor_pos,
            cursor_in_this_window, other_widget_active, cursor_in_other_widget,
            window_moving, hover_blocked, collect_text
        );
        let drag_status = self.drag_value.status(nox, style, window_pos, cursor_pos);
        other_widget_active |= matches!(drag_status, WidgetStatus::Active);
        if self.held() {
            cursor_in_widget = true;
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.falgs &= !Self::HELD;
            } else {
                self.t = self.calc_t(cursor_pos, window_pos + offset);
            }
        } else if cursor_in_this_window && !other_widget_active {
            let bounding_rect = BoundingRect::from_position_size(
                window_pos + offset,
                self.slider_rect.max,
            );
            cursor_in_widget = bounding_rect.is_point_inside(cursor_pos);
            if cursor_in_widget {
                self.falgs |= Self::CURSOR_IN_SLIDER;
                if nox.was_mouse_button_pressed(MouseButton::Left) {
                    self.falgs |= Self::HELD;
                    self.t = self.calc_t(cursor_pos, window_pos + offset);
                }
            }
        }
        UpdateResult {
            requires_triangulation: requires_triangulation || drag_result.requires_triangulation,
            cursor_in_widget: cursor_in_widget || drag_result.cursor_in_widget,
        }
    }

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        helper_points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    )
    {
        self.slider_rect.to_points(&mut |p| { points.push(p.into()); });
        self.slider_rect_vertex_range = tri(&points);
        points.clear();
        self.handle_rect.to_points(&mut |p| { points.push(p.into()); });
        outline_points(points,
            self.regular_handle_outline_width, false, &mut |p| { helper_points.push(p.into()); });
        self.regular_handle_outline_vertex_range = tri(&helper_points);
        helper_points.clear();
        outline_points(points,
            self.active_handle_outline_width, false, &mut |p| { helper_points.push(p.into()); });
        self.active_handle_outline_vertex_range = tri(&helper_points);
        self.handle_rect_vertex_range = tri(&points);
        points.clear();
        helper_points.clear();
        self.drag_value.triangulate(points, helper_points, tri);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        vertices: &mut [Vertex],
    )
    {
        let mut offset = self.offset + vec2(0.0, self.max_y * 0.5 - self.slider_rect.max.y * 0.5);
        let mut target_color = style.widget_bg_col();
        set_vertex_params(vertices, self.slider_rect_vertex_range, offset, target_color);
        offset = self.handle_off(offset);
        set_vertex_params(vertices, self.handle_rect_vertex_range, offset, target_color);
        if self.held() || self.cursor_in_slider() {
            target_color =
                if self.held() {
                    style.active_widget_outline_col()
                } else {
                    style.focused_widget_outline_col()
                };
            set_vertex_params(vertices, self.active_handle_outline_vertex_range, offset, target_color);
            hide_vertices(vertices, self.regular_handle_outline_vertex_range);
        } else {
            target_color = style.inactive_widget_outline_col();
            set_vertex_params(vertices, self.regular_handle_outline_vertex_range, offset, target_color);
            hide_vertices(vertices, self.active_handle_outline_vertex_range);
        }
        self.drag_value.set_vertex_params(style, vertices);
    }

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, Style>>, Error>
    {
        self.drag_value.render_commands(
            render_commands, style, base_pipeline_id, text_pipeline_id,
            vertex_buffer, index_buffer,
            window_pos, inv_aspect_ratio, unit_scale, get_custom_pipeline
        )
    }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    ) {
        hide_vertices(vertices, self.slider_rect_vertex_range);
        hide_vertices(vertices, self.handle_rect_vertex_range);
        hide_vertices(vertices, self.active_handle_outline_vertex_range);
        hide_vertices(vertices, self.regular_handle_outline_vertex_range);
        self.drag_value.hide(vertices);
    }
}

impl Sliderable for f32 {

    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;

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

    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;

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

                const MIN: Self = <$t>::MIN;
                const MAX: Self = <$t>::MAX;

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
                fn drag(&mut self, min: Self, max: Self, amount: f32) {
                    if amount.abs() < f32::EPSILON {
                        return
                    }
                    if amount.is_sign_negative() {
                        let amount = amount as Self;
                        if (*self > 0 && amount >= Self::MIN) || Self::MIN - *self <= amount {
                            *self += amount;
                        }
                    } else {
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

                const MIN: Self = <$t>::MIN;
                const MAX: Self = <$t>::MAX;

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
                fn drag(&mut self, min: Self, max: Self, amount: f32) {
                    if amount.abs() < f32::EPSILON {
                        return
                    }
                    if amount.is_sign_negative() {
                        let amount = amount.abs() as Self;
                        if amount <= *self {
                            *self -= amount;
                        }
                    } else {
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
