use super::*;

#[inline(always)]
pub fn orient(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

pub fn point_in_polygon<T: Copy + Into<Vec2>>(p: T, poly: &[T]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let p: Vec2 = p.into();
    for i in 0..n {
        let j = (i + n - 1) % n;
        let pi: Vec2 = poly[i].into();
        let pj: Vec2 = poly[j].into();
        if ((pi.y > p.y) != (pj.y > p.y)) &&
            (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x) {
            inside = !inside;
        }
    }
    inside
}

#[inline(always)]
pub fn point_in_triangle(a: Vec2, b: Vec2, c: Vec2, p: Vec2) -> bool {
    let abp = orient(a, b, p);
    let bcp = orient(b, c, p);
    let cap = orient(c, a, p);
    (abp >= 0.0 && bcp >= 0.0 && cap >= 0.0) || (abp <= 0.0 && bcp <= 0.0 && cap <= 0.0)
}

#[inline(always)]
pub fn segments_cross(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);

    o1 * o2 < 0.0 && o3 * o4 < 0.0
}

#[inline(always)]
pub fn line_intersection(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Option<Vec2> {
    let d1 = b - a;
    let d2 = d - c;
    let d1xd2 = d1.cross(d2);
    if d1xd2.abs() < f32::EPSILON {
        return None;
    }
    let d3 = c - a;
    let d3xd2 = d3.cross(d2);
    let t = d3xd2 / d1xd2;
    Some(a + t * d1)
}

#[inline(always)]
pub fn point_in_circumcircle(a: Vec2, b: Vec2, c: Vec2, p: Vec2) -> bool {
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
