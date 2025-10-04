use core::{
    hash::Hash,
};

use nox::{
    mem::{
        value_as_bytes, vec_types::{GlobalVec, Vector}
    },
    *,
};

use nox_font::VertexTextRenderer;

use rustc_hash::FxHashMap;

pub use nox_geom::{
    *,
    shapes::*,
};

use crate::{
    workspace::*,
    *
};

pub(crate) struct Window {
    main_rect: Rect,
    position: Vec2,
    vertices: GlobalVec<Vertex>,
    indices: GlobalVec<u32>,
    sliders: FxHashMap<u32, Slider>,
    active_sliders: GlobalVec<u32>,
    main_rect_draw_info: DrawInfo,
    flags: u8,
}

impl Window {

    const RENDERABLE: u8 = 1;
    const REQUIRES_TRIANGULATION: u8 = 2;

    pub(crate) fn new(
        size: [f32; 2],
        position: [f32; 2],
        rounding: f32,
    ) -> Self
    {
        let main_rect = rect(vec2(0.0, 0.0), size.into(), rounding); 
        Self {
            main_rect,
            position: position.into(),
            vertices: Default::default(),
            indices: Default::default(),
            sliders: FxHashMap::default(),
            active_sliders: Default::default(),
            main_rect_draw_info: Default::default(),
            flags: Self::REQUIRES_TRIANGULATION,
        }
    }

    #[inline(always)]
    fn renderable(&self) -> bool {
        self.flags & Self::RENDERABLE == Self::RENDERABLE
    }

    #[inline(always)]
    fn requires_triangulation(&self) -> bool {
        self.flags & Self::REQUIRES_TRIANGULATION == Self::REQUIRES_TRIANGULATION
    }

    pub(crate) fn bounding_rect(&self) -> BoundingRect {
        BoundingRect::from_position_size(self.position, self.main_rect.size())
    }

    pub(crate) fn update<I, FontHash>(
        &mut self,
        nox: &Nox<I>,
        cursor_pos: Vec2,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        cursor_in_other_window: bool,
    ) -> bool
        where 
            I: Interface,
            FontHash: Clone + Eq + Hash,
    {
        let cursor_in_this_window =
            !cursor_in_other_window &&
            self.bounding_rect().is_point_inside(cursor_pos);
        for id in &self.active_sliders {
            let slider = self.sliders.get_mut(id).unwrap();
            let (requires_triangulation, width) = slider.update(
                nox,
                style,
                text_renderer,
                &style.font_regular,
                self.main_rect.size().x,
                cursor_in_this_window,
                cursor_pos,
            );
            if requires_triangulation || self.main_rect.max.x != width {
                self.main_rect.max.x = width;
                self.flags |= Self::REQUIRES_TRIANGULATION;
            }
        }
        cursor_in_this_window
    }

    #[inline(always)]
    pub(crate) fn triangulate(&mut self) {
        if self.requires_triangulation() {
            self.flags |= Self::RENDERABLE;
            self.vertices.clear();
            self.indices.clear();
            let mut points = GlobalVec::new();
            let mut indices_usize = GlobalVec::new();
            self.main_rect.to_points(&mut |p| { points.push(p.into()); });
            if !earcut::earcut(&points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.main_rect_draw_info = DrawInfo {
                first_index: 0,
                index_count: indices_usize.len() as u32,
                vertex_offset: 0,
                ..Default::default()
            };
            self.flags &= !Self::REQUIRES_TRIANGULATION;
            for id in &self.active_sliders {
                let slider = self.sliders.get_mut(id).unwrap();
                slider.triangulate(&mut points,
                    |points| {
                        let mut draw_info = DrawInfo {
                            first_index: indices_usize.len() as u32,
                            ..Default::default()
                        };
                        if !earcut::earcut(points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                            self.flags &= !Self::RENDERABLE;
                        }
                        draw_info.index_count = indices_usize.len() as u32 - draw_info.first_index;
                        draw_info
                    },
                );
            }
            self.indices.append_map(&indices_usize, |&i| i as u32);
        }
    }

    pub(crate) fn render_commands<FontHash>(
        &mut self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        inv_aspect_ratio: f32,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        base_pipeline: GraphicsPipelineId,
        text_pipeline: GraphicsPipelineId,
    ) -> Result<(), Error>
    {
        if !self.renderable() {
            return Ok(())
        }
        let vert_total = self.vertices.len();
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_total)?
        };
        unsafe {
            self.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_total);
        }
        let idx_total = self.indices.len();
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, idx_total)?
        };
        unsafe {
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_total);
        }
        let push_constants_vertex = push_constants_vertex(
            self.position,
            inv_aspect_ratio,
        );
        render_commands.bind_pipeline(base_pipeline)?;
        let push_constants_fragment = push_constants_fragment(style.window_bg_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                value_as_bytes(&push_constants_vertex).unwrap()
            } else {
                value_as_bytes(&push_constants_fragment).unwrap()
            }
        })?;
        render_commands.draw_indexed(
            self.main_rect_draw_info,
            [
                DrawBufferInfo {
                    id: vertex_buffer.id(),
                    offset: vert_mem.offset,
                },
            ],
            DrawBufferInfo {
                id: index_buffer.id(),
                offset: idx_mem.offset,
            },
        )?;
        for id in &self.active_sliders {
            let slider = self.sliders.get(id).unwrap();
            slider.render_commands(
                render_commands,
                style,
                inv_aspect_ratio,
                vertex_buffer,
                index_buffer,
                vert_mem.offset,
                idx_mem.offset,
                base_pipeline,
                text_pipeline,
            )?;
        }
        self.active_sliders.clear();
        Ok(())
    }
}

pub struct WindowContext<'a, 'b, FontHash>
    where
        FontHash: Clone + Eq + Hash, 
{
    style: &'a Style<FontHash>,
    window: &'a mut Window,
    text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
    widget_y: f32,
}

impl<'a, 'b, FontHash> WindowContext<'a, 'b, FontHash>
    where
        FontHash: Clone + Eq + Hash,
{

    pub(crate) fn new(
        window: &'a mut Window,
        style: &'a Style<FontHash>,
        text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
    ) -> Self {
        Self {
            window,
            style,
            widget_y: style.item_pad_outer.y,
            text_renderer,
        }
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
        self.window.active_sliders.push(id);
        let slider = self.window.sliders.entry(id).or_insert(Slider::new(value.calc_t(min, max), title.into()));
        if slider.held {
            value.slide(min, max, slider.t);
        } else {
            slider.t = value.calc_t(min, max);
        }
        let text_size = slider.calc_text_size(self.text_renderer, &self.style.font_regular);
        slider.set_position(self.window.position + vec2(self.style.item_pad_outer.x, self.widget_y));
        self.widget_y += self.style.calc_item_height(text_size.y) + self.style.item_pad_outer.y;
        self
    }
}
