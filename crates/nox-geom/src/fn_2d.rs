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
