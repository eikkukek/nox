use core::f32::consts::TAU;

use nox_geom::{structs_2d::*};

#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub origin: Point,
    pub radius: f32,
}

pub fn circle<P: Into<Point>>(origin: P, radius: f32) -> Circle {
    Circle { origin: origin.into(), radius }
}

impl Circle {

    #[inline(always)]
    pub fn to_points(
        self,
        steps: u32,
        mut collect: impl FnMut(Point),
    ) {

        let origin = self.origin;
        let radius = self.radius;
        let step = TAU / steps as f32;
        let mut angle = 0.0f32;

        for _ in 0..steps {
            let c = angle.cos();
            let s = angle.sin();
            collect(point(c * radius, s * radius) + origin);
            angle += step;
        }
    }
}
