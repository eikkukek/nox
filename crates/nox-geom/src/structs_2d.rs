use core::ops::Sub;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn mag(self, other: Self) -> f32 {
        let d = other - self;
        (d.x * d.x + d.y * d.y).sqrt()
    }
}

impl Sub for Point {

    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl From<[f32; 2]> for Point {

    #[inline(always)]
    fn from(value: [f32; 2]) -> Self {
        Point { x: value[0], y: value[1] }
    }
}

impl From<Point> for [f32; 2] {

    #[inline(always)]
    fn from(value: Point) -> Self {
        [value.x, value.y]
    }
}

#[inline(always)]
pub fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}
