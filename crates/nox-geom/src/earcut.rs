use nox_mem::{
    vec_types::{Vector, GlobalVec},
    CapacityError,
};

use super::{*, fn_2d::*};

#[derive(Clone, Copy)]
pub struct EarcutHole<'a> {
    points: &'a [[f32; 2]],
    choose_right_most_index: bool,
}

pub fn earcut_hole<'a>(points: &'a[[f32; 2]], choose_right_most_index: bool) -> EarcutHole<'a> {
    EarcutHole { points, choose_right_most_index }
}

pub fn earcut<P>(
    outline: &[[f32; 2]],
    holes: &[EarcutHole],
    clock_wise: bool,
    out_vertices: &mut impl Vector<P>,
    out_indices: &mut impl Vector<usize>,
) -> Result<bool, CapacityError>
    where
        P: From<[f32; 2]>,
{
    let mut points = GlobalVec::<(Vec2, bool)>::new();
    points.append_map(outline, |&v| (v.into(), false));
    let mut holes = GlobalVec::from(holes);

    while let Some(hole) = holes.pop() {
        let mut out_idx: Option<usize> = None;
        if hole.points.len() == 0 {
            continue
        }
        let mut inner: Vec2 = hole.points[0].into();
        let mut inner_idx = 0;
        for (i, &v) in hole.points.iter().enumerate() {
            if v[0] > inner.x {
                inner = v.into();
                inner_idx = i
            }
        }
        let n = points.len();
        for _ in 0..hole.points.len() {
            for j in 0..n {
                let point = points[j];
                if point.1 {
                    continue
                }
                let mut valid = true;
                for k in 0..n {
                    let p0 = unsafe { points.get_unchecked(k).0 };
                    let p1 = unsafe { points.get_unchecked((k + 1) % n).0 };
                    if segments_cross(inner, point.0, p0, p1) {
                        valid = false;
                        break
                    }
                }
                if !valid { continue }
                for h in &holes {
                    let m = h.points.len();
                    for k in 0..m {
                        let p0 = unsafe { *h.points.get_unchecked(k) };
                        let p1 = unsafe { *h.points.get_unchecked((k + 1) % m) };
                        if segments_cross(inner, point.0, p0.into(), p1.into()) {
                            valid = false;
                            break
                        }
                    }
                }
                let m = hole.points.len();
                for k in 0..hole.points.len() {
                    let p0 = unsafe { *hole.points.get_unchecked(k) };
                    let p1 = unsafe { *hole.points.get_unchecked((k + 1) % m) };
                    if segments_cross(inner, point.0, p0.into(), p1.into()) {
                        valid = false;
                        break
                    }
                }
                if valid {
                    if let Some(out) = out_idx {
                        if !hole.choose_right_most_index {
                            if points[j].0.mag_to(inner) < points[out].0.mag_to(inner) {
                                out_idx = Some(j)
                            }
                        } else {
                            out_idx = Some(j);
                            break
                        }
                    } else {
                        out_idx = Some(j);
                    }
                }
            }
            if out_idx.is_some() {
                break
            }
            inner_idx = (inner_idx + 1) % hole.points.len();
            inner = unsafe { *hole.points.get_unchecked(inner_idx) }.into();
        }
        let Some(out_idx) = out_idx else {
            return Ok(false)
        };
        let a = points[out_idx].0;
        let mut hole_cycle = GlobalVec::from(&hole.points[inner_idx..]);
        hole_cycle.append(&hole.points[..inner_idx]);
        for &p in hole_cycle.iter().rev() {
            points.insert(out_idx + 1, (p.into(), true));
        }
        points.insert(out_idx + 1 + hole_cycle.len(), (a, true));
        points.insert(out_idx + 1 + hole_cycle.len(), (inner, true));
    }

    let vertex_off = out_vertices.len();
    let index_off = out_indices.len();
    
    out_indices.reserve(points.len())?;
    let mut idx = GlobalVec::with_capacity(points.len());

    let winding = if clock_wise { -1.0 } else { 1.0 };

    let n_points = points.len();
    for i in 0..n_points {
        idx.push(i);
    }

    let mut n_idx = 0;

    while idx.len() > 3 {
        let n = idx.len();
        let mut ok = false;
        for i in 0..n {
            let prev = (i + n - 1) % n;
            let next = (i + 1) % n;
            let (a, b, c) = unsafe {(
                points.get_unchecked(*idx.get_unchecked(prev)).0,
                points.get_unchecked(*idx.get_unchecked(i)).0,
                points.get_unchecked(*idx.get_unchecked(next)).0
            )};
            if orient(a, b, c) * winding < 0.0  { continue }
            ok = true;
            let bb0 =  vec2(a.x.min(b.x.min(c.x)), a.y.min(b.y.min(c.y)));
            let bb1 =  vec2(a.x.max(b.x.max(c.x)), a.y.max(b.y.max(c.y)));
            for j in 0..n {
                let j = (j + next + 1) % n;
                if j == prev { break }
                let p = unsafe { points.get_unchecked(*idx.get_unchecked(j)).0 };
                if p == a || p == b || p == c { continue }
                if  p.x >= bb0.x &&
                    p.y >= bb0.y &&
                    p.x <= bb1.x &&
                    p.y <= bb1.y &&
                    !(a.x == p.x && a.y == p.y) &&
                    point_in_triangle(a, b, c, p)
                {
                    ok = false;
                    break
                }
            }
            if ok {
                let (a, b, c) = unsafe {(
                    *idx.get_unchecked(prev),
                    *idx.get_unchecked(i),
                    *idx.get_unchecked(next)
                )};
                out_indices.append(&[a + vertex_off, b + vertex_off, c + vertex_off])?;
                n_idx += 3;
                idx.remove(i);
                break
            }
        }
        if !ok {
            return Ok(false)
        }
    }

    if idx.len() == 3 {
        out_indices.append(&[idx[0] + vertex_off, idx[1] + vertex_off, idx[2] + vertex_off])?;
        n_idx += 3;
    }

    if clock_wise {
        for i in 0..n_idx / 3 {
            let i = index_off + i * 3;
            out_indices[i] = out_indices[i + 2] ^ out_indices[i];
            out_indices[i + 2] = out_indices[i] ^ out_indices[i + 2];
            out_indices[i] = out_indices[i + 2] ^ out_indices[i];
        }
    }

    out_vertices.append_map(&points, |v| <Vec2 as Into<[f32; 2]>>::into(v.0).into())?;

    Ok(true)
}
