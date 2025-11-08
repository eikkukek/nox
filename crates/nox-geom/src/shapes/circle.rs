use core::f32::consts::TAU;

use crate::*;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Circle {
    pub origin: Vec2,
    pub radius: f32,
}

pub fn circle<P: Into<Vec2>>(origin: P, radius: f32) -> Circle {
    Circle { origin: origin.into(), radius }
}

impl Circle {

    #[inline(always)]
    pub fn to_points(
        self,
        steps: u32,
        collect: &mut impl FnMut(Vec2),
    ) {

        let origin = self.origin;
        let radius = self.radius;
        let step = TAU / steps as f32;
        let mut angle = 0.0f32;

        for _ in 0..steps {
            let c = angle.cos();
            let s = angle.sin();
            collect(vec2(c * radius, s * radius) + origin);
            angle += step;
        }
    }

    #[inline(always)]
    pub fn to_points_cw(
        self,
        steps: u32,
        collect: &mut impl FnMut(Vec2),
    ) {

        let origin = self.origin;
        let radius = self.radius;
        let step = TAU / steps as f32;
        let mut angle = 0.0f32;

        for _ in 0..steps {
            let c = angle.cos();
            let s = angle.sin();
            collect(vec2(c * radius, s * radius) + origin);
            angle -= step;
        }
    }
}
