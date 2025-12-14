use core::{
    fmt::Write,
    str::FromStr,
};

use nox_geom::{
    *,
    shapes::*,
};

use crate::{
    surface::*,
    *,
};

pub trait Sliderable: Copy + FromStr + PartialEq {

    const MIN: Self;
    const MAX: Self;

    fn slide_and_quantize_t(&mut self, min: Self, max: Self, t: f32) -> f32;

    fn drag(&mut self, min: Self, max: Self, amount: f32);

    fn calc_t(&self, min: Self, max: Self) -> f32;

    fn display(
        &self,
        style: &impl UiStyle,
        to: &mut impl Write, 
    ) -> core::fmt::Result;
}

pub struct SliderData {
    width: f32,
    t: f32,
    quantized_t: f32,
    flags: u32,
}

impl SliderData {

    const HELD: u32 = 0x1;

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            t: 0.0,
            quantized_t: 0.0,
            width: 0.0,
            flags: 0,
        }
    }

    #[inline(always)]
    pub fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    pub fn update_value<T: Sliderable>(
        &mut self,
        slider_width: f32,
        value: &mut T,
        min: T,
        max: T,
    )
    {
        self.width = slider_width;
        if self.held() {
            self.quantized_t = value.slide_and_quantize_t(min, max, self.t);
        } else {
            self.t = value.calc_t(min, max);
            self.quantized_t = self.t;
        }
    }

    #[inline(always)]
    fn handle_offset(
        &self,
        mut offset: Vec2,
        slider_size: Vec2,
        handle_size: Vec2,
    ) -> Vec2
    {
        offset.x += (slider_size.x - handle_size.x) * self.quantized_t;
        offset
    }

    #[inline(always)]
    fn calc_t(
        &self,
        mut rel_cursor_pos: Vec2,
        slider_size: Vec2,
        handle_size: Vec2,
    ) -> f32
    {
        rel_cursor_pos.x -= handle_size.x * 0.5;
        // handle_pos solved for t
        let t = rel_cursor_pos.x / (slider_size.x - handle_size.x);
        t.clamp(0.0, 1.0)
    }

    #[inline(always)]
    pub fn update<Surface: UiReactSurface, Style: UiStyle>(
        &mut self,
        ui: &mut UiReactContext<Surface, Style>,
        reaction: &mut Reaction,
    )
    {
        let radius = ui.style().default_handle_radius();
        let diameter = radius * 2.0;
        let slider_size = vec2(
            self.width,
            diameter * 0.8,
        );
        let rel_cursor_pos = reaction.rel_cursor_pos();
        let handle_size = vec2(radius * 1.5, diameter * 1.1);
        self.flags = 0;
        if reaction.held() {
            self.t = self.calc_t(rel_cursor_pos, slider_size, handle_size);
            self.flags |= Self::HELD;
        }
        let size = vec2(slider_size.x, handle_size.y);
        reaction.size = size;
        let offset = reaction.offset();
        let handle_offset = self.handle_offset(offset, slider_size, handle_size);
        let id = reaction.id();
        let rounding = ui.style().rounding();
        let visuals = ui.style().interact_visuals(reaction);
        ui.paint(move |painter, row| {
            painter
                .rect(
                    id,
                    rect(Default::default(), slider_size, rounding),
                    offset + vec2(0.0, row.height_halved - slider_size.y * 0.5),
                    visuals.fill_col,
                    visuals.bg_strokes.clone(),
                    visuals.bg_stroke_idx
                )
                .rect(
                    id,
                    rect(Default::default(), handle_size, rounding),
                    handle_offset + vec2(0.0, row.height_halved - handle_size.y * 0.5),
                    visuals.fill_col,
                    visuals.fg_strokes.clone(),
                    visuals.fg_stroke_idx
                );
        });
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
    fn display(
        &self,
        style: &impl UiStyle,
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
    fn display(
        &self,
        style: &impl UiStyle,
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
                        let clamp = *self < 0 && Self::MIN - *self >= amount;
                        if !clamp {
                            *self += amount;
                        } else {
                            *self = Self::MIN;
                        }
                    } else {
                        let amount = amount as Self;
                        let clamp = *self > 0 && Self::MAX - *self <= amount;
                        if !clamp {
                            *self += amount;
                        } else {
                            *self = Self::MAX;
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
                fn display(
                    &self,
                    _style: &impl UiStyle,
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
                        } else {
                            *self = 0;
                        }
                    } else {
                        let amount = amount as Self;
                        if Self::MAX - *self >= amount {
                            *self += amount;
                        } else {
                            *self = Self::MAX;
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
                fn display(
                    &self,
                    _style: &impl UiStyle,
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
