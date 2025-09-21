use crate::{*, bezier::cubic};

///```
///4.0 / 3.0 * (2.0.sqrt() - 1.0)
const RECT_ROUND_MAGIC: f32 = 0.5522848;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
    pub rounding: f32,
}

pub fn rect<P: Into<Vec2>>(min: P, max: P, rounding: f32) -> Rect {
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
        self.min = vec2(x, y) - half_size;
        self.max = vec2(x, y) + half_size;
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
    pub fn position(&self) -> Vec2 {
        self.min + (self.max - self.min) * 0.5
    }

    #[inline]
    pub fn to_points(
        self,
        collect: &mut impl FnMut(Vec2),
    )
    {
        let (min, max) = (self.min, self.max);
        let radius = self.rounding;
        if radius.abs() < f32::EPSILON {
            collect(min);
            collect(vec2(max.x, min.y));
            collect(max);
            collect(vec2(min.x, max.y));
            return
        }
        let magic = RECT_ROUND_MAGIC * radius;
        let tolerance = radius * 0.001;

        let mut a = vec2(min.x, min.y + radius);
        let mut b = vec2(min.x + radius, min.y);
        let mut p = a - vec2(0.0, magic);
        let mut q = b - vec2(magic, 0.0);
        cubic(a, p, q, b).flatten(tolerance, collect);

        a = vec2(max.x - radius, min.y);
        b = vec2(max.x, min.y + radius);
        p = a + vec2(magic, 0.0);
        q = b - vec2(0.0, magic);
        cubic(a, p, q, b).flatten(tolerance, collect); 

        a = vec2(max.x, max.y - radius);
        b = vec2(max.x - radius, max.y);
        p = a + vec2(0.0, magic);
        q = b + vec2(magic, 0.0);
        cubic(a, p, q, b).flatten(tolerance, collect);

        a = vec2(min.x + radius, max.y);
        b = vec2(min.x, max.y - radius);
        p = a - vec2(magic, 0.0);
        q = b + vec2(0.0, magic);
        cubic(a, p, q, b).flatten(tolerance, collect);
    }

    #[inline]
    pub fn to_points_cw(
        self,
        collect: &mut impl FnMut(Vec2),
    )
    {
        let (min, max) = (self.min, self.max);
        let radius = self.rounding;

        if radius.abs() < f32::EPSILON {
            collect(min);
            collect(vec2(min.x, max.y));
            collect(max);
            collect(vec2(max.x, min.y));
            return
        }

        let magic = RECT_ROUND_MAGIC * radius;
        let tolerance = radius * 0.001;

        let mut a = vec2(min.x + radius, min.y);
        let mut b = vec2(min.x, min.y + radius);
        let mut p = a - vec2(magic, 0.0);
        let mut q = b - vec2(0.0, magic);
        cubic(a, p, q, b).flatten(tolerance, collect);

        a = vec2(min.x, max.y - radius);
        b = vec2(min.x + radius, max.y);
        p = a + vec2(0.0, magic);
        q = b - vec2(magic, 0.0);
        cubic(a, p, q, b).flatten(tolerance, collect);

        a = vec2(max.x - radius, max.y);
        b = vec2(max.x, max.y - radius);
        p = a + vec2(magic, 0.0);
        q = b + vec2(0.0, magic);
        cubic(a, p, q, b).flatten(tolerance, collect);

        a = vec2(max.x, min.y + radius);
        b = vec2(max.x - radius, min.y);
        p = a - vec2(0.0, magic);
        q = b + vec2(magic, 0.0);
        cubic(a, p, q, b).flatten(tolerance, collect);
    }

    #[inline(always)]
    pub fn eq_epsilon(&self, rhs: &Rect, epsilon: f32) -> bool {
        return
            self.min.eq_epsilon(rhs.min, epsilon) &&
            self.max.eq_epsilon(rhs.max, epsilon) &&
            (self.rounding - rhs.rounding).abs() < epsilon
    }
}
