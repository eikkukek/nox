use core::{
    f32::consts::*,
    marker::PhantomData
};

use rustc_hash::{FxHashMap, FxHashSet};

use nox::{
    mem::{vec_types::{GlobalVec, Vector}, Hashable},
    *,
};

use nox_font::{VertexTextRenderer, CombinedRenderedText};

use nox_geom::{
    shapes::*,
    *
};

use crate::*;

pub struct ComboBox<I, FontHash, Style> {
    offset: Vec2,
    widget_size: Vec2,
    content_size: Vec2,
    rounding: f32,
    focused_outline_width: f32,
    active_outline_width: f32,
    window_outline_width: f32,
    rect_vertex_range: VertexRange,
    focused_outline_vertex_range: VertexRange,
    active_outline_vertex_range: VertexRange,
    arrow_vertex_range: VertexRange,
    selectable_tags: FxHashMap<Hashable<f64>, (u64, SelectableTag<I, FontHash, Style>)>,
    active_tags: FxHashSet<Hashable<f64>>,
    prev_active_tags: GlobalVec<Hashable<f64>>,
    selected: Option<Hashable<f64>>,
    content_vertices: GlobalVec<Vertex>,
    content_indices: GlobalVec<u32>,
    content_rect_vertex_range: VertexRange,
    content_outline_vertex_range: VertexRange,
    combined_text: CombinedRenderedText<BoundedTextInstance, GlobalVec<BoundedTextInstance>>,
    last_triangulation: u64,
    arrow_rot: f32,
    flags: u32,
    _marker: PhantomData<(I, FontHash, Style)>,
}

impl<I, FontHash, Style> ComboBox<I, FontHash, Style>
    where 
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
{

    const CONTENT_REQUIRES_TRIANGULATION: u32 = 0x1;
    const WIDGET_REQUIRES_TRIANGULATION: u32 = 0x2;
    const CONTENTS_SHOWN: u32 = 0x4;
    const WIDGET_HELD: u32 = 0x8;
    const WIDGET_HOVERED: u32 = 0x10;
    const CLICKED: u32 = 0x20;

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            offset: Default::default(),
            widget_size: Default::default(),
            content_size: Default::default(),
            rounding: 0.0,
            focused_outline_width: 0.0,
            active_outline_width: 0.0,
            window_outline_width: 0.0,
            rect_vertex_range: Default::default(),
            focused_outline_vertex_range: Default::default(),
            arrow_vertex_range: Default::default(),
            active_outline_vertex_range: Default::default(),
            selectable_tags: FxHashMap::default(),
            active_tags: FxHashSet::default(),
            prev_active_tags: Default::default(),
            selected: None,
            content_vertices: Default::default(),
            content_indices: Default::default(),
            content_rect_vertex_range: Default::default(),
            content_outline_vertex_range: Default::default(),
            combined_text: Default::default(),
            last_triangulation: 0,
            arrow_rot: 0.0,
            flags: 0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn update_values<T>(
        &mut self,
        style: &Style,
        text_renderer: &mut VertexTextRenderer<FontHash>,
        mut f: impl FnMut(&mut ComboBoxBuilder<T, I, FontHash, Style>),
    ) {
        self.prev_active_tags.clear();
        for &widget in &self.active_tags {
            self.prev_active_tags.push(widget);
        }
        self.active_tags.clear();
        let item_pad_outer = style.item_pad_outer();
        let item_pad_inner = style.item_pad_inner();
        let text_box_height = style.calc_text_box_height_from_text_height(style.calc_font_height(text_renderer));
        let widget_off =
            self.offset +
            vec2(
                item_pad_inner.x,
                text_box_height + item_pad_inner.y,
            );
        let mut builder = ComboBoxBuilder {
            combo_box: self,
            text_renderer,
            style,
            widget_off,
            min_width: 0.0,
            _marker: PhantomData,
        };
        f(&mut builder);
        let widget_off = builder.widget_off;
        let min_width = builder.min_width + item_pad_inner.x + style.collapse_symbol_scale() + item_pad_outer.x;
        let widget_size = vec2(min_width, text_box_height);
        let widget_requires_triangulation =
            self.widget_size != widget_size ||
            self.focused_outline_width != style.focused_widget_outline_width() ||
            self.active_outline_width != style.active_widget_outline_width() ||
            self.rounding != style.rounding();
        self.widget_size = widget_size;
        self.focused_outline_width = style.focused_widget_outline_width();
        self.active_outline_width = style.active_widget_outline_width();
        self.flags |= Self::WIDGET_REQUIRES_TRIANGULATION * widget_requires_triangulation as u32;
        let content_offset = self.offset + vec2(0.0, widget_size.y + item_pad_outer.y);
        let content_size = vec2(min_width, widget_off.y - content_offset.y + item_pad_outer.y);
        let content_requires_triangulation =
            content_size != self.content_size ||
            self.window_outline_width != style.window_outline_width() ||
            self.rounding != style.rounding();
        self.rounding = style.rounding();
        self.window_outline_width = style.window_outline_width();
        self.content_size = content_size;
        self.flags |= Self::CONTENT_REQUIRES_TRIANGULATION * content_requires_triangulation as u32;
        let min_width = min_width - style.collapse_symbol_scale() - item_pad_inner.x;
        for tag in &self.active_tags {
            let (_, tag) = self.selectable_tags.get_mut(tag).unwrap();
            tag.override_width(min_width);
        }
    }

    #[inline(always)]
    fn content_requires_triangulation(&self) -> bool {
        self.flags & Self::CONTENT_REQUIRES_TRIANGULATION == Self::CONTENT_REQUIRES_TRIANGULATION
    }

    #[inline(always)]
    fn widget_requires_triangulation(&self) -> bool {
        self.flags & Self::WIDGET_REQUIRES_TRIANGULATION == Self::WIDGET_REQUIRES_TRIANGULATION
    }

    #[inline(always)]
    fn contents_shown(&self) -> bool {
        self.flags & Self::CONTENTS_SHOWN == Self::CONTENTS_SHOWN
    }

    #[inline(always)]
    fn widget_held(&self) -> bool {
        self.flags & Self::WIDGET_HELD == Self::WIDGET_HELD
    }

    #[inline(always)]
    fn widget_hovered(&self) -> bool {
        self.flags & Self::WIDGET_HOVERED == Self::WIDGET_HOVERED
    }

    #[inline(always)]
    fn clicked(&self) -> bool {
        self.flags & Self::CLICKED == Self::CLICKED
    }

    #[inline(always)]
    fn activate_tag(
        &mut self,
        label: &str,
    ) -> (&mut SelectableTag<I, FontHash, Style>, Hashable<f64>)
    {
        let mut id = Hashable((label as *const str).addr() as f64);
        while !self.active_tags.insert(id) {
            id.0 += 0.01;
        }
        let (last_triangulation, tag) = self.selectable_tags
            .entry(id)
            .or_insert((0, SelectableTag::new()));
        if *last_triangulation < self.last_triangulation {
            self.flags |= Self::CONTENT_REQUIRES_TRIANGULATION;
        }
        (tag, id)
    }
}

pub struct ComboBoxBuilder<'a, 'b, T, I, FontHash: UiFontHash, Style> {
    combo_box: &'a mut ComboBox<I, FontHash, Style>,
    text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
    style: &'a Style,
    widget_off: Vec2,
    min_width: f32,
    _marker: PhantomData<T>,
}

impl<'a, 'b, T: Eq, I, FontHash, Style> ComboBoxBuilder<'a, 'b, T, I, FontHash, Style>
    where
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
{

    pub fn item(&mut self, value: &mut T, target: T, label: &str) {
        let (tag, id) = self.combo_box.activate_tag(label);
        tag.set_label(label, self.text_renderer, self.style);
        let size = tag.calc_size(self.style, self.text_renderer);
        let clicked = tag.update_value(value, target);
        tag.set_offset(self.widget_off);
        if clicked {
            self.combo_box.selected = Some(id);
        }
        self.widget_off.y += size.y + self.style.item_pad_inner().y;
        self.min_width = self.min_width.max(size.x);
    }
}

impl<I, FontHash, Style> Widget<I, FontHash, Style> for ComboBox<I, FontHash, Style>
    where
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
{

    fn get_offset(&self) -> Vec2 {
        self.offset
    }

    fn set_offset(
        &mut self,
        offset: Vec2,
    ) {
        self.offset = offset;
    }

    fn calc_size(
        &mut self,
        _style: &Style,
        _text_renderer: &mut VertexTextRenderer<FontHash>,
    ) -> Vec2 {
        self.widget_size
    }

    fn status<'a>(
        &'a self,
        nox: &Nox<I>,
        style: &Style,
        window_pos: Vec2,
        cursor_pos: Vec2,
    ) -> WidgetStatus<'a>
    {
        for active_tag in &self.active_tags {
            let (_, tag) = &self.selectable_tags[active_tag];
            match tag.status(nox, style, window_pos, cursor_pos) {
                WidgetStatus::Active | WidgetStatus::Hovered(_) => {
                    return WidgetStatus::Active
                },
                WidgetStatus::Inactive => {}
            }
        }
        let content_active = self.contents_shown() && BoundingRect::from_position_size(
            window_pos + self.offset + vec2(0.0, self.widget_size.y),
            self.content_size,
        ).is_point_inside(cursor_pos);
        if self.widget_held() || content_active {
            WidgetStatus::Active
        } else if self.widget_hovered() {
            WidgetStatus::Hovered(None)
        } else {
            WidgetStatus::Inactive
        }
    }

    fn update(
        &mut self,
        nox: &mut Nox<I>,
        style: &Style,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        window_size: Vec2,
        window_pos: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        cursor_in_other_widget: bool,
        window_moving: bool,
        collect_text: &mut dyn FnMut(&nox_font::RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult
    {
        self.prev_active_tags.retain(|v| !self.active_tags.contains(v));
        self.combined_text.clear();
        for tag in &self.prev_active_tags {
            let (_, widget) = &self.selectable_tags[tag];
            widget.hide(&mut self.content_vertices);
        }
        let item_pad_inner = style.item_pad_inner();
        let (min_bounds, max_bounds) = calc_bounds(window_pos, self.offset, window_size);
        let bounded_instance = BoundedTextInstance {
            add_scale: vec2(1.0, 1.0),
            min_bounds,
            max_bounds,
            color: if self.contents_shown() || self.widget_held() {
                style.active_text_col()
            } else if self.widget_hovered() {
                style.focused_text_col()
            } else {
                style.inactive_text_col()
            },
        };
        if let Some(selected) = self.selected {
            let (_, tag) = self.selectable_tags.get_mut(&selected).unwrap();
            collect_text(
                tag.label_text(),
                self.offset + item_pad_inner,
                bounded_instance,
            );
            self.selected = None;
        }
        let error_margin = style.cursor_error_margin();
        let error_margin_2 = error_margin + error_margin;
        let bounding_rect = BoundingRect::from_position_size(
            window_pos + self.offset - vec2(error_margin, error_margin),
            self.widget_size + vec2(error_margin_2, error_margin_2)
        );
        let cursor_in_widget = bounding_rect.is_point_inside(cursor_pos);
        let mouse_pressed = nox.was_mouse_button_pressed(MouseButton::Left);
        self.flags &= !Self::WIDGET_HOVERED;
        if self.widget_held() {
            if nox.was_mouse_button_released(MouseButton::Left) {
                self.flags &= !Self::WIDGET_HELD;
                if cursor_in_widget {
                    self.flags ^= Self::CONTENTS_SHOWN;
                }
            }
        } else if cursor_in_widget {
            self.flags |= Self::WIDGET_HOVERED;
            if mouse_pressed {
                self.flags |= Self::WIDGET_HELD;
            }
        }
        let content_offset = self.offset + vec2(0.0, self.widget_size.y);
        let cursor_in_contents = BoundingRect::from_position_size(
            window_pos + content_offset,
            self.content_size,
        ).is_point_inside(cursor_pos);
        if !window_moving && self.clicked() && !self.widget_held() && !cursor_in_contents {
            self.flags &= !Self::CONTENTS_SHOWN;
        }
        self.flags &= !Self::CLICKED;
        if mouse_pressed {
            self.flags |= Self::CLICKED;
        }
        let contents_shown = self.contents_shown();
        let mut active_widget = None;
        let mut hovered_widget = None;
        for (i, tag) in self.active_tags.iter().enumerate() {
            let tag = self.selectable_tags.get_mut(tag).unwrap();
            match tag.1.status(nox, style, window_pos, cursor_pos) {
                WidgetStatus::Active => active_widget = Some(i),
                WidgetStatus::Hovered(_) => hovered_widget = Some(i),
                WidgetStatus::Inactive => {}
            }
        }
        let font_scale = style.font_scale();
        for (i, tag) in self.active_tags.iter().enumerate() {
            let tag = self.selectable_tags.get_mut(tag).unwrap();
            let result = tag.1.update(
                nox, style, text_renderer, window_size,
                window_pos, cursor_pos, delta_cursor_pos, contents_shown,
                if let Some(idx) = active_widget {
                    idx != i
                } else {
                    other_widget_active
                },
                if let Some(idx) = hovered_widget {
                    idx != i
                } else {
                    cursor_in_other_widget
                },
                window_moving,
                &mut |text, offset, bounded_instance| {
                    self.combined_text.add_text(text, offset / font_scale, bounded_instance).unwrap();
                }
            );
            if tag.1.clicked() {
                self.flags &= !Self::CONTENTS_SHOWN;
            }
            self.flags |= Self::CONTENT_REQUIRES_TRIANGULATION * result.requires_triangulation as u32;
        }
        if self.content_requires_triangulation() {
            self.last_triangulation += 1;
            self.content_vertices.clear();
            self.content_indices.clear();
            let mut points = GlobalVec::new();
            let mut helper_points = GlobalVec::new();
            let mut indices_usize = GlobalVec::new();
            let mut tri = |points: &[[f32; 2]]| {
                let vertex_off = self.content_vertices.len();
                earcut::earcut(&points, &[], false,
                    &mut self.content_vertices, &mut indices_usize
                ).unwrap();
                VertexRange::new(vertex_off..self.content_vertices.len())
            };
            rect(
                Default::default(),
                self.content_size,
                self.rounding,
            ).to_points(&mut |p| { points.push(p.into()); });
            outline_points(&points, self.focused_outline_width, false,
                &mut |p| { helper_points.push(p.into()); }
            );
            self.content_outline_vertex_range = tri(&helper_points);
            self.content_rect_vertex_range = tri(&points);
            points.clear();
            helper_points.clear();
            let last_triangulation = self.last_triangulation + 1;
            for tag in &self.active_tags {
                let tag = self.selectable_tags.get_mut(tag).unwrap();
                tag.1.triangulate(
                    &mut points,
                    &mut helper_points,
                    &mut tri
                );
                tag.0 = last_triangulation;
                points.clear();
                helper_points.clear();
            }
            self.last_triangulation = last_triangulation;
            self.content_indices.append_map(&indices_usize, |&i| i as u32);
            self.flags &= !Self::CONTENT_REQUIRES_TRIANGULATION;
        }
        if contents_shown {
            let offset = content_offset;
            set_vertex_params(&mut self.content_vertices, self.content_outline_vertex_range,
                offset, style.window_outline_col(),
            );
            set_vertex_params(&mut self.content_vertices, self.content_rect_vertex_range,
                offset, style.hover_window_bg_col(),
            );
            for tag in &self.active_tags {
                let tag = self.selectable_tags.get_mut(tag).unwrap();
                tag.1.set_vertex_params(style, &mut self.content_vertices);
            }
        }
        let requires_triangulation = self.widget_requires_triangulation();
        self.flags &= !Self::WIDGET_REQUIRES_TRIANGULATION;
        if self.contents_shown() {
            self.arrow_rot = (self.arrow_rot + FRAC_PI_2 * style.animation_speed() * nox.delta_time_secs_f32()).clamp(0.0, FRAC_PI_2);
        } else {
            self.arrow_rot = (self.arrow_rot - FRAC_PI_2 * style.animation_speed() * nox.delta_time_secs_f32()).clamp(0.0, FRAC_PI_2);
        }
        UpdateResult {
            requires_triangulation,
            cursor_in_widget: cursor_in_this_window && !other_widget_active && !cursor_in_other_widget &&
                (self.widget_hovered() || (self.contents_shown() && cursor_in_contents))
        }
    }

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        helper_points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    ) {
        rect(
            Default::default(),
            self.widget_size,
            self.rounding,
        ).to_points(&mut |p| { points.push(p.into()); });
        outline_points(points, self.focused_outline_width, false,
            &mut |p| { helper_points.push(p.into()); }
        );
        self.focused_outline_vertex_range = tri(&helper_points);
        helper_points.clear();
        outline_points(points, self.active_outline_width, false,
            &mut |p| { helper_points.push(p.into()); }
        );
        self.active_outline_vertex_range = tri(&helper_points);
        self.rect_vertex_range = tri(&points);
        points.clear();
        points.append(&[Default::default(); 3]);
        self.arrow_vertex_range = tri(&points);
        points.clear();
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        vertices: &mut [Vertex],
    ) {
        let offset = self.offset;
        let contents_shown = self.contents_shown();
        if self.widget_held() && !contents_shown {
            set_vertex_params(vertices, self.active_outline_vertex_range,
                offset, style.active_widget_outline_col()
            );
        } else {
            hide_vertices(vertices, self.active_outline_vertex_range);
        }
        if self.widget_hovered() && !contents_shown {
            set_vertex_params(vertices, self.focused_outline_vertex_range, offset,
                style.focused_widget_outline_col()
            );
        } else {
            hide_vertices(vertices, self.focused_outline_vertex_range);
        }
        set_vertex_params(vertices, self.rect_vertex_range, offset, style.widget_bg_col());
        let (scale, color) = 
            if self.contents_shown() {
                (
                    style.collapse_symbol_scale(),
                    style.inactive_text_col(),
                )
            }
            else if self.widget_held() {
                (
                    style.focused_collapse_symbol_scale(),
                    style.active_text_col()
                )
            }
            else if self.widget_hovered() {
                (
                    style.focused_collapse_symbol_scale(),
                    style.focused_text_col(),
                )
            } else {
                (
                    style.collapse_symbol_scale(),
                    style.inactive_text_col(),
                )
            };
        let rot = self.arrow_rot;
        let size = self.widget_size;
        let offset = offset + vec2(size.x - style.item_pad_outer().x, size.y * 0.5);
        vertices[self.arrow_vertex_range.start()] = Vertex {
            pos: vec2(0.5, 0.0).rotated(rot) * scale,
            offset: offset,
            color,
        };
        vertices[self.arrow_vertex_range.start() + 1] = Vertex {
            pos: vec2(-0.5, 0.5).rotated(rot) * scale,
            offset: offset,
            color,
        };
        vertices[self.arrow_vertex_range.start() + 2] = Vertex {
            pos: vec2(-0.5, -0.5).rotated(rot) * scale,
            offset: offset,
            color,
        };
    }

    fn render_commands(
        &self,
        _render_commands: &mut RenderCommands,
        _style: &Style,
        _base_pipeline_id: GraphicsPipelineId,
        _text_pipeline_id: GraphicsPipelineId,
        _vertex_buffer: &mut RingBuf,
        _index_buffer: &mut RingBuf,
        _window_pos: Vec2,
        _inv_aspect_ratio: f32,
        _unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, Style>>, Error>
    {
        if self.contents_shown() {
            Ok(Some(self))
        } else {
            Ok(None)
        }
    }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    ) {
        hide_vertices(vertices, self.rect_vertex_range);
        hide_vertices(vertices, self.active_outline_vertex_range);
        hide_vertices(vertices, self.focused_outline_vertex_range);
        hide_vertices(vertices, self.arrow_vertex_range);
    }
}

impl<I, FontHash, Style> HoverContents<I, FontHash, Style> for ComboBox<I, FontHash, Style>
    where 
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
{

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<(), Error> {
        let vertex_count = self.content_vertices.len();
        let index_count = self.content_indices.len();
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vertex_count)?
        };
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, index_count)?
        };
        unsafe {
            self.content_vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vertex_count);
            self.content_indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), index_count);
        }
        render_commands.bind_pipeline(base_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            window_pos,
            vec2(1.0, 1.0),
            inv_aspect_ratio,
            unit_scale,
        );
        let pc_fragment = base_push_constants_fragment(
            vec2(f32::MIN, f32::MIN),
            vec2(f32::MAX, f32::MAX),
        );
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            DrawInfo {
                index_count: index_count as u32,
                ..Default::default()
            },
            [
                DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset)
            ],
            DrawBufferInfo::new(index_buffer.id(), idx_mem.offset)
        )?;
        render_commands.bind_pipeline(text_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            window_pos, vec2(style.font_scale(), style.font_scale()),
            inv_aspect_ratio, unit_scale,
        );
        render_text(render_commands,
            self.combined_text
                .iter()
                .map(|(&c, (t, b))| (c, t, b.as_slice())),
            pc_vertex, vertex_buffer, index_buffer,
        )?;
        Ok(())
    }
}
