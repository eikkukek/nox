use rustc_hash::{FxHashMap, FxHashSet};

use nox::mem::vec_types::{ArrayVec, GlobalVec, Vector};

use nox_geom::{
    shapes::*,
    *,
};

use crate::*;

#[derive(Default, Clone, Copy)]
pub struct Stroke {
    pub col: ColorSRGBA,
    pub thickness: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Rect(Rect),
    Circle(Circle, u32),
    Checkmark(f32),
}

#[derive(Clone)]
struct ShapeParams {
    shape: Shape,
    offset: Vec2,
    fill_col: ColorSRGBA,
    shape_vertex_range: Option<VertexRange>,
    strokes: ArrayVec<(Stroke, Option<VertexRange>), 4>,
    stroke_idx: u32,
}

impl ShapeParams {

    #[inline(always)]
    fn new_rect(
        rect: Rect,
        offset: Vec2,
        fill_col: ColorSRGBA,
        strokes: ArrayVec<Stroke, 4>,
        stroke_idx: u32,
    ) -> Self {
        Self {
            shape: Shape::Rect(rect),
            offset,
            fill_col,
            shape_vertex_range: None,
            strokes: strokes.mapped(|&v| (v, None)),
            stroke_idx
        }
    }

    #[inline(always)]
    fn new_circle(
        circle: Circle,
        steps: u32,
        offset: Vec2,
        fill_col: ColorSRGBA,
        strokes: ArrayVec<Stroke, 4>,
        stroke_idx: u32,
    ) -> Self {
        Self {
            shape: Shape::Circle(circle, steps),
            offset,
            fill_col,
            shape_vertex_range: None,
            strokes: strokes.mapped(|&v| (v, None)),
            stroke_idx,
        }
    }

    #[inline(always)]
    fn new_checkmark(
        scale: f32,
        offset: Vec2,
        fill_col: ColorSRGBA,
        strokes: ArrayVec<Stroke, 4>,
        stroke_idx: u32,
    ) -> Self {
        Self {
            shape: Shape::Checkmark(scale),
            offset,
            fill_col,
            shape_vertex_range: None,
            strokes: strokes.mapped(|&v| (v, None)),
            stroke_idx,
        }
    }
}

#[derive(Default)]
struct ReactionShapes {
    shapes: GlobalVec<ShapeParams>,
    rendered_shapes: GlobalVec<ShapeParams>,
    prev_shapes: GlobalVec<(Shape, ArrayVec<f32, 4>)>,
}

impl ReactionShapes {

    #[inline(always)]
    fn changed(&mut self) -> bool {
        self.shapes.len() != self.rendered_shapes.len() ||
        self.shapes
            .iter()
            .map(|v| (v.shape, v.strokes.mapped(|&v| v.0.thickness)))
            .ne(self.prev_shapes.iter().cloned())
    }

    #[inline(always)]
    fn hide(&self, vertices: &mut [Vertex]) {
        for params in &self.rendered_shapes {
            hide_vertices(vertices, params.shape_vertex_range);
            for &(_, range) in &params.strokes {
                hide_vertices(vertices, range);
            }
        }
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.prev_shapes.clear();
        self.prev_shapes.append_map(&self.shapes, |v| (v.shape, v.strokes.mapped(|&v| v.0.thickness)));
        if self.rendered_shapes.len() == self.shapes.len() {
            for (i, shape) in self.rendered_shapes.iter_mut().enumerate() {
                let update = self.shapes[i].clone();
                shape.offset = update.offset;
                shape.fill_col = update.fill_col;
                for (j, stroke) in update.strokes.iter().enumerate() {
                    shape.strokes[j].0 = stroke.0;
                }
                shape.stroke_idx = update.stroke_idx;
            }
        }
        self.shapes.clear();
    }
}

pub struct PainterStorage {
    vertices: GlobalVec<Vertex>,
    indices_usize: GlobalVec<usize>,
    points: GlobalVec<[f32; 2]>,
    checkmark_points: GlobalVec<[f32; 2]>,
    helper_points: GlobalVec<[f32; 2]>,
    reaction_shapes: FxHashMap<ReactionId, ReactionShapes>,
    active_reactions: FxHashSet<ReactionId>,
    prev_active_reactions: GlobalVec<ReactionId>,
    shapes: GlobalVec<(ReactionId, ShapeParams)>,
}

impl PainterStorage {

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            vertices: Default::default(),
            indices_usize: Default::default(),
            points: Default::default(),
            checkmark_points: Default::default(),
            helper_points: Default::default(),
            reaction_shapes: FxHashMap::default(),
            active_reactions: FxHashSet::default(),
            prev_active_reactions: Default::default(),
            shapes: Default::default(),
        }
    }

    pub fn begin(&mut self) {
        self.prev_active_reactions.clear();
        for &id in &self.active_reactions {
            self.reaction_shapes.get_mut(&id).unwrap().reset();
            self.prev_active_reactions.push(id);
        }
        self.active_reactions.clear();
    }

    pub fn triangulate(&mut self) -> (&[Vertex], &[usize])
    {
        let mut requires_triangulation = false;
        for &id in &self.active_reactions {
            let reaction_shapes = self.reaction_shapes.get_mut(&id).unwrap();
            if !requires_triangulation && reaction_shapes.changed() {
                requires_triangulation = true;
            }
            for shape in reaction_shapes.rendered_shapes.iter().cloned() {
                self.shapes.push((id, shape));
            }
        }
        let vertices = &mut self.vertices;
        let indices_usize = &mut self.indices_usize;
        if requires_triangulation {
            println!("here");
            vertices.clear();
            indices_usize.clear();
            self.shapes.clear();
            for shapes in &mut self.reaction_shapes {
                shapes.1.rendered_shapes.clear();
            }
            vertices.clear();
            indices_usize.clear();
            let points = &mut self.points;
            let helper_points = &mut self.helper_points;
            for id in self.active_reactions.iter() {
                let reaction_shapes = self.reaction_shapes.get_mut(&id).unwrap();
                for shape in &mut reaction_shapes.shapes {
                    match shape.shape {
                        Shape::Rect(rect) => {
                            rect.to_points(&mut |p| { points.push(p.into()); });
                            for (stroke, range) in &mut shape.strokes {
                                outline_points(
                                    points,
                                    stroke.thickness,
                                    false,
                                    &mut |p| { helper_points.push(p.into()); }
                                );
                                let vertex_off = vertices.len();
                                earcut::earcut(&helper_points, &[], false, vertices, indices_usize).ok();
                                *range = VertexRange::new(vertex_off..vertices.len());
                                helper_points.clear();
                            }
                            let vertex_off = vertices.len();
                            earcut::earcut(&points, &[], false, vertices, indices_usize).ok();
                            shape.shape_vertex_range = VertexRange::new(vertex_off..vertices.len());
                        },
                        Shape::Circle(circle, steps) => {
                            circle.to_points(steps, &mut |p| { points.push(p.into()); });
                            for (stroke, range) in &mut shape.strokes {
                                outline_points(
                                    points,
                                    stroke.thickness,
                                    false,
                                    &mut |p| { helper_points.push(p.into()); }
                                );
                                let vertex_off = vertices.len();
                                earcut::earcut(&helper_points, &[], false, vertices, indices_usize).ok();
                                *range = VertexRange::new(vertex_off..vertices.len());
                                helper_points.clear();
                            }
                            let vertex_off = vertices.len();
                            earcut::earcut(&points, &[], false, vertices, indices_usize).ok();
                            shape.shape_vertex_range = VertexRange::new(vertex_off..vertices.len());
                        },
                        Shape::Checkmark(scale) => {
                            points.clone_from_slice(&self.checkmark_points);
                            for point in &mut *points {
                                point[0] *= scale;
                                point[1] *= scale;
                            }
                            for (stroke, range) in &mut shape.strokes {
                                outline_points(
                                    points,
                                    stroke.thickness,
                                    false,
                                    &mut |p| { helper_points.push(p.into()); }
                                );
                                let vertex_off = vertices.len();
                                earcut::earcut(&helper_points, &[], false, vertices, indices_usize).ok();
                                *range = VertexRange::new(vertex_off..vertices.len());
                                helper_points.clear();
                            }
                            let vertex_off = vertices.len();
                            earcut::earcut(&points, &[], false, vertices, indices_usize).ok();
                            shape.shape_vertex_range = VertexRange::new(vertex_off..vertices.len());
                        },
                    };
                    reaction_shapes
                        .rendered_shapes.push(shape.clone());
                    points.clear();
                    helper_points.clear();
                }
            }
        }
        for (_, params) in &self.shapes {
            set_vertex_params(vertices, params.shape_vertex_range, params.offset, params.fill_col);
            for (i, stroke) in params.strokes.iter().enumerate() {
                if i as u32 == params.stroke_idx {
                    set_vertex_params(vertices, stroke.1, params.offset, stroke.0.col);
                } else {
                    hide_vertices(vertices, stroke.1);
                }
            }
        }
        self.prev_active_reactions.retain(|v| !self.active_reactions.contains(v));
        for reaction in &self.prev_active_reactions {
            let shapes = self.reaction_shapes.get(reaction).unwrap();
            shapes.hide(vertices);
        }
        self.shapes.clear();
        (vertices, indices_usize)
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
        style: &impl WindowStyle,
        text_renderer: &mut TextRenderer,
    ) -> Self {
        storage.checkmark_points.clear();
        style.get_checkmark_points(text_renderer, &mut storage.checkmark_points);
        Self {
            storage,
        }
    }

    pub fn rect(
        &mut self,
        reaction_id: ReactionId,
        rect: Rect,
        offset: Vec2,
        fill_col: ColorSRGBA,
        strokes: ArrayVec<Stroke, 4>,
        stroke_idx: u32,
    ) -> &mut Self {
        self.storage.active_reactions.insert(reaction_id);
        let entry = self.storage.reaction_shapes
            .entry(reaction_id)
            .or_default();
        let shape_params = ShapeParams::new_rect(rect, offset, fill_col, strokes, stroke_idx);
        entry.shapes.push(shape_params);
        self
    }

    pub fn circle(
        &mut self,
        reaction_id: ReactionId,
        circle: Circle,
        steps: u32,
        offset: Vec2,
        fill_col: ColorSRGBA,
        strokes: ArrayVec<Stroke, 4>,
        stroke_idx: u32,
    ) -> &mut Self {
        self.storage.active_reactions.insert(reaction_id);
        let entry = self.storage.reaction_shapes
            .entry(reaction_id)
            .or_default();
        let shape_params = ShapeParams::new_circle(circle, steps, offset, fill_col, strokes, stroke_idx);
        entry.shapes.push(shape_params);
        self
    }

    pub fn checkmark(
        &mut self,
        reaction_id: ReactionId,
        scale: f32,
        offset: Vec2,
        fill_col: ColorSRGBA,
        strokes: ArrayVec<Stroke, 4>,
        stroke_idx: u32,
    ) -> &mut Self {
        self.storage.active_reactions.insert(reaction_id);
        let entry = self.storage.reaction_shapes
            .entry(reaction_id)
            .or_default();
        let shape_params = ShapeParams::new_checkmark(scale, offset, fill_col, strokes, stroke_idx);
        entry.shapes.push(shape_params);
        self
    }
}
