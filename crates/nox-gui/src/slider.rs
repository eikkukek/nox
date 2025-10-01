use core::{
    fmt::Display,
    hash::Hash,
};

use compact_str::CompactString;

use nox::mem::vec_types::GlobalVec;

use nox_font::*;

use nox_geom::{
    shapes::Rect,
    BoundingRect,
    Vec2,
    vec2,
};

use crate::{workspace::RingBufMem, *};

pub trait Sliderable: Copy + Display {

    fn slide(&mut self, min: Self, max: Self, t: f32);

    fn calc_t(&self, min: Self, max: Self) -> f32;
}

pub(crate) struct Slider
{
    pub title: CompactString,
    pub main_rect: Rect,
    pub position: Vec2,
    pub t: f32,
    pub held: bool,
    pub title_text: Option<RenderedText>,
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
            title_text: Default::default(),
        }
    }

    pub fn slide<T: Sliderable>(&self, value: &mut T, min: T, max: T) {
        value.slide(min, max, self.t);
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
    pub fn update<FontHash, F1, F2>(
        &mut self,
        style: &Style,
        vertex_text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        font: FontHash,
    ) -> bool
        where 
            FontHash: Clone + Eq + Hash,
    {
        if self.title_text.is_none() {
            self.title_text = Some(vertex_text_renderer
                .render(&[text_segment(self.title.as_str(), font)], false, 1.0)
                .unwrap());
        }
        let title_text = self.title_text.as_ref().unwrap();
        let main_rect = Rect::from_position_size(
            vec2(0.0, 0.0),
            vec2(title_text.text_width * style.font_scale, style.calc_item_height()),
            0.0,
        );
        if main_rect != self.main_rect {
            self.main_rect = main_rect;
            return true
        }
        false
    }

    #[inline(always)]
    pub fn triangulate<Tri>(&mut self, triangulate: Tri)
        where
    {
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
