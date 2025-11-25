use core::{
    cell::UnsafeCell,
    ptr::NonNull,
};

use rustc_hash::{FxHashMap, FxHashSet};

use nox::{
    mem::{
        vec_types::{ArrayVec, GlobalVec, Vector},
        Allocator,
    },
    alloc::arena_alloc::ArenaAlloc,
    *,
};

use nox_geom::{
    shapes::*,
    *,
};

use crate::{image::{ImageSourceInternal, ImageSourceUnsafe, ImageData}, *};

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
                match other {
                    Shape::FlatRect(_, _) => true,
                    _ => false,
                }
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
            stroke_idx,
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
            stroke_idx: 0,
        }
    }
}

#[derive(Default)]
struct ReactionShapes {
    shapes: GlobalVec<ShapeParams>,
    rendered_shapes: GlobalVec<ShapeParams>,
    prev_shapes: GlobalVec<(Shape, ArrayVec<f32, 4>)>,
    images_by_path: FxHashMap<CompactString, UnsafeCell<ImageData>>,
    images_by_id: FxHashMap<ImageId, UnsafeCell<ImageData>>,
    prev_active_images: GlobalVec<ImageSourceUnsafe>,
    active_images: GlobalVec<ImageSourceUnsafe>,
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
    fn active_image_iter(&self) -> impl Iterator<Item = Option<&mut ImageData>> {
        self.active_images
            .iter()
            .map(|source| unsafe {
                match source.as_image_source() {
                    ImageSource::Path(p) => {
                        self.images_by_path
                            .get(p)
                            .map(|i| &mut *i.get())
                    },
                    ImageSource::Id(id) => {
                        self.images_by_id
                            .get(&id)
                            .map(|i| &mut *i.get())
                    },
                }
            })
    }

    #[inline(always)]
    fn prev_image_iter(&self) -> impl Iterator<Item = Option<&mut ImageData>> {
        self.prev_active_images
            .iter()
            .map(|source| unsafe {
                match source.as_image_source() {
                    ImageSource::Path(p) => {
                        self.images_by_path
                            .get(p)
                            .map(|i| &mut *i.get())
                    },
                    ImageSource::Id(id) => {
                        self.images_by_id
                            .get(&id)
                            .map(|i| &mut *i.get())
                    },
                }
            })
    }

    #[inline(always)]
    fn hide(
        &self,
        vertices: &mut [Vertex],
        window_semaphore: (TimelineSemaphoreId, u64),
        global_resources: &mut GlobalResources,
        tmp_alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        for params in &self.rendered_shapes {
            hide_vertices(vertices, params.shape_vertex_range);
            for &(_, range) in &params.strokes {
                hide_vertices(vertices, range);
            }
        }
        for data in self.prev_image_iter() {
            if let Some(data) = data {
                data.hide(window_semaphore, global_resources, tmp_alloc)?;
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.prev_shapes.clear();
        self.prev_shapes.append_map(&self.shapes, |v| (v.shape, v.strokes.mapped(|&v| v.0.thickness)));
        if self.rendered_shapes.len() == self.shapes.len() {
            for (i, shape) in self.rendered_shapes.iter_mut().enumerate() {
                let update = self.shapes[i].clone();
                shape.shape = update.shape;
                shape.offset = update.offset;
                shape.fill_col = update.fill_col;
                for (j, stroke) in update.strokes.iter().enumerate() {
                    shape.strokes[j].0 = stroke.0;
                }
                shape.stroke_idx = update.stroke_idx;
            }
        }
        self.shapes.clear();
        self.prev_active_images.clear();
        self.prev_active_images.append(&self.active_images);
        self.active_images.clear();
    }
}

pub struct PainterStorage {
    vertices: GlobalVec<Vertex>,
    indices_usize: GlobalVec<usize>,
    indices: GlobalVec<u32>,
    points: GlobalVec<[f32; 2]>,
    checkmark_points: GlobalVec<[f32; 2]>,
    helper_points: GlobalVec<[f32; 2]>,
    reaction_shapes: FxHashMap<ReactionId, ReactionShapes>,
    active_reactions: FxHashSet<ReactionId>,
    prev_active_reactions: GlobalVec<ReactionId>,
    shapes: GlobalVec<(ReactionId, ShapeParams)>,
    stack: ArenaAlloc,
    flags: u32,
}

impl PainterStorage {

    const REQUIRES_TRANSFER_COMMANDS: u32 = 0x1;

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            vertices: Default::default(),
            indices_usize: Default::default(),
            indices: Default::default(),
            points: Default::default(),
            checkmark_points: Default::default(),
            helper_points: Default::default(),
            reaction_shapes: FxHashMap::default(),
            active_reactions: FxHashSet::default(),
            prev_active_reactions: Default::default(),
            shapes: Default::default(),
            stack: ArenaAlloc::new(1 << 16).unwrap(),
            flags: 0,
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
        self.flags &= !Self::REQUIRES_TRANSFER_COMMANDS;
    }

    pub fn end(
        &mut self,
        window_semaphore: (TimelineSemaphoreId, u64),
        global_resources: &mut GlobalResources,
        tmp_alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        self.prev_active_reactions.retain(|v| !self.active_reactions.contains(v));
        for reaction in &self.prev_active_reactions {
            let shapes = self.reaction_shapes.get(reaction).unwrap();
            shapes.hide(&mut self.vertices, window_semaphore, global_resources, tmp_alloc)?;
        }
        Ok(())
    }

    #[inline(always)]
    pub fn requires_transfer_commands(&self) -> bool {
        self.flags & Self::REQUIRES_TRANSFER_COMMANDS == Self::REQUIRES_TRANSFER_COMMANDS
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
        let indices_usize = &mut self.indices_usize;
        let indices = &mut self.indices;
        if requires_triangulation {
            vertices.clear();
            indices_usize.clear();
            indices.clear();
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
                        Shape::FlatRect(min, max) => {
                            let vertex_off = vertices.len();
                            vertices.append(&[
                                min.into(),
                                vec2(min.x, max.y).into(),
                                max.into(),
                                vec2(max.x, min.y).into(),
                            ]);
                            shape.shape_vertex_range = VertexRange::new(vertex_off..vertices.len());
                            indices_usize.append(&[
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
            indices.append_map(&indices_usize, |&v| v as u32);
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
                    if i as u32 == params.stroke_idx {
                        set_vertex_params(vertices, stroke.1, offset, stroke.0.col);
                    } else {
                        hide_vertices(vertices, stroke.1);
                    }
                }
            }
        }
        self.shapes.clear();
    }

    pub fn render(
        &mut self,
        frame_graph: &mut dyn FrameGraph,
        render_format: ColorFormat,
        add_read: &mut dyn FnMut(ReadInfo),
    ) -> Result<(), Error> {
        for id in &self.active_reactions {
            if let Some(shapes) = self.reaction_shapes.get_mut(id) {
                for data in shapes.active_image_iter() {
                    if let Some(data) = data {
                        data.render(frame_graph, render_format, add_read)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn transfer_commands(
        &mut self,
        transfer_commands: &mut TransferCommands,
        window_semaphore: (TimelineSemaphoreId, u64),
        sampler: SamplerId,
        texture_pipeline_layout: PipelineLayoutId,
        tmp_alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        for id in &self.active_reactions {
            let shapes= self.reaction_shapes
                .get_mut(id)
                .unwrap();
            for data in shapes.active_image_iter() {
                if let Some(data) = data {
                    data.transfer_commands(
                        transfer_commands,
                        window_semaphore,
                        sampler,
                        texture_pipeline_layout,
                        tmp_alloc
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn render_commands(
        &mut self,
        render_commands: &mut RenderCommands,
        sampler: SamplerId,
        offset: Vec2,
        bounds: BoundingRect,
        base_pipeline: GraphicsPipelineId,
        text_pipeline: GraphicsPipelineId,
        texture_pipeline: GraphicsPipelineId,
        texture_pipeline_layout: PipelineLayoutId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        tmp_alloc: &impl Allocator,
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<(), Error>
    {
        let vert_count = self.vertices.len();
        let idx_count = self.indices.len();
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_count)?
        };
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, idx_count)?
        };
        unsafe {
            self.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_count);
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_count);
        }
        let draw_info = DrawInfo {
            index_count: idx_count as u32,
            ..Default::default()
        };
        render_commands.bind_pipeline(base_pipeline)?;
        let pc_vertex = push_constants_vertex(
            offset,
            vec2(1.0, 1.0),
            inv_aspect_ratio,
            unit_scale,
        );
        let pc_fragment = base_push_constants_fragment(
            bounds.min,
            bounds.max,
        );
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            draw_info,
            [
                DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset),
            ],
            DrawBufferInfo {
                id: index_buffer.id(),
                offset: idx_mem.offset,
            },
        )?;
        for id in &self.active_reactions {
            let shapes = self.reaction_shapes
                .get_mut(id)
                .unwrap();
            for data in shapes.active_image_iter() {
                if let Some(data) = data {
                    data.render_commands(
                        render_commands, sampler,
                        texture_pipeline, texture_pipeline_layout,
                        offset, bounds,
                        inv_aspect_ratio, unit_scale, tmp_alloc
                    )?;
                }
            }
        }
        Ok(())
    }
}

pub struct Painter<'a> {
    storage: &'a mut PainterStorage,
    image_loader: &'a mut ImageLoader,
}

impl<'a> Painter<'a>
{

    #[inline(always)]
    pub fn new(
        storage: &'a mut PainterStorage,
        style: &impl UiStyle,
        text_renderer: &mut TextRenderer,
        image_loader: &'a mut ImageLoader,
    ) -> Self {
        storage.checkmark_points.clear();
        style.get_checkmark_points(text_renderer, &mut storage.checkmark_points);
        Self {
            storage,
            image_loader,
        }
    }

    #[inline(always)]
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
        let shape_params = ShapeParams::new_rect(
            rect,
            offset,
            fill_col,
            strokes,
            stroke_idx,
        );
        entry.shapes.push(shape_params);
        self
    }

    #[inline(always)]
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
        let shape_params = ShapeParams::new_circle(
            circle,
            steps,
            offset,
            fill_col,
            strokes,
            stroke_idx,
        );
        entry.shapes.push(shape_params);
        self
    }

    #[inline(always)]
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
        let shape_params = ShapeParams::new_checkmark(
            scale,
            offset,
            fill_col,
            strokes,
            stroke_idx,
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
                let src = self.image_loader.load_image(p);
                if let Some(data) = entry.images_by_path
                    .get_mut(p)
                {
                    let data = data.get_mut();
                    data.update_source(src, offset, size);
                    if data.requires_transfer_commands() {
                        self.storage.flags |= PainterStorage::REQUIRES_TRANSFER_COMMANDS;
                    }
                } else
                {
                    let data = entry.images_by_path
                        .entry(p.into())
                        .or_default();
                    let data = data.get_mut();
                    data.update_source(src, offset, size);
                    if data.requires_transfer_commands() {
                        self.storage.flags |= PainterStorage::REQUIRES_TRANSFER_COMMANDS;
                    }
                }
                let len = p.len();
                if let Some(data) = self.storage.stack.allocate_uninit(len) {
                    p.as_ptr()
                        .copy_to_nonoverlapping(data.as_ptr(), len);
                    ImageSourceUnsafe::Path(data, len)
                } else {
                    ImageSourceUnsafe::Path(NonNull::dangling(), 0)
                }
            },
            ImageSource::Id(id) => {
                let src = ImageSourceInternal::Id(id);
                let data = entry.images_by_id
                    .entry(id)
                    .or_default();
                let data = data.get_mut();
                data.update_source(src, offset, size);
                if data.requires_transfer_commands() {
                    self.storage.flags |= PainterStorage::REQUIRES_TRANSFER_COMMANDS;
                }
                ImageSourceUnsafe::Id(id)
            },
        };
        entry.active_images.push(source);
        self
    }
}
