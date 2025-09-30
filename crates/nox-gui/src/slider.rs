use core::fmt::Display;

pub trait Sliderable: Copy + Display {

    fn slide(&mut self, min: Self, max: Self, t: f32);

    fn calc_t(&self, min: Self, max: Self) -> f32;
}

#[derive(Clone)]
pub(crate) struct Slider
{
    pub t: f32,
    pub clicked: bool,
    pub title: String,
}

impl Slider
{

    pub fn new(
        t: f32,
        title: String,
    ) -> Self
    {
        Self {
            t,
            clicked: false,
            title,
        }
    }

    pub fn slide<T: Sliderable>(&self, value: &mut T, min: T, max: T) {
        value.slide(min, max, self.t);
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
