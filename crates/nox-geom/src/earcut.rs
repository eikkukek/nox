use nox_mem::vec_types::{Vector, GlobalVec};

use super::{*, fn_2d::*};

#[derive(Clone, Copy)]
pub struct EarcutHole<'a> {
    points: &'a [[f32; 2]],
    choose_right_most_index: bool,
}

pub fn earcut_hole(points: &[[f32; 2]], choose_right_most_index: bool) -> EarcutHole {
    EarcutHole { points, choose_right_most_index }
}

pub fn earcut(
    outline: &[[f32; 2]],
    holes: &[EarcutHole],
    clock_wise: bool,
) -> Option<(GlobalVec<[f32; 2]>, GlobalVec<usize>)>
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
            return None
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

    let mut indices = GlobalVec::with_capacity(points.len());
    let mut idx = GlobalVec::with_capacity(points.len());

    let winding = if clock_wise { -1.0 } else { 1.0 };

    let n_points = points.len();
    for i in 0..n_points {
        idx.push(i);
    }

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
            if orient(a, b, c) * winding < -1e-6  { continue }
            ok = true;
            for j in 0..n {
                let kp = unsafe { points.get_unchecked(*idx.get_unchecked(j)).0 };
                if kp == a || kp == b || kp == c { continue }
                if point_in_triangle(a, b, c, kp) { ok = false; break }
            }
            if ok {
                let prev = (i + n - 1) % n;
                let next = (i + 1) % n;
                let (a, b, c) = unsafe {(
                    *idx.get_unchecked(prev),
                    *idx.get_unchecked(i),
                    *idx.get_unchecked(next)
                )};
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
