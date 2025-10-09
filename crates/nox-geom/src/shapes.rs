mod rect;
mod circle;

use core::f32::consts::PI;

pub use rect::*;
pub use circle::*;

use crate::*;

pub fn outline_points<F, P>(
    points: &[P],
    width: f32,
    clock_wise: bool,
    collect: &mut F,
)
    where
        F: FnMut(Vec2),
        P: Into<Vec2> + Copy,
{
    let n = points.len();
    if n < 2 {
        return
    }
    if !clock_wise {
        const MAGIC_RADIANS: f32 = -3.0 * PI / 4.0; // -(360° - 90°) / 2
        let mut prev_ab: Vec2 = points[0].into() - points[n - 1].into();
        for i in 0..n {
            let a: Vec2 = points[i].into();
            let b: Vec2 = points[(i + 1) % n].into();
            let ab = b - a;
            if prev_ab.dot(ab).abs() < f32::EPSILON {
                collect(a + ab.rotated(MAGIC_RADIANS).normalized() * width);
            } else {
                let c = a.right(b).normalized() * width;
                collect(a + c);
            }
            prev_ab = ab;
        }
    } else {
        const MAGIC_RADIANS: f32 = 3.0 * PI / 4.0; // (360° - 90°) / 2
        let mut prev_ab: Vec2 = points[1].into() - points[0].into();
        for i in (0..n).rev() {
            let a: Vec2 = points[i].into();
            let b: Vec2 = points[(i + 1) % n].into();
            let ab = b - a;
            if prev_ab.dot(ab).abs() < f32::EPSILON {
                collect(a + ab.rotated(MAGIC_RADIANS).normalized() * width);
            } else {
                let c = b.right(a).normalized() * width;
                collect(a + c);
            }
            prev_ab = ab;
        }
    }
}
