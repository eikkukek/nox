use nox::mem::vec_types::{GlobalVec, Vector};

use nox_geom::{
    shapes::*,
    *,
};

use crate::*;

pub struct Stroke {
    pub col: ColorSRGBA,
    pub thickness: f32,
}

pub struct PainterStorage {
    vertices: GlobalVec<Vertex>,
    indices_usize: GlobalVec<usize>,
    points: GlobalVec<[f32; 2]>,
    helper_points: GlobalVec<[f32; 2]>,
}

impl PainterStorage {

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            vertices: Default::default(),
            indices_usize: Default::default(),
            points: Default::default(),
            helper_points: Default::default(),
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices_usize.clear();
    }

    pub fn get_vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn get_indices(&self) -> &[usize] {
        &self.indices_usize
    }
}

pub struct Painter<'a> {
    storage: &'a mut PainterStorage,
}

impl<'a> Painter<'a>
{

    #[inline(always)]
    pub fn new(
        storage: &'a mut PainterStorage,
    ) -> Self {
        Self {
            storage,
        }
    }

    pub fn rect(
        &mut self,
        rect: Rect,
        fill_col: ColorSRGBA,
        outline: Stroke,
    ) -> &mut Self {
        let points = &mut self.storage.points;
        let helper_points = &mut self.storage.helper_points;
        let vertices = &mut self.storage.vertices;
        let indices_usize = &mut self.storage.indices_usize;
        rect.to_points(&mut |p| { points.push(p.into()); });
        outline_points(
            points,
            outline.thickness,
            false,
            &mut |p| { helper_points.push(p.into()); }
        );
        let mut vertex_off = vertices.len();
        earcut::earcut(&helper_points, &[], false, vertices, indices_usize).ok();
        let outline_vertex_range = VertexRange::new(vertex_off..vertices.len());
        vertex_off = vertices.len();
        earcut::earcut(&points, &[], false, vertices, indices_usize).ok();
        let base_vertex_range = VertexRange::new(vertex_off..vertices.len());
        color_vertices(vertices, outline_vertex_range, outline.col);
        color_vertices(vertices, base_vertex_range, fill_col);
        self.storage.points.clear();
        self.storage.helper_points.clear();
        self
    }

    pub fn circle(
        &mut self,
        circle: Circle,
        steps: u32,
        fill_col: ColorSRGBA,
        outline: Stroke
    ) -> &mut Self {
        let points = &mut self.storage.points;
        let helper_points = &mut self.storage.helper_points;
        let vertices = &mut self.storage.vertices;
        let indices_usize = &mut self.storage.indices_usize;
        circle.to_points(steps, &mut |p| { points.push(p.into()); });
        outline_points(
            points,
            outline.thickness,
            false,
            &mut |p| { helper_points.push(p.into()); }
        );
        let mut vertex_off = vertices.len();
        earcut::earcut(&helper_points, &[], false, vertices, indices_usize).ok();
        let outline_vertex_range = VertexRange::new(vertex_off..vertices.len());
        vertex_off = vertices.len();
        earcut::earcut(&points, &[], false, vertices, indices_usize).ok();
        let base_vertex_range = VertexRange::new(vertex_off..vertices.len());
        color_vertices(vertices, outline_vertex_range, outline.col);
        color_vertices(vertices, base_vertex_range, fill_col);
        self.storage.points.clear();
        self.storage.helper_points.clear();
        self
    }
}
