use nox_mem::vec_types::{Vector, GlobalVec};

use super::{structs_2d::*, fn_2d::*};

pub fn earcut(
    outer: &[[f32; 2]],
    holes: &[&[[f32; 2]]],
    clock_wise: bool
) -> Option<(GlobalVec<[f32; 2]>, GlobalVec<usize>)>
{
    let mut points = GlobalVec::<(Point, bool)>::new();
    points.append_map(outer, |&v| (v.into(), false));
    let mut holes = GlobalVec::from(holes);

    while let Some(hole) = holes.pop() {
        let mut out_idx: Option<usize> = None;
        let mut inner: Point = hole[0].into();
        let mut inner_idx = 0;
        for (i, &v) in hole.iter().enumerate() {
            if v[0] > inner.x {
                inner = v.into();
                inner_idx = i
            }
        }
        let n = points.len();
        for _ in 0..hole.len() {
            for j in 0..n {
                let point = points[j];
                if point.1 {
                    continue
                }
                let mut valid = true;
                for k in 0..n {
                    let p0 = points[k].0;
                    let p1 = points[(k + 1) % n].0;
                    if segments_cross(inner, point.0, p0, p1) {
                        valid = false;
                        break
                    }
                }
                if !valid { continue }
                for h in &holes {
                    let m = h.len();
                    for k in 0..m {
                        let p0 = h[k];
                        let p1 = h[(k + 1) % m];
                        if segments_cross(inner, point.0, p0.into(), p1.into()) {
                            valid = false;
                            break
                        }
                    }
                }
                let m = hole.len();
                for k in 0..hole.len() {
                    let p0 = hole[k];
                    let p1 = hole[(k + 1) % m];
                    if segments_cross(inner, point.0, p0.into(), p1.into()) {
                        valid = false;
                        break
                    }
                }
                if valid {
                    if let Some(out) = out_idx {
                        if points[j].0.mag(inner) < points[out].0.mag(inner) {
                            out_idx = Some(j)
                        }
                    } else {
                        out_idx = Some(j);
                    }
                }
            }
            if out_idx.is_some() {
                break
            }
            inner_idx = (inner_idx + 1) % hole.len();
            inner = hole[inner_idx].into();
        }
        let Some(out_idx) = out_idx else {
            return None
        };
        let a = points[out_idx].0;
        let mut hole_cycle = GlobalVec::from(&hole[inner_idx..]);
        hole_cycle.append(&hole[..inner_idx]);
        for &p in hole_cycle.iter().rev() {
            points.insert(out_idx + 1, (p.into(), true));
        }
        points.insert(out_idx + 1 + hole_cycle.len(), (a, true));
        points.insert(out_idx + 1 + hole_cycle.len(), (inner, true));
    }

    let mut indices = GlobalVec::with_capacity(points.len());
    let mut idx = GlobalVec::with_capacity(points.len());

    for i in 0..points.len() {
        idx.push(i);
    }

    let winding = if clock_wise { -1.0 } else { 1.0 };

    while idx.len() > 3 {
        let n = idx.len();
        let mut ok = false;
        for i in 0..n {
            let prev = (i + n - 1) % n;
            let next = (i + 1) % n;
            let a = points[idx[prev]].0;
            let b = points[idx[i]].0;
            let c = points[idx[next]].0;
            if orient(a, b, c) * winding < -1e-6  { continue }
            ok = true;
            for k in 0..n {
                if k == prev || k == i || k == next { continue }
                let kp = points[idx[k]].0;
                if kp == a || kp == b || kp == c { continue }
                if point_in_triangle(a, b, c, points[idx[k]].0.into()) { ok = false; break }
            }
            if ok {
                let prev = (i + n - 1) % n;
                let next = (i + 1) % n;
                let (a, b, c) = (idx[prev], idx[i], idx[next]);
                indices.append(&[a, b, c]);
                idx.remove(i);
                break
            }
        }
        if !ok {
            return None
        }
    }

    if idx.len() == 3 {
        indices.append(&[idx[0], idx[1], idx[2]]);
    }

    if clock_wise {
        for i in 0..indices.len() / 3 {
            let i = i * 3;
            indices[i] = indices[i + 2] ^ indices[i];
            indices[i + 2] = indices[i] ^ indices[i + 2];
            indices[i] = indices[i + 2] ^ indices[i];
        }
    }

    let mut vertices = GlobalVec::new();
    vertices.append_map(&points, |v| v.0.into());

    Some((vertices, indices))
}
