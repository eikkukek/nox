use core::{
    ops::{Add, Sub, Mul, MulAssign, Neg},
    fmt::Display,
};

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

pub type Point = Vector;

impl Vector {

    #[inline(always)]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline(always)]
    pub fn mag_to(self, to: Self) -> f32 {
        let d = to - self;
        (d.x * d.x + d.y * d.y).sqrt()
    }

    #[inline(always)]
    pub fn normalized(self) -> Vector {
        let mag = (self.x * self.x + self.y * self.y).sqrt();
        Vector {
            x: self.x / mag,
            y: self.y / mag,
        }
    }

    #[inline(always)]
    pub fn right(self, to: Self) -> Vector {
        let a = to - self;
        Vector {
            x: a.y,
            y: -a.x,
        }
    }

    #[inline(always)]
    pub fn lerp(self, other: Self, t: f32) -> Vector {
        (1.0 - t) * self + t * other
    }
}

impl Add for Vector {

    type Output = Vector;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector {

    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for Vector {

    type Output = Vector;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vector> for f32 {

    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Vector {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl MulAssign<f32> for Vector {

    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Neg for Vector {

    type Output = Vector;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl From<[f32; 2]> for Vector {

    #[inline(always)]
    fn from(value: [f32; 2]) -> Self {
        Vector { x: value[0], y: value[1] }
    }
}

impl From<Vector> for [f32; 2] {

    #[inline(always)]
    fn from(value: Vector) -> Self {
        [value.x, value.y]
    }
}

impl Display for Vector {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "{ x: ".fmt(f)?;
        self.x.fmt(f)?;
        ", ".fmt(f)?;
        self.y.fmt(f)?;
        " }".fmt(f)
    }
}

#[inline(always)]
pub fn point(x: f32, y: f32) -> Vector {
    Vector { x, y }
}
