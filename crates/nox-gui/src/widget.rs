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
    flags: u8,
}

impl Widget {

    const RENDERABLE: u8 = 1;
    const REQUIRES_TRIANGULATION: u8 = 2;

    pub(crate) fn new(
        size: [f32; 2],
        position: [f32; 2],
    ) -> Self
    {
        let half_size = vec2(size[0] * 0.5, size[1] * 0.5);
        let main_rect = rect(-half_size, half_size, 0.0); 
        Self {
            main_rect,
            position: position.into(),
            color: Default::default(),
            vertices: Default::default(),
            indices: Default::default(),
            points: Default::default(),
            indices_usize: Default::default(),
            sliders: FxHashMap::default(),
            active_sliders: Default::default(),
            flags: Self::REQUIRES_TRIANGULATION,
        }
    }

    #[inline(always)]
    fn renderable(&self) -> bool {
        self.flags & Self::RENDERABLE != 0
    }

    #[inline(always)]
    fn requires_triangulation(&self) -> bool {
        self.flags & Self::REQUIRES_TRIANGULATION != 0
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
        if !self.renderable() {
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
        let push_constants = push_constants(self.position + self.main_rect.size() * 0.5, inv_aspect_ratio, self.color);
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

    pub(crate) fn bounding_rect(&self) -> BoundingRect {
        BoundingRect::from_position_size(self.position, self.main_rect.size())
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
        if slider.held {
            value.slide(min, max, slider.t);
        } else {
            slider.t = value.calc_t(min, max);
        }
        self.active_sliders.push(id);
        if slider.update(self.position, vec2(0.1, 0.1), 0.0) {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        self
    }

    pub(crate) fn update<I: Interface>(
        &mut self,
        nox: &Nox<I>,
        cursor_pos: Vec2,
        style: &Style,
    ) -> bool
    {
        let bounding_rect = self.bounding_rect();
        if !bounding_rect.is_point_inside(cursor_pos) {
            return false
        }
        if nox.is_mouse_button_held(MouseButton::Left) {
            self.color = style.widget_bg_hl;
        } else {
            self.color = style.widget_bg;
        }
        true
    }

    #[inline(always)]
    pub(crate) fn triangulate(&mut self) {
        if self.requires_triangulation() {
            self.points.clear();
            self.vertices.clear();
            self.indices.clear();
            self.indices_usize.clear();
            self.main_rect.to_points(&mut |p| { self.points.push(p.into()); });
            if !earcut::earcut(&self.points, &[], false, &mut self.vertices, &mut self.indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            } else {
                self.flags |= Self::RENDERABLE;
            }
            self.indices.append_map(&self.indices_usize, |&i| i as u32);
            self.flags &= !Self::REQUIRES_TRIANGULATION;
        }
    }
}
