use core::{
    fmt::Display,
    hash::Hash,
};

use compact_str::CompactString;

use nox::{
    *,
    mem::{
        vec_types::{GlobalVec, Vector},
        value_as_bytes,
    },
};

use nox_font::{VertexTextRenderer, text_segment, RenderedText};

use nox_geom::{
    shapes::Rect,
    BoundingRect,
    Vec2,
    vec2,
};

use crate::{workspace::*, *};

pub trait Sliderable: Copy + Display {

    fn slide(&mut self, min: Self, max: Self, t: f32);

    fn calc_t(&self, min: Self, max: Self) -> f32;
}

pub(crate) struct Slider
{
    title: CompactString,
    main_rect: Rect,
    position: Vec2,
    pub t: f32,
    title_text: Option<RenderedText>,
    main_rect_draw_info: DrawInfo,
    index_off: u32,
    vertex_off: u32,
    pub held: bool,
}

impl Slider
{

    pub fn new(
        t: f32,
        title: &str,
    ) -> Self
    {
        Self {
            title: CompactString::new(title),
            main_rect: Default::default(),
            position: Default::default(),
            t,
            title_text: Default::default(),
            main_rect_draw_info: Default::default(),
            index_off: 0,
            vertex_off: 0,
            held: false,
        }
    }

    #[inline(always)]
    pub fn set_position(
        &mut self,
        position: Vec2,
    )
    {
        self.position = position;
    }

    #[inline(always)]
    pub fn update<FontHash>(
        &mut self,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        font: FontHash,
        _cursor_in_this_window: bool,
    ) -> bool
        where 
            FontHash: Clone + Eq + Hash,
    {
        if self.title_text.is_none() {
            self.title_text = Some(text_renderer
                .render(&[text_segment(self.title.as_str(), font)], false, 1.0)
                .unwrap_or(Default::default()));
        }
        let title_text = self.title_text.as_ref().unwrap();
        let main_rect = Rect::from_position_size(
            vec2(0.0, 0.0),
            vec2(title_text.text_width * style.font_scale, style.calc_item_height()),
            0.0,
        );
        if main_rect != self.main_rect {
            self.main_rect = main_rect;
            return true
        }
        false
    }

    #[inline(always)]
    pub fn triangulate<F>(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        mut tri: F,
    )
        where
            F: FnMut(&[[f32; 2]]) -> DrawInfo
    {
        points.clear();
        self.main_rect.to_points(&mut |p| { points.push(p.into()); });
        self.main_rect_draw_info = tri(&points);
    }

    #[inline(always)]
    pub fn render_commands<FontHash, F1, F2>(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        inv_aspect_ratio: f32,
        vertex_buf_id: BufferId,
        index_buf_id: BufferId,
        main_vert_mem_offset: u64,
        main_idx_mem_offset: u64,
        _allocate_vertices: &mut F1,
        _allocate_indices: &mut F2,
    ) -> Result<(), Error>
        where
            F1: FnMut(&mut RenderCommands, usize) -> Result<RingBufMem<Vertex>, Error>,
            F2: FnMut(&mut RenderCommands, usize) -> Result<RingBufMem<u32>, Error>,
    {
        let push_constants_vertex = push_constants_vertex(self.position + self.main_rect.size() * 0.5, inv_aspect_ratio);
        let push_constants_fragment = push_constants_fragment(style.widget_bg_col);
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
                    id: vertex_buf_id,
                    offset: main_vert_mem_offset,
                },
            ],
            DrawBufferInfo {
                id: index_buf_id,
                offset: main_idx_mem_offset,
            },
        )?;
        Ok(())
    }
}

macro_rules! impl_sliderable {
    ($($t:ty),+ $(,)?) => {
        $(
            impl Sliderable for $t {

                fn slide(&mut self, min: Self, max: Self, t: f32) {
                    *self = ((1.0 - t) * min as f32 + t * max as f32) as $t
                }

                fn calc_t(&self, min: Self, max: Self) -> f32 {
                    if *self >= max { return 1.0 }
                    if *self <= min { return 0.0 }
                    let d0 = max - min;
                    let d1 = max - self;
                    d1 as f32 / d0 as f32
                }
            }
        )+
    };
}

impl_sliderable!(
    i8, i16, i32, i64, i128,
    u8, u16, u32, u64, u128,
    f32, f64,
);
