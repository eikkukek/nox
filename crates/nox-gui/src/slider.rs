use core::fmt::Display;

use compact_str::CompactString;

use nox_geom::{
    shapes::Rect,
    BoundingRect,
    Vec2,
    vec2,
};

pub trait Sliderable: Copy + Display {

    fn slide(&mut self, min: Self, max: Self, t: f32);

    fn calc_t(&self, min: Self, max: Self) -> f32;
}

#[derive(Clone)]
pub(crate) struct Slider
{
    pub title: CompactString,
    pub main_rect: Rect,
    pub position: Vec2,
    pub t: f32,
    pub held: bool,
}

impl Slider
{

    pub fn new(
        t: f32,
        title: &str,
    ) -> Self
    {
        Self {
            title: CompactString::new(title),
            main_rect: Default::default(),
            position: Default::default(),
            t,
            held: false,
        }
    }

    pub fn slide<T: Sliderable>(&self, value: &mut T, min: T, max: T) {
        value.slide(min, max, self.t);
    }

    pub fn update(
        &mut self,
        position: Vec2,
        size: Vec2,
        rounding: f32,
    ) -> bool
    {
        let main_rect = Rect::from_position_size(vec2(0.0, 0.0), size, rounding);
        if main_rect != self.main_rect {
            self.main_rect = main_rect;
            return true
        }
        self.position = position;
        false
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
                    let d1 = max - self;
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
