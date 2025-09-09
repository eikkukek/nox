mod consts;
mod enums;
mod shapes;

pub use consts::*;
pub use enums::*;
pub use shapes::*;

use nox_geom::structs_2d::*;

pub fn outline_points<P: Into<Point> + Copy>(
    points: &[P],
    width: f32,
    clock_wise: bool,
    mut collect: impl FnMut(Point),
) {
    let n = points.len();
    if !clock_wise {
        for i in 0..n - 1 {
            let a: Point = points[i].into();
            let b: Point = points[i + 1].into();
            let c = a.right(b).normalized() * width;
            collect(a + c);
        }
    } else {
        for i in (0..n - 1).rev() {
            let a: Point = points[i].into();
            let b: Point = points[i + 1].into();
            let c = a.right(b).normalized() * width;
            collect(a - c);
        }
    }
}
