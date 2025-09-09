use nox_geom::{fn_2d::*, structs_2d::*};

use crate::*;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
    pub rounding: f32,
}

pub fn rect<P: Into<Point>>(min: P, max: P, rounding: f32) -> Rect {
    Rect { min: min.into(), max: max.into(), rounding }
}

impl Rect {

    #[inline(always)]
    pub fn scale(mut self, scalar: f32) -> Self {
        let half_size = (self.max - self.min) * 0.5;
        let center = self.min + half_size;
        let scaled = half_size * scalar;
        self.min = center - scaled;
        self.max = center + scaled;
        self.rounding *= scalar;
        self
    }

    #[inline(always)]
    pub fn inset(mut self, amount: f32) -> Self {

        self.min.x += amount;
        self.min.y += amount;
        self.max.x -= amount;
        self.max.y -= amount;
        self
    }

    #[inline(always)]
    pub fn translate(mut self, x: f32, y: f32) -> Self {
        let half_size = (self.max - self.min) * 0.5;
        self.min = point(x, y) - half_size;
        self.max = point(x, y) + half_size;
        self
    }

    #[inline(always)]
    pub fn widen(mut self, by: f32) -> Self {
        self.min.x -= by;
        self.max.x += by;
        self
    }

    #[inline(always)]
    pub fn heighten(mut self, by: f32) -> Self {
        self.min.y -= by;
        self.max.y += by;
        self
    }

    #[inline(always)]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            min: self.min.lerp(other.min, t),
            max: self.max.lerp(other.max, t),
            rounding: (1.0 - t) * self.rounding + t * other.rounding,
        }
    }

    #[inline(always)]
    pub fn position(&self) -> Point {
        self.min + (self.max - self.min) * 0.5
    }

    #[inline]
    pub fn to_points(
        self,
        bezier_depth: u32,
        mut collect: impl FnMut(Point),
    )
    {
        let (min, max) = (self.min, self.max);
        let radius = self.rounding;
        let magic = CUBIC_CIRCLE_MAGIC * radius;
        let tolerance = radius * 0.001;

        let mut a = point(min.x, min.y + radius);
        let mut b = point(min.x + radius, min.y);
        let mut p = a - point(0.0, magic);
        let mut q = b - point(magic, 0.0);
        flatten_cubic(a, p, q, b, tolerance, bezier_depth, &mut collect);

        a = point(max.x - radius, min.y);
        b = point(max.x, min.y + radius);
        p = a + point(magic, 0.0);
        q = b - point(0.0, magic);
        flatten_cubic(a, p, q, b, tolerance, bezier_depth, &mut collect); 

        a = point(max.x, max.y - radius);
        b = point(max.x - radius, max.y);
        p = a + point(0.0, magic);
        q = b + point(magic, 0.0);
        flatten_cubic(a, p, q, b, tolerance, bezier_depth, &mut collect);  

        a = point(min.x + radius, max.y);
        b = point(min.x, max.y - radius);
        p = a - point(magic, 0.0);
        q = b + point(0.0, magic);
        flatten_cubic(a, p, q, b, tolerance, bezier_depth, &mut collect);  
    }

    #[inline]
    pub fn to_points_cw(
        self,
        bezier_depth: u32,
        mut collect: impl FnMut(Point),
    )
    {
        let (min, max) = (self.min, self.max);
        let radius = self.rounding;
        let magic = CUBIC_CIRCLE_MAGIC * radius;
        let tolerance = radius * 0.001;

        let mut a = point(min.x + radius, min.y);
        let mut b = point(min.x, min.y + radius);
        let mut p = a - point(magic, 0.0);
        let mut q = b - point(0.0, magic);
        flatten_cubic(a, p, q, b, tolerance, bezier_depth, &mut collect);

        a = point(min.x, max.y - radius);
        b = point(min.x + radius, max.y);
        p = a + point(0.0, magic);
        q = b - point(magic, 0.0);
        flatten_cubic(a, p, q, b, tolerance, bezier_depth, &mut collect);  

        a = point(max.x - radius, max.y);
        b = point(max.x, max.y - radius);
        p = a + point(magic, 0.0);
        q = b + point(0.0, magic);
        flatten_cubic(a, p, q, b, tolerance, bezier_depth, &mut collect);  

        a = point(max.x, min.y + radius);
        b = point(max.x - radius, min.y);
        p = a - point(0.0, magic);
        q = b + point(magic, 0.0);
        flatten_cubic(a, p, q, b, tolerance, bezier_depth, &mut collect); 
    }
}
