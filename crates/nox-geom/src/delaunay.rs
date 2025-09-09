use core::hash::Hash;

use rustc_hash::FxHashMap;

use nox_mem::vec_types::{GlobalVec, Vector};
use crate::{fn_2d::*, structs_2d::*};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct EdgeHash {
    a: usize,
    b: usize,
}

fn edge_hash(a: usize, b: usize) -> EdgeHash {
    if a > b { EdgeHash { a, b, } }
    else { EdgeHash { a: b, b: a, } }
}

#[derive(Clone, Copy, Debug)]
struct Triangle {
    a: usize,
    b: usize,
    c: usize,
}

fn triangle(a: usize, b: usize, c: usize) -> Triangle {
    Triangle { a, b, c }
}

impl Triangle {

    fn edges(self) -> [(usize, usize); 3] {
        [
            (self.a, self.b),
            (self.b, self.c),
            (self.c, self.a),
        ]
    }

    fn points(self) -> [usize; 3] {
        [
            self.a,
            self.b,
            self.c,
        ]
    }

    fn orient(self, points: &[Point]) -> Self {
        let (a, b, c) = (points[self.a], points[self.b], points[self.c]);
        if orient(a, b, c) < 0.0 {
            self
        } else {
            Self { a: self.b, b: self.a, c: self.c, }
        }
    }
}

pub fn delaunay(points: &[[f32; 2]]) -> Option<GlobalVec<usize>> {
    let mut pnt = GlobalVec::<Point>::with_capacity(points.len());
    pnt.append_map(points, |&v| v.into());
    let mut points = pnt;
    let n_points = points.len();
    let big = 1.0e6;
    points.append(&[
        point(-big, -big),
        point(big, -big),
        point(0.0, big),
    ]);
    let mut triangles = GlobalVec::new();
    triangles.push(triangle(n_points, n_points + 1, n_points + 2));
    let mut edge_counts = FxHashMap::<EdgeHash, usize>::default();
    for i in 0..n_points {
        let p: Point = points[i].into();
        let mut j = 0;
        while j < triangles.len() {
            let tri = triangles[j];
            let (a, b, c) = (points[tri.a], points[tri.b], points[tri.c]);
            if point_in_circumcircle(a.into(), b.into(), c.into(), p) {
                let tri = triangles.swap_remove(j).unwrap();
                for (a, b) in tri.edges() {
                    let hash = edge_hash(a, b);
                    *edge_counts.entry(hash).or_default() += 1;
                }
            } else {
                j += 1;
            }
        }
        for (&edge, _) in edge_counts.iter().filter(|(_, c)| **c == 1) {
            triangles.push(triangle(edge.a, edge.b, i).orient(&points));
        }
        edge_counts.clear();
    }
    triangles.retain(|t| {
        !t.points().iter().any(|&p| p == n_points || p == n_points + 1 || p == n_points + 2)
    });
    let mut indices = GlobalVec::new();
    for triangle in &triangles {
        let mut points = triangle.points();
        points.reverse();
        indices.append(&points);
    }
    if indices.len() == 0 {
        return None
    }
    Some(indices)
}
