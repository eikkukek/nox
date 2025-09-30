use nox::{
    mem::{
        value_as_bytes, vec_types::{GlobalVec, Vector}
    },
    *,
};

use rustc_hash::FxHashMap;

pub use nox_geom::{
    *,
    shapes::*,
};

use crate::{
    workspace::*,
    *
};

pub struct Widget {
    main_rect: Rect,
    position: Vec2,
    color: ColorRGBA,
    vertices: GlobalVec<Vertex>,
    indices: GlobalVec<u32>,
    points: GlobalVec<[f32; 2]>,
    indices_usize: GlobalVec<usize>,
    sliders: FxHashMap<u32, Slider>,
    active_sliders: GlobalVec<u32>,
    renderable: bool,
}

impl Widget {

    pub(crate) fn new(
        size: [f32; 2],
        position: [f32; 2],
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
            color: Default::default(),
            vertices,
            indices,
            points,
            indices_usize,
            sliders: FxHashMap::default(),
            active_sliders: Default::default(),
            renderable,
        }
    }

    pub(crate) fn render_commands<F1, F2>(
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
        let push_constants = push_constants(self.position, inv_aspect_ratio, self.color);
        render_commands.push_constants(unsafe {
            value_as_bytes(&push_constants).unwrap()
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

    pub fn update_slider<T: Sliderable>(
        &mut self,
        id: u32,
        title: &str,
        value: &mut T,
        min: T,
        max: T,
    ) -> &mut Self
    {
        let slider = self.sliders.entry(id).or_insert(Slider::new(value.calc_t(min, max), title.into()));
        if slider.clicked {
            value.slide(min, max, slider.t);
        } else {
            slider.t = value.calc_t(min, max);
        }
        self.active_sliders.push(id);
        self
    }

    pub(crate) fn update(&mut self, style: &Style) {
        self.color = style.widget_bg;
    }
}
