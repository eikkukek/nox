mod rect;
mod circle;

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
        for i in 0..n {
            let a: Vec2 = points[i].into();
            let b: Vec2 = points[(i + 1) % n].into();
            if a.dot(b).abs() < f32::EPSILON {
                collect(a + a.normalized() * width);
            } else {
                let c = a.right(b).normalized() * width;
                collect(a + c);
            }
        }
    } else {
        for i in (0..n).rev() {
            let a: Vec2 = points[i].into();
            let b: Vec2 = points[(i + 1) % n].into();
            if a.dot(b).abs() < f32::EPSILON {
                collect(a - a.normalized() * width);
            } else {
                let c = a.right(b).normalized() * width;
                collect(a - c);
            }
        }
    }
}
