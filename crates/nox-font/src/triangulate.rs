use nox::mem::{
    vec_types::{GlobalVec, Vector},
};

use nox_geom::{earcut, fn_2d::*};

use super::*;

#[derive(Debug)]
struct Outline {
    vertices: GlobalVec<[f32; 2]>,
    units_per_em: f32,
}

impl Outline {
    
    #[inline(always)]
    fn new(units_per_em: f32) -> Self {
        Self {
            vertices: GlobalVec::new(),
            units_per_em,
        }
    }

    #[inline(always)]
    fn insert_vertex(&mut self, mut vert: [f32; 2]) {
        vert[0] /= self.units_per_em;
        vert[1] /= self.units_per_em;
        vert[1] = 1.0 - vert[1];
        self.vertices.push(vert);
    }

    #[inline(always)]
    fn is_hole(&self, winding_rule: f32) -> bool {
        let mut area = 0.0;
        let vertices = &self.vertices;
        let len = vertices.len();
        for i in 0..len {
            let a = vertices[i];
            let b = vertices[(i + 1) % len];
            area += a[0] * b[1] - b[0] * a[1];
        }
        area * winding_rule < 0.0
    }
}

struct OutlineBuilder {
    outlines: GlobalVec<Outline>,
    current_outline: Option<Outline>,
    curve_depth: u32,
    pos: [f32; 2],
    vertex_count: u32,
    winding_rule: i16,
    units_per_em: u16,
}

impl OutlineBuilder {

    #[inline(always)]
    fn new(curve_depth: u32, units_per_em: u16, winding_rule: i16) -> Self {
        Self {
            outlines: GlobalVec::new(),
            current_outline: Some(Outline::new(units_per_em as f32)),
            curve_depth,
            pos: Default::default(),
            vertex_count: 0,
            units_per_em,
            winding_rule,
        }
    }

    #[inline(always)]
    fn insert_vertex(&mut self, vert: [f32; 2]) {
        unsafe {
            self.current_outline
                .as_mut()
                .unwrap_unchecked()
                .insert_vertex(vert);
        }
    }

    #[inline(always)]
    fn finalize(self) -> Option<GlyphTriangles> {

        if self.vertex_count == 0 {
            println!("white space");
            return None
        }

        let mut vertices = GlobalVec::new();
        let mut indices = GlobalVec::new();

        let winding_rule = self.winding_rule as f32;
        let clock_wise = if self.winding_rule < 0 { true } else { false };

        let outlines = &self.outlines;

        for outline in outlines {

            if outline.is_hole(winding_rule) {
                continue
            }

            let outer = &outline.vertices;

            let mut holes = GlobalVec::new();

            for o in outlines {
                if o.is_hole(winding_rule) &&
                    point_in_polygon(o.vertices[0].into(), outer) {
                    holes.push(o.vertices.as_slice());
                }
            }

            let (vert, ind) = earcut(&outer, &holes, clock_wise)?;

            let index_off = vertices.len();
            indices.append_map(&ind, |&v| (v + index_off) as u32);
            vertices.append_map(&vert, |&v| Vertex { pos: v });

        }
        Some(GlyphTriangles {
            vertices,
            indices,
        })
    }
}

fn mag(v: [f32; 2]) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn mul(a: [f32; 2], s: f32) -> [f32; 2] {
    [a[0] * s, a[1] * s]
}

fn flatten_quad(
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    tolerance: f32,
    curve_depth: u32,
    out: &mut Outline,
)
{
    fn recurse(
        p0: [f32; 2],
        p1: [f32; 2],
        p2: [f32; 2],
        depth: u32,
        tolerance: f32,
        curve_depth: u32,
        out: &mut Outline
    ) {
        let mid = mul(add(p0, p1), 0.5);
        let mag = mag(sub(p1, mid));
        if depth >= curve_depth || mag < tolerance {
            out.insert_vertex(p2);
        } else {
            let p0p1 = mul(add(p0, p1), 0.5);
            let p1p2 = mul(add(p1, p2), 0.5);
            let p01_12 = mul(add(p0p1, p1p2),0.5);
            recurse(p0, p0p1, p01_12, depth + 1, tolerance, curve_depth, out);
            recurse(p01_12, p1p2, p2, depth + 1, tolerance, curve_depth, out);
        }
    }
    recurse(p0, p1, p2, 0, tolerance, curve_depth, out);
}

fn flatten_cubic(
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    tolerance: f32,
    curve_depth: u32,
    out: &mut Outline,
)
{
    fn recurse(
        p0: [f32; 2],
        p1: [f32; 2],
        p2: [f32; 2],
        p3: [f32; 2],
        depth: u32,
        tolerance: f32,
        curve_depth: u32,
        out: &mut Outline
    ) {
        let u = sub(sub(mul(p1, 3.0), mul(p0, 2.0)), p3);
        let v = sub(sub(mul(p2, 3.0), mul(p3, 2.0)), p0);
        let dx = u[0] * u[0];
        let dy = u[1] * u[1];
        let ex = v[0] * v[0];
        let ey = v[1] * v[1];
        if depth >= curve_depth || dx.max(dy).max(ex).max(ey) < tolerance * tolerance * 16.0 {
            out.insert_vertex(p3);
        } else {
            let p01 = mul(add(p0, p1), 0.5);
            let p12 = mul(add(p1, p2), 0.5);
            let p23 = mul(add(p2, p3), 0.5);
            let p012 = mul(add(p01, p12), 0.5);
            let p123 = mul(add(p12, p23), 0.5);
            let p0123 = mul(add(p012, p123), 0.5);
            recurse(p0, p01, p012, p0123, depth + 1, tolerance, curve_depth, out);
            recurse(p0123, p123, p23, p3, depth + 1, tolerance, curve_depth, out);
        }
    }

    recurse(p0, p1, p2, p3, 0, tolerance, curve_depth, out);
}

impl ttf_parser::OutlineBuilder for OutlineBuilder {

    fn move_to(&mut self, x: f32, y: f32) {
        self.pos = [x, y];
        self.insert_vertex(self.pos);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let end = [x, y];
        self.insert_vertex(end);
        self.pos = end;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let end = [x, y];
        flatten_quad(
            self.pos,
            [x1, y1],
            end,
            0.1,
            self.curve_depth,
            self.current_outline.as_mut().unwrap()
        );
        self.pos = end;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let end = [x, y];
        flatten_cubic(
            self.pos,
            [x1, y1],
            [x2, y2],
            end,
            0.1,
            self.curve_depth,
            self.current_outline.as_mut().unwrap()
        );
        self.pos = end;
    }

    fn close(&mut self) {
        let outline = self.current_outline.take().unwrap();
        self.vertex_count += outline.vertices.len() as u32;
        self.outlines
            .push(outline);
        self.current_outline = Some(Outline::new(self.units_per_em as f32));
    }
}

#[derive(Default, Clone)]
pub struct GlyphTriangles {
    pub vertices: GlobalVec<Vertex>,
    pub indices: GlobalVec<u32>,
}

pub fn triangulate(
    glyph: char,
    face: &Face,
    curve_depth: u32,
) -> Option<GlyphTriangles>
{
    let id = face.glyph_index(glyph)?;
    let mut winding_rule = None;
    if face.tables().glyf.is_some() {
        winding_rule = Some(1);
    }
    if face.tables().cff.is_some() || face.tables().cff2.is_some() {
        winding_rule = Some(-1);
    }
    let mut builder = OutlineBuilder::new(curve_depth, face.units_per_em(), winding_rule?);
    face.outline_glyph(id, &mut builder)?;
    builder.finalize()
}
