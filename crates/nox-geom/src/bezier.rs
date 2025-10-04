use crate::{fn_2d::line_intersection, *};

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Quad {
    pub start: Vec2,
    pub mid: Vec2,
    pub end: Vec2,
}

#[inline(always)]
pub fn quad(
    start: Vec2,
    mid: Vec2,
    end: Vec2,
) -> Quad {
    Quad {
        start,
        mid,
        end,
    }
}

impl Quad {

    pub fn eval(&self, t: f32) -> Vec2 {
        let t2 = 1.0 - t;
        t2 * t2 * self.start + 2.0 * t2 * t * self.mid + t * t * self.end
    }

    pub fn flatten<F>(
        &self,
        tolerance: f32,
        collect: &mut F
    )
        where 
            F: FnMut(Vec2)
    {
        if tolerance.abs() < f32::EPSILON {
            return;
        }
        // approx of (1 + 4x^2)^-0.25
        fn approx_integral(x: f32) -> f32 {
            const NUM1: f32 = 0.67;
            const NUM2: f32 = NUM1 * NUM1 * NUM1 * NUM1;
            x / (1.0 - NUM1 + (NUM2 + 0.25 * x * x).powf(0.25))
        }

        // approx of (1 + 4x^2)^0.25
        fn approx_inv_integral(x: f32) -> f32 {
            const NUM1: f32 = 0.39;
            const NUM2: f32 = NUM1 * NUM1;
            x * (1.0 - NUM1 + (NUM2 + 0.25 * x * x).sqrt())
        }

        let &Quad { start, mid, end } = self;

        let dd = 2.0 * mid - start - end;
        let u0 = (mid.x - start.x) * dd.x + (mid.y - start.y) * dd.y;
        let u2 = (end.x - mid.x) * dd.x + (end.y - mid.y) * dd.y;
        let cross = (end.x - start.x) * dd.y - (end.y - start.y) * dd.x;
        let cross_abs = cross.abs();
        if cross_abs < f32::EPSILON {
            collect(start);
            collect(end);
            return;
        }
        let x0 = u0 / cross;
        let x2 = u2 / cross;
        let scale = cross_abs / (vec2(dd.x, dd.y).mag() * (x2 - x0).abs());
        let a0 = approx_integral(x0);
        let a2 = approx_integral(x2);
        let x0 = approx_inv_integral(a0);
        let x2 = approx_inv_integral(a2);
        let n = 0.5 * (a2 - a0).abs() * (scale / tolerance).sqrt();
        for i in 0..n.ceil() as u32 {
            let u = approx_inv_integral(a0 + ((a2 - a0) * i as f32) / n);
            let t = (u - x0) / (x2 - x0);
            collect(quad(start, mid, end).eval(t));
        }
        collect(end);
    }
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Cubic {
    pub start: Vec2,
    pub mid0: Vec2,
    pub mid1: Vec2,
    pub end: Vec2,
}

#[inline(always)]
pub fn cubic(
    start: Vec2,
    mid0: Vec2,
    mid1: Vec2,
    end: Vec2
) -> Cubic
{
    Cubic {
        start,
        mid0,
        mid1,
        end,
    }
}

impl Cubic {

    pub fn split(&self, t: f32) -> (Self, Self) {
        let q0 = self.start.lerp(self.mid0, t);
        let q1 = self.mid0.lerp(self.mid1, t);
        let q2 = self.mid1.lerp(self.end, t);

        let r0 = q0.lerp(q1, t);
        let r1 = q1.lerp(q2, t);

        let mid = r0.lerp(r1, t);

        let left = cubic(self.start, q0, r0, mid);
        let right = cubic(mid, r1, q2, self.end);
        (left, right)
    }

    pub fn to_quad(&self) -> Option<Quad> {
        let mid = line_intersection(self.start, self.mid0, self.mid1, self.end)?;
        Some(quad(self.start, mid, self.end))
    }

    pub fn flatten<F>(
        &self,
        tolerance: f32,
        collect: &mut F
    )
        where 
            F: FnMut(Vec2)
    {
        const N_MUL: f32 = 36.0 * 36.0 / 3.0;
        const N_POW: f32 = 1.0 / 6.0;
        let err = self.end - self.mid1 * 3.0 + self.mid0 * 3.0 - self.start;
        let mut n = (err.sqr_mag() / (N_MUL * tolerance * tolerance)).powf(N_POW) as u32;
        if n == 0 {
            collect(self.start);
            collect(self.end);
            return
        }
        if n == 1 {
            if let Some(quad) = self.to_quad() {
                quad.flatten(tolerance, collect);
            } else {
                n = 3;
            }
        }
        let mut current = *self;
        for i in (1..n).rev() {
            let t = 1.0 / i as f32;
            let (left, right) = current.split(t);
            if let Some(quad) = left.to_quad() {
                quad.flatten(tolerance, collect);
            }
            current = right;
        }
    }
}
