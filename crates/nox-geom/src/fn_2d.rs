use super::structs_2d::*;

#[inline(always)]
pub fn orient(a: Point, b: Point, c: Point) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

pub fn point_in_polygon<T: Copy + Into<Point>>(p: T, poly: &[T]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let p: Point = p.into();
    for i in 0..n {
        let j = (i + n - 1) % n;
        let pi: Point = poly[i].into();
        let pj: Point = poly[j].into();
        if ((pi.y > p.y) != (pj.y > p.y)) &&
            (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x) {
            inside = !inside;
        }
    }
    inside
}

#[inline(always)]
pub fn point_in_triangle(a: Point, b: Point, c: Point, p: Point) -> bool {
    let abp = orient(a, b, p);
    let bcp = orient(b, c, p);
    let cap = orient(c, a, p);
    (abp >= 0.0 && bcp >= 0.0 && cap >= 0.0) || (abp <= 0.0 && bcp <= 0.0 && cap <= 0.0)
}

#[inline(always)]
pub fn segments_cross(a: Point, b: Point, c: Point, d: Point) -> bool {
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);

    o1 * o2 < 0.0 && o3 * o4 < 0.0
}

#[inline(always)]
pub fn point_in_circumcircle(a: Point, b: Point, c: Point, p: Point) -> bool {
    let ap = a - p;
    let bp = b - p;
    let cp = c - p;

    let apd = ap.dot(ap);
    let bpd = bp.dot(bp);
    let cpd = cp.dot(cp);

    let det =   ap.x * (bp.y * cpd - bpd * cp.y) -
                ap.y * (bp.x * cpd - bpd * cp.x) +
                apd  * (bp.x * cp.y - cp.x * bp.y);

    if orient(a, b, c) > 0.0 {
        det > 0.0
    } else {
        det < 0.0
    }
}

#[inline(always)]
pub fn flatten_quad<F: FnMut(Point)>(
    p0: Point,
    p1: Point,
    p2: Point,
    tolerance: f32,
    curve_depth: u32,
    collect: &mut F,
)
{
    fn recurse<F: FnMut(Point)>(
        p0: Point,
        p1: Point,
        p2: Point,
        depth: u32,
        tolerance: f32,
        curve_depth: u32,
        collect: &mut F,
    ) {
        let mid = (p0 + p1) * 0.5;
        let mag = p1.mag_to(mid);
        if depth >= curve_depth || mag < tolerance {
            collect(p2);
        } else {
            let p0p1 = (p0 + p1) * 0.5;
            let p1p2 = (p1 + p2) * 0.5;
            let p01_12 = (p0p1 + p1p2) * 0.5;
            recurse(p0, p0p1, p01_12, depth + 1, tolerance, curve_depth, collect);
            recurse(p01_12, p1p2, p2, depth + 1, tolerance, curve_depth, collect);
        }
    }
    recurse(p0.into(), p1.into(), p2.into(), 0, tolerance, curve_depth, collect);
}

#[inline(always)]
pub fn flatten_cubic<F: FnMut(Point)>(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    tolerance: f32,
    curve_depth: u32,
    collect: &mut F,
)
{
    fn recurse<F: FnMut(Point)>(
        p0: Point,
        p1: Point,
        p2: Point,
        p3: Point,
        depth: u32,
        tolerance: f32,
        curve_depth: u32,
        collect: &mut F,
    ) {
        let u = p1 * 3.0 - p0 * 2.0 - p3;
        let v = p2 * 3.0 - p3 * 2.0 - p0;
        let dx = u.x * u.x;
        let dy = u.y * u.y;
        let ex = v.x * v.x;
        let ey = v.y * v.y;
        if depth >= curve_depth || dx.max(dy).max(ex).max(ey) < tolerance * tolerance * 16.0 {
            collect(p3)
        } else {
            let p01 = (p0 + p1) * 0.5;
            let p12 = (p1 + p2) * 0.5;
            let p23 = (p2 + p3) * 0.5;
            let p012 = (p01 + p12) * 0.5;
            let p123 = (p12 + p23) * 0.5;
            let p0123 = (p012 + p123) * 0.5;
            recurse(p0, p01, p012, p0123, depth + 1, tolerance, curve_depth, collect);
            recurse(p0123, p123, p23, p3, depth + 1, tolerance, curve_depth, collect);
        }
    }

    recurse(p0.into(), p1.into(), p2.into(), p3.into(), 0, tolerance, curve_depth, collect);
}
