use core::{
    ops::{
        Add, AddAssign, Sub, SubAssign,
        Mul, MulAssign, Div, DivAssign, Neg,
    },
    fmt::Display,
};

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[inline(always)]
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl Vec2 {

    #[inline(always)]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline(always)]
    pub fn sqr_mag(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline(always)]
    pub fn mag(self) -> f32 {
        let mag = self.sqr_mag();
        if (mag - 1.0).abs() < f32::EPSILON {
            return mag
        }
        mag.sqrt()
    }

    #[inline(always)]
    pub fn mag_to(self, to: Self) -> f32 {
        let d = to - self;
        d.mag()
    }

    #[inline(always)]
    pub fn normalized(self) -> Self {
        let mag = self.mag();
        if mag.abs() < f32::EPSILON {
            return vec2(0.0, 0.0)
        }
        Vec2 {
            x: self.x / mag,
            y: self.y / mag,
        }
    }

    #[inline(always)]
    pub fn cross(self, rhs: Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }

    #[inline(always)]
    pub fn right(self, to: Self) -> Self {
        let a = to - self;
        Vec2 {
            x: a.y,
            y: -a.x,
        }
    }

    #[inline(always)]
    pub fn rotated(self, rad: f32) -> Self {
        let cos = rad.cos();
        let sin = rad.sin();
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    #[inline(always)]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        (1.0 - t) * self + t * other
    }

    #[inline(always)]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }

    #[inline(always)]
    pub fn eq_epsilon(self, rhs: Self, epsilon: f32) -> bool {
        return (self.x - rhs.x).abs() < epsilon &&
                (self.y - rhs.y).abs() < epsilon
    }

    #[inline(always)]
    #[must_use]
    pub fn min(self, rhs: Self) -> Self {
        Self {
            x: self.x.min(rhs.x),
            y: self.y.min(rhs.y),
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn max(self, rhs: Self) -> Self {
        Self {
            x: self.x.max(rhs.x),
            y: self.y.max(rhs.y),
        }
    }
}

impl Add for Vec2 {

    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2 {

    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {

    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Vec2 {

    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {

    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vec2> for f32 {

    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl MulAssign<f32> for Vec2 {

    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Neg for Vec2 {

    type Output = Vec2;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Div<f32> for Vec2 {

    type Output = Vec2;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign<f32> for Vec2 {

    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl From<[f32; 2]> for Vec2 {

    #[inline(always)]
    fn from(value: [f32; 2]) -> Self {
        Vec2 { x: value[0], y: value[1] }
    }
}

impl From<Vec2> for [f32; 2] {

    #[inline(always)]
    fn from(value: Vec2) -> Self {
        [value.x, value.y]
    }
}

impl From<(f32, f32)> for Vec2 {

    #[inline(always)]
    fn from(value: (f32, f32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<Vec2> for (f32, f32) {

    #[inline(always)]
    fn from(value: Vec2) -> Self {
        (value.x, value.y)
    }
}

impl Display for Vec2 {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "{ x: ".fmt(f)?;
        self.x.fmt(f)?;
        ", ".fmt(f)?;
        self.y.fmt(f)?;
        " }".fmt(f)
    }
}
