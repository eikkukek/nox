use core::ptr::NonNull;

use ahash::{AHashMap, AHashSet};

use compact_str::format_compact;

use nox_proc::BuildStructure;
use nox::{
    mem::{
        vec::{ArrayVec, Vec32},
        alloc::LocalAllocExt,
        AsRaw,
    },
    alloc::arena::Arena,
    gpu,
    error::*,
};

use nox_geom::{
    shapes::*,
    *,
};

use crate::{
    image::{ImageSourceInternal, ImageSourceUnsafe, ImageData},
    *,
};

#[derive(Default, Clone, Copy)]
pub struct Stroke {
    pub col: ColorSRGBA,
    pub thickness: f32,
}

#[derive(Clone, Copy)]
enum Shape {
    Rect(Rect),
    Circle(Circle, u32),
    Checkmark(f32),
    FlatRect(Vec2, Vec2),
}

impl PartialEq for Shape {

    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Rect(rect) => {
                match other {
                    Shape::Rect(other) => rect == other,
                    _ => false,
                }
            },
            Self::Circle(circle, steps) => {
                match other {
                    Shape::Circle(other_circle, other_steps) =>
                        circle == other_circle &&
                        steps == other_steps,
                    _ => false,
                }
            },
            Self::Checkmark(scale) => {
                match other {
                    Shape::Checkmark(other)  => scale == other,
                    _ => false,
                }
            },
            Self::FlatRect(_, _) => {
                matches!(other, Shape::FlatRect(_, _))
            },
        }
    }
}

#[derive(Clone)]
struct ShapeParams {
    shape: Shape,
    offset: Vec2,
    fill_col: ColorSRGBA,
    shape_vertex_range: Option<VertexRange>,
    strokes: ArrayVec<(Stroke, Option<VertexRange>), 4>,
    stroke_type: StrokeType,
}

impl ShapeParams {

    #[inline(always)]
    fn new_rect(
        rect: Rect,
        offset: Vec2,
        fill_col: ColorSRGBA,
        strokes: ArrayVec<Stroke, 4>,
        stroke_type: StrokeType,
    ) -> Self {
        Self {
            shape: Shape::Rect(rect),
            offset,
            fill_col,
            shape_vertex_range: None,
            strokes: strokes.map(|&v| (v, None)),
            stroke_type,
        }
    }

    #[inline(always)]
    fn new_circle(
        circle: Circle,
        steps: u32,
        offset: Vec2,
        fill_col: ColorSRGBA,
        strokes: ArrayVec<Stroke, 4>,
        stroke_type: StrokeType,
    ) -> Self {
        Self {
            shape: Shape::Circle(circle, steps),
            offset,
            fill_col,
            shape_vertex_range: None,
            strokes: strokes.map(|&v| (v, None)),
            stroke_type,
        }
    }

    #[inline(always)]
    fn new_checkmark(
        scale: f32,
        offset: Vec2,
        fill_col: ColorSRGBA,
        strokes: ArrayVec<Stroke, 4>,
        stroke_type: StrokeType,
    ) -> Self {
        Self {
            shape: Shape::Checkmark(scale),
            offset,
            fill_col,
            shape_vertex_range: None,
            strokes: strokes.map(|&v| (v, None)),
            stroke_type,
        }
    }

    #[inline(always)]
    fn new_flat_rect(
        min: Vec2,
        max: Vec2,
        offset: Vec2,
        fill_col: ColorSRGBA,
    ) -> Self {
        Self {
            shape: Shape::FlatRect(min, max),
            offset,
            fill_col,
            shape_vertex_range: None,
            strokes: Default::default(),
            stroke_type: StrokeType::Type1,
        }
    }
}

#[derive(Default)]
struct ReactionShapes {
    shapes: Vec32<ShapeParams>,
    rendered_shapes: Vec32<ShapeParams>,
    prev_shapes: Vec32<(Shape, ArrayVec<f32, 4>)>,
    images_by_path: AHashMap<CompactString, ImageData>,
    images_by_id: AHashMap<gpu::ImageViewId, ImageData>,
    prev_active_images: Vec32<ImageSourceUnsafe>,
    active_images: Vec32<ImageSourceUnsafe>,
}

impl ReactionShapes {

    #[inline(always)]
    fn changed(&mut self) -> bool {
        self.shapes.len() != self.rendered_shapes.len() ||
        self.shapes
            .iter()
            .map(|v| (v.shape, v.strokes.map(|&v| v.0.thickness)))
            .ne(self.prev_shapes.iter().cloned())
    }

    #[inline(always)]
    fn active_image_iter<'a>(&'a mut self) -> impl Iterator<Item = &'a mut ImageData> + 'a {
        let s: *mut Self = self;
        (0..).scan(s, |s, idx| {
            let s = unsafe { &mut **s };
            s.active_images
                .get(idx)
                .and_then(|&source| unsafe {
                    match source.as_image_source() {
                        ImageSource::Path(p) => {
                            s.images_by_path
                                .get_mut(p)
                        },
                        ImageSource::Id(id) => {
                            s.images_by_id
                                .get_mut(&id)
                        },
                    }
                }) 
        })
    } 

    #[inline(always)]
    fn hide(
        &self,
        vertices: &mut [Vertex],
    ) -> Result<()>
    {
        for params in &self.rendered_shapes {
            hide_vertices(vertices, params.shape_vertex_range);
            for &(_, range) in &params.strokes {
                hide_vertices(vertices, range);
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.prev_shapes.clear();
        self.prev_shapes.extend(self.shapes
            .iter()
            .map(|v| (v.shape, v.strokes.map(|&v| v.0.thickness)))
        );
        if self.rendered_shapes.len() == self.shapes.len() {
            for (i, shape) in self.rendered_shapes.iter_mut().enumerate() {
                let update = self.shapes[i].clone();
                shape.shape = update.shape;
                shape.offset = update.offset;
                shape.fill_col = update.fill_col;
                for (j, stroke) in update.strokes.iter().enumerate() {
                    shape.strokes[j].0 = stroke.0;
                }
                shape.stroke_type = update.stroke_type;
            }
        }
        self.shapes.clear();
        self.prev_active_images.clear();
        self.prev_active_images.append(&self.active_images);
        self.active_images.clear();
    }
}

pub struct PainterStorage {
    vertices: Vec32<Vertex>,
    indices: Vec32<u32>,
    points: Vec32<[f32; 2]>,
    checkmark_points: Vec32<[f32; 2]>,
    helper_points: Vec32<[f32; 2]>,
    reaction_shapes: AHashMap<ReactionId, ReactionShapes>,
    active_reactions: AHashSet<ReactionId>,
    prev_active_reactions: Vec32<ReactionId>,
    shapes: Vec32<(ReactionId, ShapeParams)>,
    stack: Arena,
}

impl PainterStorage {

    #[inline(always)]
    pub fn new(stack_size: usize) -> Self {
        Self {
            vertices: Default::default(),
            indices: Default::default(),
            points: Default::default(),
            checkmark_points: Default::default(),
            helper_points: Default::default(),
            reaction_shapes: AHashMap::default(),
            active_reactions: AHashSet::default(),
            prev_active_reactions: Default::default(),
            shapes: Default::default(),
            stack: Arena::new(stack_size).unwrap(),
        }
    } 

    pub fn begin(&mut self) {
        self.prev_active_reactions.clear();
        for &id in &self.active_reactions {
            self.reaction_shapes.get_mut(&id).unwrap().reset();
            self.prev_active_reactions.push(id);
        }
        self.active_reactions.clear();
        unsafe {
            self.stack.clear();
        }
    }

    pub fn end(
        &mut self,
    ) -> Result<()>
    {
        self.prev_active_reactions.retain(|v| !self.active_reactions.contains(v));
        for reaction in &self.prev_active_reactions {
            let shapes = self.reaction_shapes.get(reaction).unwrap();
            shapes.hide(&mut self.vertices)
                .context("failed to hide shape")?;
        }
        Ok(())
    }

    pub fn triangulate(&mut self)
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
        let indices = &mut self.indices;
        if requires_triangulation {
            vertices.clear();
            indices.clear();
            self.shapes.clear();
            for shapes in &mut self.reaction_shapes {
                shapes.1.rendered_shapes.clear();
            }
            vertices.clear();
            let points = &mut self.points;
            let helper_points = &mut self.helper_points;
            for id in self.active_reactions.iter() {
                let reaction_shapes = self.reaction_shapes.get_mut(id).unwrap();
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
                                earcut::earcut(helper_points, &[], false, vertices, indices);
                                *range = VertexRange::new(vertex_off..vertices.len());
                                helper_points.clear();
                            }
                            let vertex_off = vertices.len();
                            earcut::earcut(points, &[], false, vertices, indices);
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
                                earcut::earcut(helper_points, &[], false, vertices, indices);
                                *range = VertexRange::new(vertex_off..vertices.len());
                                helper_points.clear();
                            }
                            let vertex_off = vertices.len();
                            earcut::earcut(points, &[], false, vertices, indices);
                            shape.shape_vertex_range = VertexRange::new(vertex_off..vertices.len());
                        },
                        Shape::Checkmark(scale) => {
                            points.fast_append(&self.checkmark_points);
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
                                earcut::earcut(helper_points, &[], false, vertices, indices);
                                *range = VertexRange::new(vertex_off..vertices.len());
                                helper_points.clear();
                            }
                            let vertex_off = vertices.len();
                            earcut::earcut(points, &[], false, vertices, indices);
                            shape.shape_vertex_range = VertexRange::new(vertex_off..vertices.len());
                        },
                        Shape::FlatRect(min, max) => {
                            let vertex_off = vertices.len();
                            vertices.append(&[
                                min.into(),
                                vec2(min.x, max.y).into(),
                                max.into(),
                                vec2(max.x, min.y).into(),
                            ]);
                            shape.shape_vertex_range = VertexRange::new(vertex_off..vertices.len());
                            indices.fast_append(&[
                                vertex_off, vertex_off + 2, vertex_off + 1,
                                vertex_off + 3, vertex_off + 2, vertex_off,
                            ]);
                        },
                    };
                    reaction_shapes
                        .rendered_shapes.push(shape.clone());
                    self.shapes.push((*id, shape.clone()));
                    points.clear();
                    helper_points.clear();
                }
            }
        }
        for (_, params) in self.shapes.iter().cloned() {
            let offset = params.offset;
            if let Shape::FlatRect(min, max) = params.shape {
                if min.x != max.x && min.y != max.y {
                    if let Some(range) = params.shape_vertex_range {
                        let color = params.fill_col;
                        let start = range.start();
                        let mut vertex = &mut vertices[start];
                        vertex.pos = min;
                        vertex.offset = offset;
                        vertex.color = color;
                        vertex = &mut vertices[start + 1];
                        vertex.pos = vec2(min.x, max.y);
                        vertex.offset = offset;
                        vertex.color = color;
                        vertex = &mut vertices[start + 2];
                        vertex.pos = max;
                        vertex.offset = offset;
                        vertex.color = color;
                        vertex = &mut vertices[start + 3];
                        vertex.pos = vec2(max.x, min.y);
                        vertex.offset = offset;
                        vertex.color = color;
                    }
                } else {
                    hide_vertices(vertices, params.shape_vertex_range);
                }
            } else {
                set_vertex_params(vertices, params.shape_vertex_range, offset, params.fill_col);
                for (i, stroke) in params.strokes.iter().enumerate() {
                    if i as u32 == params.stroke_type.as_raw() {
                        set_vertex_params(vertices, stroke.1, offset, stroke.0.col);
                    } else {
                        hide_vertices(vertices, stroke.1);
                    }
                }
            }
        }
        self.shapes.clear();
    } 

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        cmd: &mut gpu::DrawCommands<'_>,
        rec: &mut RecordCmd<'_>,
        cached_data: &CachedUiData,
        sampler: gpu::Sampler,
        offset: Vec2,
        bounds: BoundingRect,
    ) -> Result<()>
    {
        let vert_count = self.vertices.len();
        let idx_count = self.indices.len();
        let vert_mem = rec.allocate_vertices(vert_count)?;
        let idx_mem = rec.allocate_indices(idx_count)?;
        unsafe {
            self.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_count as usize);
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_count as usize);
        }
        let draw_info = gpu::IndexedDrawInfo {
            index_count: idx_count,
            ..Default::default()
        };
        let (viewport, scissor) = cached_data.viewport_and_scissor();
        let mut pipeline_cmd = cmd.bind_pipeline(rec.base_pipeline(), &[viewport], &[scissor])?;
        let pc_vertex = push_constants_vertex(
            offset,
            vec2(1.0, 1.0),
            cached_data.inv_aspect_ratio,
            cached_data.unit_scale,
        );
        let pc_fragment = base_push_constants_fragment(
            bounds.min,
            bounds.max,
        );
        pipeline_cmd.push_constants(pc_vertex.0, &[pc_vertex.1])?;
        pipeline_cmd.push_constants(pc_fragment.0, &[pc_fragment.1])?;
        pipeline_cmd.begin_drawing_indexed(
            draw_info,
            gpu::IndexBufferInfo::new(
                rec.index_buffer_id(),
                idx_mem.offset,
            ),
            &[gpu::DrawBufferRange::new(
                rec.vertex_buffer_id(),
                vert_mem.offset,
                vert_mem.size
            )], None,
            |cmd| { cmd.draw_indexed()?; Ok(()) }
        )?;
        for id in &self.active_reactions {
            let shapes = self.reaction_shapes
                .get_mut(id)
                .unwrap();
            for data in shapes.active_image_iter() {
                data.draw(cmd, rec, sampler.clone(), cached_data, offset, bounds)
                    .context_with(|| format_compact!(
                        "failed to render image"
                    ))?;
            }
        }
        Ok(())
    }
}

pub use nox_geom::shapes::Rect;

#[derive(Default, Clone, BuildStructure)]
pub struct UiRect {
    #[skip]
    pub rect: Rect,
    pub strokes: InteractStrokes,
    pub stroke_type: StrokeType,
    pub fill_col: ColorSRGBA,
}

impl UiRect {

    #[inline]
    pub fn fg(visuals: &InteractVisuals) -> Self {
        Self {
            rect: Default::default(),
            strokes: visuals.fg_strokes.clone(),
            stroke_type: visuals.fg_stroke_type,
            fill_col: visuals.fill_col,
        }
    }

    #[inline]
    pub fn bg(visuals: &InteractVisuals) -> Self {
        Self {
            rect: Default::default(),
            strokes: visuals.bg_strokes.clone(),
            stroke_type: visuals.bg_stroke_type,
            fill_col: visuals.fill_col,
        }
    }

    #[inline]
    pub fn rect<P: Into<Vec2>>(mut self, min: P, max: P, rounding: f32) -> Self
    {
        self.rect = rect(min, max, rounding);
        self
    }
}

pub use nox_geom::shapes::Circle;

#[derive(Clone, BuildStructure)]
pub struct UiCircle {
    #[skip]
    pub circle: Circle,
    /// Specifies how many segments the circle is divided to when triangulating.
    ///
    /// Default value is 16.
    #[default(16)]
    pub steps: u32,
    pub strokes: InteractStrokes,
    pub stroke_type: StrokeType,
    pub fill_col: ColorSRGBA,
}

impl UiCircle {

    #[inline]
    pub fn fg(visuals: &InteractVisuals) -> Self {
        Self {
            circle: Default::default(),
            steps: 16,
            strokes: visuals.fg_strokes.clone(),
            stroke_type: visuals.fg_stroke_type,
            fill_col: visuals.fill_col,
        }
    }

    #[inline]
    pub fn bg(visuals: &InteractVisuals) -> Self {
        Self {
            circle: Default::default(),
            steps: 16,
            strokes: visuals.bg_strokes.clone(),
            stroke_type: visuals.bg_stroke_type,
            fill_col: visuals.fill_col,
        }
    }

    #[inline]
    pub fn circle<P: Into<Vec2>>(mut self, origin: P, radius: f32) -> Self {
        self.circle = circle(origin, radius);
        self
    }
}

#[derive(Clone, BuildStructure)]
pub struct UiCheckmark {
    /// Specifies how big the checkmark will be relative to text size.
    ///
    /// Default value is 1.0.
    #[default(1.0)]
    pub scale: f32,
    pub strokes: InteractStrokes,
    pub stroke_type: StrokeType,
    pub fill_col: ColorSRGBA,
}

impl UiCheckmark {

    #[inline]
    pub fn fg(visuals: &InteractVisuals) -> Self {
        Self {
            scale: 1.0,
            strokes: visuals.fg_strokes.clone(),
            stroke_type: visuals.fg_stroke_type,
            fill_col: visuals.fill_col,
        }
    }

    #[inline]
    pub fn bg(visuals: &InteractVisuals) -> Self {
        Self {
            scale: 1.0,
            strokes: visuals.bg_strokes.clone(),
            stroke_type: visuals.bg_stroke_type,
            fill_col: visuals.fill_col,
        }
    }
}

pub struct Painter<'a> {
    storage: &'a mut PainterStorage,
    image_loader: &'a mut ImageLoader,
    command_dependencies: &'a mut Vec32<gpu::CommandDependency>,
}

impl<'a> Painter<'a>
{

    #[inline(always)]
    pub fn new(
        storage: &'a mut PainterStorage,
        style: &UiStyle,
        text_renderer: &mut TextRenderer,
        image_loader: &'a mut ImageLoader,
        command_dependencies: &'a mut Vec32<gpu::CommandDependency>,
    ) -> Self {
        storage.checkmark_points.clear();
        style.get_checkmark_points(text_renderer, &mut storage.checkmark_points);
        Self {
            storage,
            image_loader,
            command_dependencies,
        }
    }

    #[inline(always)]
    pub fn rect(
        &mut self,
        reaction_id: ReactionId,
        offset: Vec2,
        rect: UiRect,
    ) -> &mut Self {
        self.storage.active_reactions.insert(reaction_id);
        let entry = self.storage.reaction_shapes
            .entry(reaction_id)
            .or_default();
        let shape_params = ShapeParams::new_rect(
            rect.rect,
            offset,
            rect.fill_col,
            rect.strokes,
            rect.stroke_type,
        );
        entry.shapes.push(shape_params);
        self
    }

    #[inline(always)]
    pub fn circle(
        &mut self,
        reaction_id: ReactionId,
        offset: Vec2,
        circle: UiCircle,
    ) -> &mut Self {
        self.storage.active_reactions.insert(reaction_id);
        let entry = self.storage.reaction_shapes
            .entry(reaction_id)
            .or_default();
        let shape_params = ShapeParams::new_circle(
            circle.circle,
            circle.steps,
            offset,
            circle.fill_col,
            circle.strokes,
            circle.stroke_type,
        );
        entry.shapes.push(shape_params);
        self
    }

    #[inline(always)]
    pub fn checkmark(
        &mut self,
        reaction_id: ReactionId,
        offset: Vec2,
        check_mark: UiCheckmark,
    ) -> &mut Self {
        self.storage.active_reactions.insert(reaction_id);
        let entry = self.storage.reaction_shapes
            .entry(reaction_id)
            .or_default();
        let shape_params = ShapeParams::new_checkmark(
            check_mark.scale,
            offset,
            check_mark.fill_col,
            check_mark.strokes,
            check_mark.stroke_type,
        );
        entry.shapes.push(shape_params);
        self
    }

    #[inline(always)]
    pub fn flat_rect(
        &mut self,
        reaction_id: ReactionId,
        min: Vec2,
        max: Vec2,
        offset: Vec2,
        fill_col: ColorSRGBA
    ) -> &mut Self {
        self.storage.active_reactions.insert(reaction_id);
        let entry = self.storage.reaction_shapes
            .entry(reaction_id)
            .or_default();
        let shape_params = ShapeParams::new_flat_rect(
            min,
            max,
            offset,
            fill_col,
        );
        entry.shapes.push(shape_params);
        self
    }

    #[inline(always)]
    #[track_caller]
    pub fn image(
        &mut self,
        reaction_id: ReactionId,
        source: ImageSource,
        offset: Vec2,
        size: Vec2,
    ) -> &mut Self {
        self.storage.active_reactions.insert(reaction_id);
        let entry = self.storage.reaction_shapes
            .entry(reaction_id)
            .or_default();
        let source = match source {
            ImageSource::Path(p) => unsafe {
                let (src, dep) = self.image_loader.load_image(p);
                if let Some(dep) = dep {
                    self.command_dependencies.push(dep);
                }
                if let Some(data) = entry.images_by_path
                    .get_mut(p)
                {
                    data.update_source(src, caller!(), offset, size);
                } else {
                    let data = entry.images_by_path
                        .entry(p.into())
                        .or_default();
                    data.update_source(src, caller!(), offset, size);
                }
                let len = p.len();
                if let Ok(data) = self.storage.stack.alloc_uninit(len) {
                    p.as_ptr()
                        .copy_to_nonoverlapping(data.as_ptr(), len);
                    ImageSourceUnsafe::Path(data, len)
                } else {
                    ImageSourceUnsafe::Path(NonNull::dangling(), 0)
                }
            },
            ImageSource::Id(id) => {
                let src = ImageSourceInternal {
                    view_id: id,
                };
                let data = entry.images_by_id
                    .entry(id)
                    .or_default();
                data.update_source(src, caller!(), offset, size);
                ImageSourceUnsafe::Id(id)
            },
        };
        entry.active_images.push(source);
        self
    }
}
