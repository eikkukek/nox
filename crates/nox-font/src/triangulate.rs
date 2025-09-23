use nox::mem::{
    vec_types::{GlobalVec, Vector},
};

use nox_geom::{
    earcut::{earcut, earcut_hole},
    bezier::{quad, cubic},
    fn_2d::*,
    vec2,
};

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
    curve_tolerance: f32,
    pos: [f32; 2],
    vertex_count: u32,
    winding_rule: i16,
    units_per_em: u16,
}

impl OutlineBuilder {

    #[inline(always)]
    fn new(curve_tolerance: f32, units_per_em: u16, winding_rule: i16) -> Self {
        Self {
            outlines: GlobalVec::new(),
            current_outline: Some(Outline::new(units_per_em as f32)),
            curve_tolerance,
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
            return None
        }

        let mut vertices = GlobalVec::new();
        let mut ind = GlobalVec::new();

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
                    holes.push(earcut_hole(o.vertices.as_slice(), false));
                }
            }

            earcut(&outer, &holes, clock_wise, &mut vertices, &mut ind).unwrap();
        }

        let mut indices = GlobalVec::new();
        indices.append_map(&ind, |&v| v as u32);

        if vertices.len() == 0 {
            return None
        }

        Some(GlyphTriangles {
            vertices,
            indices,
        })
    }
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
        let outline = self.current_outline.as_mut().unwrap();
        quad(self.pos.into(), vec2(x1, y1), end.into())
            .flatten(self.curve_tolerance, &mut |p| outline.insert_vertex(p.into()));
        self.pos = end;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let end = [x, y];
        let outline = self.current_outline.as_mut().unwrap();
        cubic(self.pos.into(), vec2(x1, y1), vec2(x2, y2), end.into())
            .flatten(self.curve_tolerance, &mut |p| outline.insert_vertex(p.into()));
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
    curve_tolerance: f32,
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
    let mut builder = OutlineBuilder::new(curve_tolerance, face.units_per_em(), winding_rule?);
    face.outline_glyph(id, &mut builder)?;
    builder.finalize()
}
