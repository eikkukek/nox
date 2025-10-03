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
    pub fn calc_text_size<FontHash>(
        &mut self,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        font: &FontHash,
    ) -> Vec2
        where
            FontHash: Clone + Eq + Hash, 
    {
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(self.title.as_str(), font)], false, 5.0).unwrap_or_default());
        vec2(title_text.text_width, title_text.font_height)
    }

    #[inline(always)]
    pub fn update<FontHash>(
        &mut self,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        font: &FontHash,
        _cursor_in_this_window: bool,
    ) -> bool
        where 
            FontHash: Clone + Eq + Hash,
    {
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(self.title.as_str(), font)], false, 5.0).unwrap_or_default());
        let main_rect = Rect::from_position_size(
            vec2(0.0, 0.0),
            vec2(style.calc_text_box_width(title_text.text_width), style.calc_item_height(title_text.font_height)),
            style.rounding,
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
    pub fn render_commands<FontHash>(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        inv_aspect_ratio: f32,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_vertex_offset: u64,
        window_index_offset: u64,
        base_pipeline: GraphicsPipelineId,
        text_pipeline: GraphicsPipelineId,
    ) -> Result<(), Error>
    {
        let vertex_buffer_id = vertex_buffer.id();
        let index_buffer_id = index_buffer.id();
        let pc_vertex = push_constants_vertex(self.position, inv_aspect_ratio);
        let pc_fragment = push_constants_fragment(style.widget_bg_col);
        render_commands.bind_pipeline(base_pipeline)?;
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                value_as_bytes(&pc_vertex).unwrap()
            } else {
                value_as_bytes(&pc_fragment).unwrap()
            }
        })?;
        render_commands.draw_indexed(
            self.main_rect_draw_info,
            [
                DrawBufferInfo {
                    id: vertex_buffer_id,
                    offset: window_vertex_offset,
                },
            ],
            DrawBufferInfo {
                id: index_buffer_id,
                offset: window_index_offset,
            },
        )?;
        let title_text = self.title_text.as_ref().unwrap();
        let text_pc_vertex = text_push_constants_vertex(self.position + style.item_pad_inner, inv_aspect_ratio, style.font_scale);
        let text_pc_fragment = push_constants_fragment(style.text_col);
        render_commands.bind_pipeline(text_pipeline)?;
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                value_as_bytes(&text_pc_vertex).unwrap()
            } else {
                value_as_bytes(&text_pc_fragment).unwrap()
            }
        })?;
        for text in &title_text.text {
            let vert_mem = unsafe {
                vertex_buffer.allocate(render_commands, text.trigs.vertices.len())?
            };
            let vert_off_mem = unsafe {
                vertex_buffer.allocate(render_commands, text.offsets.len())?
            };
            let idx_mem = unsafe {
                index_buffer.allocate(render_commands, text.trigs.indices.len())?
            }; 
            unsafe {
                text.trigs.vertices
                    .as_ptr()
                    .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), text.trigs.vertices.len());
                text.offsets
                    .as_ptr()
                    .copy_to_nonoverlapping(vert_off_mem.ptr.as_ptr(), text.offsets.len());
                text.trigs.indices
                    .as_ptr()
                    .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), text.trigs.indices.len());
            }
            render_commands.draw_indexed(
                DrawInfo {
                    index_count: text.trigs.indices.len() as u32,
                    instance_count: text.offsets.len() as u32,
                    ..Default::default()
                },
                [
                    DrawBufferInfo {
                        id: vertex_buffer_id,
                        offset: vert_mem.offset,
                    },
                    DrawBufferInfo {
                        id: vertex_buffer_id,
                        offset: vert_off_mem.offset,
                    },
                ],
                DrawBufferInfo {
                    id: index_buffer_id,
                    offset: idx_mem.offset,
                }
            )?;
        }
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
