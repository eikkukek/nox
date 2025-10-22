use core::ops::Deref;

use nox_mem::vec_types::{GlobalVec, Vector};

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
        let tm1 = 1.0 - t;
        tm1 * tm1 * self.start + 2.0 * tm1 * t * self.mid + t * t * self.end
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
    pub mid_0: Vec2,
    pub mid_1: Vec2,
    pub end: Vec2,
}

#[inline(always)]
pub fn cubic(
    start: Vec2,
    mid_0: Vec2,
    mid_1: Vec2,
    end: Vec2
) -> Cubic
{
    Cubic {
        start,
        mid_0,
        mid_1,
        end,
    }
}

impl Cubic {

    pub fn eval(&self, t: f32) -> Vec2 {
        let tm1 = 1.0 - t;
        let tm1_2 = tm1 * tm1;
        let t_2 = t * t;
        tm1_2 * tm1 * self.start +
        3.0 * tm1_2 * t * self.mid_0 +
        3.0 * tm1 * t_2 * self.mid_1 +
        t_2 * t * self.end
    }

    pub fn split(&self, t: f32) -> (Self, Self) {
        let q0 = self.start.lerp(self.mid_0, t);
        let q1 = self.mid_0.lerp(self.mid_1, t);
        let q2 = self.mid_1.lerp(self.end, t);

        let r0 = q0.lerp(q1, t);
        let r1 = q1.lerp(q2, t);

        let mid = r0.lerp(r1, t);

        let left = cubic(self.start, q0, r0, mid);
        let right = cubic(mid, r1, q2, self.end);
        (left, right)
    }

    pub fn to_quad(&self) -> Option<Quad> {
        let mid = line_intersection(self.start, self.mid_0, self.mid_1, self.end)?;
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
        let err = self.end - self.mid_1 * 3.0 + self.mid_0 * 3.0 - self.start;
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

    #[inline(always)]
    pub fn min_y(&self) -> f32 {
        self.start.y
            .min(self.mid_0.y)
            .min(self.mid_1.y)
            .min(self.end.y)
    }

    #[inline(always)]
    pub fn max_y(&self) -> f32 {
        self.start.y
            .max(self.mid_0.y)
            .max(self.mid_1.y)
            .max(self.end.y)
    }
}

#[derive(Clone, PartialEq)]
pub struct AnimationCurve {
    cubics: GlobalVec<Cubic>,
    tolerance: f32,
}

impl AnimationCurve {

    #[inline(always)]
    pub fn new(c: Cubic, tolerance: f32) -> Self {
        let c =
            if c.start.x <= c.end.x {
                let mid_0_x = c.mid_0.x.clamp(c.start.x, c.end.x);
                let mid_1_y = c.mid_1.x.clamp(c.start.x, c.end.x);
                cubic(
                    c.start,
                    vec2(mid_0_x, c.mid_0.y),
                    vec2(mid_1_y, c.mid_1.y),
                    c.end
                )
            } else {
                let mid_0_x = c.mid_0.x.clamp(c.end.x, c.start.x);
                let mid_1_x = c.mid_1.x.clamp(c.end.x, c.start.x);
                cubic(
                    c.end,
                    vec2(mid_0_x, c.mid_0.y),
                    vec2(mid_1_x, c.mid_1.y),
                    c.start,
                )
            };
        Self {
            cubics: GlobalVec::with_len(1, c),
            tolerance,
        }
    }

    #[inline(always)]
    pub fn tolerance(&self) -> f32 {
        self.tolerance
    }

    #[inline(always)]
    pub fn set_tolerance(&self) -> f32 {
        self.tolerance
    }

    #[inline(always)]
    pub fn min_coords(&self) -> Vec2 {
        vec2(
            self.cubics[0].start.x,
            self.cubics
                .iter()
                .min_by(|&a, &b| a.min_y().total_cmp(&b.min_y()))
                .map(|&c| c.min_y())
                .unwrap_or_default()
        )
    }

    #[inline(always)]
    pub fn max_coords(&self) -> Vec2 {
        vec2(
            self.cubics.last().unwrap().end.x,
            self.cubics
                .iter()
                .max_by(|&a, &b| a.max_y().total_cmp(&b.max_y()))
                .map(|&c| c.max_y())
                .unwrap_or_default()
        )
    }

    pub fn add_point(
        &mut self,
        point: Vec2,
    ) -> &mut Self
    {
        let min_x = self.min_coords().x;
        let max_x = self.max_coords().x;
        if (point.x - min_x).abs() < f32::EPSILON ||
            (point.x - max_x).abs() < f32::EPSILON
        {
            return self
        }
        if point.x < min_x {
            let first = self.cubics[0];
            let d = first.start - point;
            self.cubics.insert(0, cubic(point, d * 0.25, d * 0.75, first.start));
            return self
        }
        if point.x > max_x {
            let last = self.cubics.last().unwrap();
            let d = point - last.end;
            self.cubics.push(cubic(last.end, d * 0.25, d * 0.75, point));
            return self
        }
        let mut idx = None;
        for (i, cubic) in self.cubics.iter().enumerate() {
            if cubic.start.x > point.x {
                idx = Some(i);
            }
        }
        let idx = idx.unwrap();
        let c = self.cubics[idx];
        const N_MUL: f32 = 36.0 * 36.0 / 3.0;
        const N_POW: f32 = 1.0 / 6.0;
        let err = c.end - c.mid_1 * 3.0 + c.mid_0 * 3.0 - c.start;
        let n = (err.sqr_mag() / (N_MUL * self.tolerance * self.tolerance)).powf(N_POW) as u32;
        for i in (1..n).rev() {
            let t = 1.0 / i as f32;
            if c.eval(t).x > point.x {
                let (left, right) = c.split(t);
                self.cubics[idx] = left;
                self.cubics.insert(idx + 1, right);
                return self
            }
        }
        return self
    }

    #[inline(always)]
    pub fn get_cubic(&self, index: usize) -> Cubic {
        self.cubics[index]
    }

    #[inline(always)]
    pub fn set_start(&mut self, index: usize, mut pos: Vec2) -> Vec2 {
        if index != 0 {
            let c = self.cubics[index];
            let prev = &mut self.cubics[index - 1];
            pos.x = pos.x.clamp(prev.mid_0.x.max(prev.mid_1.x), c.mid_0.x.min(c.mid_1.x));
            prev.end = pos;
            self.cubics[index].start = pos;
            pos
        } else {
            let c = &mut self.cubics[index];
            pos.x = pos.x.clamp(f32::MIN, c.mid_0.x.min(c.mid_1.x));
            c.start = pos;
            pos
        }
    }

    #[inline(always)]
    pub fn set_mid_0(&mut self, index: usize, mut pos: Vec2) -> Vec2 {
        let c = &mut self.cubics[index];
        pos.x = pos.x.clamp(c.start.x, c.end.x);
        c.mid_0 = pos;
        pos
    }

    #[inline(always)]
    pub fn set_mid_1(&mut self, index: usize, mut pos: Vec2) -> Vec2 {
        let c = &mut self.cubics[index];
        pos.x = pos.x.clamp(c.start.x, c.end.x);
        c.mid_1 = pos;
        pos
    }

    #[inline(always)]
    pub fn set_end(&mut self, index: usize, mut pos: Vec2) -> Vec2 {
        if index != self.cubics.len() - 1 {
            let c = self.cubics[index];
            let next = &mut self.cubics[index + 1];
            pos.x = pos.x.clamp(c.mid_0.x.max(c.mid_1.x), next.mid_0.x.min(next.mid_1.x));
            next.start = pos;
            self.cubics[index].end = pos;
            pos
        } else {
            let c = &mut self.cubics[index];
            pos.x = pos.x.clamp(c.mid_0.x.max(c.mid_1.x), f32::MAX);
            c.end = pos;
            pos
        }
    }

    pub fn clone_from_other(&mut self, other: &Self) {
        self.cubics.clone_from_slice(&other.cubics);
    }

    pub fn iter<'a>(&'a self) -> core::slice::Iter<'a, Cubic> {
        self.cubics.iter()
    }
}

impl<'c> IntoIterator for &'c AnimationCurve {

    type Item = &'c Cubic;
    type IntoIter = core::slice::Iter<'c, Cubic>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'c> IntoIterator for &'c mut AnimationCurve {

    type Item = &'c Cubic;
    type IntoIter = core::slice::Iter<'c, Cubic>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Deref for AnimationCurve {

    type Target = GlobalVec<Cubic>;

    fn deref(&self) -> &Self::Target {
        &self.cubics
    }
}
