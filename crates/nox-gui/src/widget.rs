use nox::{
    mem::{
        vec_types::{GlobalVec, Vector},
        value_as_bytes,
    },
    *,
};

pub use nox_geom::{
    *,
    shapes::*,
};

use crate::{
    workspace::{vertex_push_constant, RingBufMem, Vertex},
    ColorRGBA,
};

pub(crate) struct Widget {
    pub main_rect: Rect,
    pub position: Vec2,
    pub color: ColorRGBA,
    vertices: GlobalVec<Vertex>,
    indices: GlobalVec<u32>,
    points: GlobalVec<[f32; 2]>,
    indices_usize: GlobalVec<usize>,
    renderable: bool,
}

impl Widget {

    pub fn new(
        size: [f32; 2],
        position: [f32; 2],
        color: ColorRGBA,
    ) -> Self
    {
        let half_size = vec2(size[0] * 0.5, size[1] * 0.5);
        let main_rect = rect(-half_size, half_size, 0.0);
        let mut vertices = GlobalVec::new();
        let mut indices = GlobalVec::new();
        let mut points = GlobalVec::new();
        let mut indices_usize = GlobalVec::new();
        main_rect.to_points(
            &mut |p| {
                points.push(p.into());
            }
        );
        let renderable = earcut::earcut(&points, &[], false, &mut vertices, &mut indices_usize).unwrap();
        indices.append_map(&indices_usize, |&v| v as u32);
        Self {
            main_rect,
            position: position.into(),
            color,
            vertices,
            indices,
            points,
            indices_usize,
            renderable,
        }
    }

    pub fn bounding_rect(&self) -> BoundingRect {
        BoundingRect::from_center_size(self.position, self.main_rect.size())
    }

    pub fn update(
        &mut self,
        main_rect: Rect,
        position: Vec2,
        color: ColorRGBA,
    )
    {
        if !main_rect.eq_epsilon(&self.main_rect, 1.0e-6) {
            self.vertices.clear();
            self.indices.clear();
            self.points.clear();
            self.indices_usize.clear();
            self.main_rect.to_points(
                &mut |p| {
                    self.points.push(p.into());
                }
            );
            if !earcut::earcut(&self.points, &[], false, &mut self.vertices, &mut self.indices_usize).unwrap() {
                self.renderable = false;
            }
            self.indices.append_map(&self.indices_usize, |&v| v as u32);
            self.main_rect = main_rect;
        }
        self.position = position;
        self.color = color;
    }

    pub fn render_commands<F1, F2>(
        &self,
        render_commands: &mut RenderCommands,
        inv_aspect_ratio: f32,
        vertex_buf_id: BufferId,
        index_buf_id: BufferId,
        mut allocate_vertices: F1,
        mut allocate_indices: F2,
    ) -> Result<(), Error>
        where
            F1: FnMut(&mut RenderCommands, usize) -> Result<RingBufMem<Vertex>, Error>,
            F2: FnMut(&mut RenderCommands, usize) -> Result<RingBufMem<u32>, Error>,
    {
        if !self.renderable {
            return Ok(())
        }
        let vert_count = self.vertices.len();
        let vert_mem = allocate_vertices(render_commands, vert_count)?;
        unsafe {
            self.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_count);
        }
        let idx_count = self.indices.len();
        let idx_mem = allocate_indices(render_commands, idx_count)?;
        unsafe {
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_count);
        }
        let vert_pc = vertex_push_constant(self.position, inv_aspect_ratio);
        render_commands.push_constants(|p| {
            if p.offset == 0 {
                return unsafe {
                    value_as_bytes(&vert_pc).unwrap()
                }
            }
            unsafe {
                value_as_bytes(&self.color).unwrap()
            }
        })?;
        render_commands.draw_indexed(
            DrawInfo {
                index_count: idx_count as u32,
                ..Default::default()
            },
            [
                DrawBufferInfo {
                    id: vertex_buf_id,
                    offset: vert_mem.offset,
                },
            ],
            DrawBufferInfo {
                id: index_buf_id,
                offset: idx_mem.offset,
            },
        )?;
        Ok(())
    }
}
