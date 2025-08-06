use core::slice;

use nox_mem::{Vector, vec_types::GlobalVec};

use nox_math::Vec2;

use super::*;

#[derive(Debug)]
struct Outline {
    vertices: GlobalVec<Vec2>,
}

impl Outline {
    
    #[inline(always)]
    fn new() -> Self {
        Self {
            vertices: GlobalVec::new(),
        }
    }

    #[inline(always)]
    fn insert_vertex(&mut self, vert: Vec2) {
        self.vertices.push(vert).unwrap();
    }

    #[inline(always)]
    fn is_clock_wise(&self) -> bool {
        let mut area = 0.0;
        let vertices = &self.vertices;
        let len = vertices.len();
        for i in 0..len {
            let a = vertices[i];
            let b = vertices[(i + 1) % len];
            area += a.x * b.y - b.x * a.y;
        }
        area < 0.0
    }

    #[inline(always)]
    fn join(&self, vertices: &mut GlobalVec<Vec2>) {
        vertices.append(&self.vertices).unwrap();
    }
}

struct OutlineBuilder {
    outlines: GlobalVec<Outline>,
    current_outline: Option<Outline>,
    pos: Vec2,
    vertex_count: u32,
}

#[inline(always)]
fn flatten_vertices(vertices: &[Vec2]) -> &[f32] {
    let ptr = vertices.as_ptr() as *const f32;
    unsafe {
        slice::from_raw_parts(ptr, vertices.len() * 2)
    }
}

#[inline(always)]
fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut inside = false;
    let len = polygon.len();
    for i in 0..len {
        let a  = polygon[i];
        let b = polygon[(i + 1) % len];
        if ((a.y > point.y) != (b.y > point.y)) &&
            point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y + f32::EPSILON) + a.x
        {
            inside = !inside;
        }
    }
    inside
}

impl OutlineBuilder {

    #[inline(always)]
    fn new() -> Self {
        Self {
            outlines: GlobalVec::new(),
            current_outline: Some(Outline::new()),
            pos: Default::default(),
            vertex_count: 0,
        }
    }

    #[inline(always)]
    fn insert_vertex(&mut self, vert: Vec2) {
        unsafe {
            self.current_outline
                .as_mut()
                .unwrap_unchecked()
                .insert_vertex(vert);
        }
    }

    fn finalize(self) -> Result<(GlobalVec<Vec2>, GlobalVec<u32>), earcutr::Error> {

        let mut outers = GlobalVec
            ::with_capacity(self.outlines.len())
            .unwrap();
        let mut holes = GlobalVec
            ::with_capacity(self.outlines.len())
            .unwrap();

        for outline in &self.outlines {
            if outline.is_clock_wise() {
                holes.push(outline).unwrap();
            } else {
                outers.push(outline).unwrap();
            }
        }

        let mut vertices = GlobalVec
            ::with_capacity(self.vertex_count as usize)
            .unwrap();

        let mut indices = GlobalVec
            ::with_capacity(3 * self.vertex_count as usize)
            .unwrap();

        for outer in &outers {
            let offset = vertices.len();
            let mut index_offset = outer.vertices.len();
            outer.join(&mut vertices);
            let mut hole_indices = GlobalVec::new();
            for hole in &holes {
                let p = hole.vertices[0];
                if point_in_polygon(p, &outer.vertices) {
                    hole_indices.push(index_offset).unwrap();
                    index_offset += hole.vertices.len();
                    hole.join(&mut vertices);
                }
            }
            indices
                .append_map(
                    &earcutr::earcut(flatten_vertices(&vertices[offset..vertices.len()]), &hole_indices, 2)?,
                    |v| *v as u32
                )
            .unwrap();
        }
        
        Ok((
            vertices,
            indices,
        ))
    }
}

fn flatten_quad(p0: Vec2, p1: Vec2, p2: Vec2, tolerance: f32, out: &mut Outline) {
    fn recurse(
        p0: Vec2,
        p1: Vec2,
        p2: Vec2,
        depth: u32,
        tolerance: f32,
        out: &mut Outline
    ) {
        let mid = (p0 + p1) * 0.5;
        let mag = (p1 - mid).mag();
        if depth >= 10 || mag < tolerance {
            out.insert_vertex(p2);
        } else {
            let p0p1 = (p0 + p1) * 0.5;
            let p1p2 = (p1 + p2) * 0.5;
            let p01_12 = (p0p1 + p1p2) * 0.5;
            recurse(p0, p0p1, p01_12, depth + 1, tolerance, out);
            recurse(p01_12, p1p2, p2, depth + 1, tolerance, out);
        }
    }
    recurse(p0, p1, p2, 0, tolerance, out);
}

fn flatten_cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, tolerance: f32, out: &mut Outline) {
    fn recurse(
        p0: Vec2,
        p1: Vec2,
        p2: Vec2,
        p3: Vec2,
        depth: u32,
        tolerance: f32,
        out: &mut Outline
    ) {
        let u = 3.0 * p1 - 2.0 * p0 - p3;
        let v = 3.0 * p2 - 2.0 * p3 - p0;
        let dx = u.x * u.x;
        let dy = u.y * u.y;
        let ex = v.x * v.y;
        let ey = v.y * v.y;
        if depth >= 10 || dx.max(dy).max(ex).max(ey) < tolerance * tolerance * 16.0 {
            out.insert_vertex(p3);
        } else {
            let p01 = (p0 + p1) * 0.5;
            let p12 = (p1 + p2) * 0.5;
            let p23 = (p2 + p3) * 0.5;
            let p012 = (p01 + p12) * 0.5;
            let p123 = (p12 + p23) * 0.5;
            let p0123 = (p012 + p123) * 0.5;
            recurse(p0, p01, p012, p0123, depth + 1, tolerance, out);
            recurse(p0123, p123, p23, p3, depth + 1, tolerance, out);
        }
    }

    recurse(p0, p1, p2, p3, 0, tolerance, out);
}

impl ttf_parser::OutlineBuilder for OutlineBuilder {

    fn move_to(&mut self, x: f32, y: f32) {
        self.pos = Vec2::new(x, y);
        self.insert_vertex(self.pos);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let end = Vec2::new(x, y);
        self.insert_vertex(end);
        self.pos = end;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let end = Vec2::new(x, y);
        flatten_quad(self.pos, Vec2::new(x1, y1), end, 0.1, self.current_outline.as_mut().unwrap());
        self.pos = end;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let end = Vec2::new(x, y);
        flatten_cubic(
            self.pos,
            Vec2::new(x1, y1),
            Vec2::new(x2, y2),
            end,
            0.1,
            self.current_outline.as_mut().unwrap()
        );
        self.pos = end;
    }

    fn close(&mut self) {
        let outline = self.current_outline.take().unwrap();
        self.vertex_count += outline.vertices.len() as u32;
        self.outlines
            .push(outline)
            .unwrap();
        self.current_outline = Some(Outline::new());
    }
}

pub fn triangulate(
    glyph: char,
    face: &Face
) -> Option<(GlobalVec<Vec2>, GlobalVec<u32>)>
{
    let id = face.glyph_index(glyph)?;
    let mut builder = OutlineBuilder::new();
    face.outline_glyph(id, &mut builder)?;
    Some(builder.finalize().ok()?)
}
