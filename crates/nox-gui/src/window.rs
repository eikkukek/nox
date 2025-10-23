use core::{
    hash::Hash,
    str::FromStr,
    marker::PhantomData,
    f32::consts::FRAC_PI_2,
    fmt::Write,
};

use nox::{
    mem::vec_types::{GlobalVec, Vector},
    *,
};

use rustc_hash::FxHashMap;

use compact_str::CompactString;

use nox_font::{VertexTextRenderer, text_segment, RenderedText, CombinedRenderedText};

use nox_geom::{
    shapes::*, *
};

use crate::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveWidget {
    Slider(u32),
    Button(u32),
    Checkbox(u32),
    ColorPicker(u32),
    InputText(u32),
    DragValue(u32),
}

struct HoverWindow {
    text: CompactString,
    rendered_text: CombinedRenderedText<BoundedTextInstance, GlobalVec<BoundedTextInstance>>,
    rect: Rect,
    vertices: GlobalVec<Vertex>,
    indices: GlobalVec<u32>,
    position: Vec2,
}

impl HoverWindow {

    fn new() -> Self {
        Self {
            text: Default::default(),
            rendered_text: Default::default(),
            rect: Default::default(),
            vertices: Default::default(),
            indices: Default::default(),
            position: Default::default(),
        }
    }

    fn update<FontHash>(
        &mut self,
        style: &impl WindowStyle<FontHash>,
        text_renderer: &mut VertexTextRenderer<FontHash>,
        cursor_pos: Vec2,
        text: &str,
    ) -> bool
        where
            FontHash: Clone + Eq + Hash,
    {
        let mut rect = self.rect;
        rect.rounding = style.rounding();
        if text != self.text {
            self.rendered_text.clear();
            let text = text_renderer.render(
                &[text_segment(text, style.font_regular())], false, 0.0
            ).unwrap_or_default();
            self.rendered_text.add_text(
                &text,
                vec2(0.0, 0.0),
                BoundedTextInstance {
                    add_scale: vec2(1.0, 1.0),
                    min_bounds: vec2(f32::MIN, f32::MIN),
                    max_bounds: vec2(f32::MAX, f32::MAX),
                    color: style.text_col(),
                }
            ).unwrap();
            rect.max = style.calc_text_box_size(&text);
        }
        if rect != self.rect {
            self.rect = rect;
            return self.triangulate();
        }
        self.position = cursor_pos - vec2(self.rect.max.x, 0.0);
        return false
    }

    fn triangulate(&mut self) -> bool {
        self.vertices.clear();
        self.indices.clear();
        let mut points = GlobalVec::new();
        let mut indices_usize = GlobalVec::new();
        self.rect.to_points(&mut |p| { points.push(p.into()); });
        if !earcut::earcut(&points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
            return false
        }
        self.indices.append_map(&indices_usize, |&v| v as u32);
        true
    }

    fn set_vertex_params<FontHash>(
        &mut self,
        style: &impl WindowStyle<FontHash>,
    ) {
        let vertex_sample = self.vertices[0];
        if vertex_sample.color != style.window_bg_col() {
            let target_color = style.window_bg_col();
            for vertex in &mut self.vertices {
                vertex.color = target_color;
            }
        }
    }

    fn render_commands<FontHash>(
        &self,
        render_commands: &mut RenderCommands,
        style: & impl WindowStyle<FontHash>,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        inv_aspect_ratio: f32,
        unit_scale: f32,
    ) -> Result<(), Error>
    {
        let vert_count = self.vertices.len();
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_count)?
        };
        let idx_count = self.indices.len();
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
        render_commands.bind_pipeline(base_pipeline_id)?;
        let pc_vertex = push_constants_vertex(self.position, vec2(1.0, 1.0), inv_aspect_ratio, unit_scale);
        render_commands.push_constants(|_| unsafe {
            pc_vertex.as_bytes()
        })?;
        render_commands.draw_indexed(
            DrawInfo {
                index_count: self.indices.len() as u32,
                ..Default::default()
            },
            [
                DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset),
            ],
            DrawBufferInfo::new(index_buffer.id(), idx_mem.offset)
        )?;
        render_commands.bind_pipeline(text_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            self.position + style.item_pad_inner(),
            vec2(style.font_scale(), style.font_scale()),
            inv_aspect_ratio,
            unit_scale,
        );
        render_text(
            render_commands,
            self.rendered_text.iter().map(|(c, (t, b))| (*c, t, b.as_slice())),
            pc_vertex, vertex_buffer, index_buffer
        )?;
        Ok(())
    }
}

struct CollapsedWidgets {
    title: CompactString,
    title_text: RenderedText,
    offset: Vec2,
    symbol_vertex_range: VertexRange,
    rotation: f32,
    flags: u32,
}

impl CollapsedWidgets {

    const COLLAPSED: u32 = 0x1;
    const HOVERED: u32 = 0x2;

    #[inline(always)]
    fn new() -> Self {
        Self {
            title: Default::default(),
            title_text: Default::default(),
            offset: Default::default(),
            symbol_vertex_range: Default::default(),
            rotation: 0.0,
            flags: Self::COLLAPSED,
        }
    }

    #[inline(always)]
    fn collapsed(&self) -> bool {
        self.flags & Self::COLLAPSED == Self::COLLAPSED
    }

    #[inline(always)]
    fn hovered(&self) -> bool {
        self.flags & Self::HOVERED == Self::HOVERED
    }

    #[inline(always)]
    fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }

    #[inline(always)]
    fn set_title<FontHash>(&mut self, style: &impl WindowStyle<FontHash>, text_renderer: &mut VertexTextRenderer<FontHash>, title: &str)
        where 
            FontHash: Clone + Eq + Hash,
    {
        if self.title != title {
            self.title = CompactString::new(title);
            self.title_text = text_renderer.render(
                &[text_segment(&self.title, style.font_regular())], false, 0.0 
            ).unwrap_or_default();
        }
    }

    #[inline(always)]
    fn update<I, FontHash>(
        &mut self,
        nox: &Nox<I>,
        window_pos: Vec2,
        cursor_pos: Vec2,
        style: &impl WindowStyle<FontHash>,
        mut collect_text: impl FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> f32
        where
            I: Interface,
            FontHash: Clone + Eq + Hash,
    {
        let item_pad_outer = style.item_pad_outer();
        let collapse_scale = style.collapse_symbol_scale();
        let text_size = style.calc_text_size(&self.title_text);
        let offset = self.offset;
        let bounding_rect = BoundingRect::from_position_size(
            window_pos + offset,
            vec2(collapse_scale + item_pad_outer.x + text_size.x, text_size.y)
        );
        self.flags &= !Self::HOVERED;
        self.flags |= Self::HOVERED * bounding_rect.is_point_inside(cursor_pos) as u32;
        if nox.was_mouse_button_pressed(MouseButton::Left) && self.hovered() {
            self.flags ^= Self::COLLAPSED;
        }
        if self.collapsed() {
            self.rotation = (self.rotation - FRAC_PI_2 * style.animation_speed() * nox.delta_time_secs_f32()).clamp(0.0, FRAC_PI_2);
        } else {
            self.rotation = (self.rotation + FRAC_PI_2 * 8.0 * nox.delta_time_secs_f32()).clamp(0.0, FRAC_PI_2);
        }
        collect_text(&self.title_text, offset + vec2(collapse_scale + style.item_pad_inner().x, 0.0), BoundedTextInstance {
            add_scale: vec2(1.0, 1.0),
            min_bounds: vec2(f32::MIN, f32::MIN),
            max_bounds: vec2(f32::MAX, f32::MAX),
            color: if self.hovered() {
                style.focused_text_col()
            } else {
                style.text_col()
            }
        });
        offset.x + collapse_scale + text_size.x + item_pad_outer.x
    }

    #[inline(always)]
    fn set_vertex_params<FontHash>(&self, style: &impl WindowStyle<FontHash>, vertices: &mut [Vertex]) {
        let rotation = self.rotation;
        let (scale, color) = 
            if self.hovered() {
                (
                    style.focused_collapse_symbol_scale(),
                    style.focused_text_col(),
                )
            } else {
                (
                    style.collapse_symbol_scale(),
                    style.text_col(),
                )
            };
        let offset = self.offset + vec2(0.0, style.calc_text_height(&self.title_text) * 0.5);
        vertices[self.symbol_vertex_range.start()] = Vertex {
            pos: vec2(0.5, 0.0).rotated(rotation) * scale,
            offset: offset,
            color,
        };
        vertices[self.symbol_vertex_range.start() + 1] = Vertex {
            pos: vec2(-0.5, 0.5).rotated(rotation) * scale,
            offset: offset,
            color,
        };
        vertices[self.symbol_vertex_range.start() + 2] = Vertex {
            pos: vec2(-0.5, -0.5).rotated(rotation) * scale,
            offset: offset,
            color,
        };
    }

    #[inline(always)]
    fn hide(&self, vertices: &mut [Vertex]) {
        hide_vertices(vertices, self.symbol_vertex_range);
    }
}

pub(crate) struct Window<I, FontHash, Style>
{
    main_rect: Rect,
    title_bar_rect: Rect,
    main_rect_vertex_range: VertexRange,
    title_bar_vertex_range: VertexRange,
    focused_outline_vertex_range: VertexRange,
    outline_vertex_range: VertexRange,
    title_outline_vertex_range: VertexRange,
    main_draw_info: DrawInfo,
    position: Vec2,
    title: CompactString,
    title_text: Option<RenderedText>,
    combined_text: Option<CombinedRenderedText<BoundedTextInstance, GlobalVec<BoundedTextInstance>>>,
    vertices: Option<GlobalVec<Vertex>>,
    indices: GlobalVec<u32>,
    buttons: FxHashMap<u32, (u64, Button<I, FontHash, Style>)>,
    sliders: FxHashMap<u32, (u64, Slider<I, FontHash, Style>)>,
    checkboxs: FxHashMap<u32, (u64, Checkbox<I, FontHash, Style>)>,
    color_pickers: FxHashMap<u32, (u64, ColorPicker<I, FontHash, Style>)>,
    input_texts: FxHashMap<u32, (u64, InputText<DefaultText, I, FontHash, Style>)>,
    drag_values: FxHashMap<u32, (u64, DragValue<DefaultText, I, FontHash, Style>)>,
    active_widgets: Option<GlobalVec<ActiveWidget>>,
    prev_active_widgets: Option<GlobalVec<ActiveWidget>>,
    collapsing_widgets: FxHashMap<u32, (u64, CollapsedWidgets)>,
    active_collapsing_widgets: GlobalVec<u32>,
    prev_active_collapsing_widgets: GlobalVec<u32>,
    hover_window: Option<HoverWindow>,
    last_triangulation: u64,
    last_frame: u64,
    min_width: f32,
    min_height: f32,
    focused_outline_width: f32,
    outline_width: f32,
    distance_from_edge: Vec2,
    flags: u32,
    _marker: PhantomData<Style>,
}

impl<I, FontHash, Style> Window<I, FontHash, Style>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
{

    const RENDERABLE: u32 = 0x1;
    const REQUIRES_TRIANGULATION: u32 = 0x2;
    const CURSOR_IN_WINDOW: u32 = 0x4;
    const HELD: u32 = 0x8;
    const RESIZE_LEFT: u32 = 0x10;
    const RESIZE_RIGHT: u32 = 0x20;
    const RESIZE_TOP: u32 = 0x40;
    const RESIZE_BOTTOM: u32 = 0x80;
    const RESIZE_BLOCKED_COL: u32 = 0x100;
    const RESIZE_BLOCKED_ROW: u32 = 0x200;
    const HOVER_WINDOW_ACTIVE: u32 = 0x400;
    const APPEARING: u32 = 0x800;

    pub(crate) fn new(
        title: &str,
        position: [f32; 2],
        size: [f32; 2],
    ) -> Self
    {
        Self {
            main_rect: rect(Default::default(), size, 0.0),
            title_bar_rect: Default::default(),
            main_rect_vertex_range: Default::default(),
            title_bar_vertex_range: Default::default(),
            focused_outline_vertex_range: Default::default(),
            title_outline_vertex_range: Default::default(),
            outline_vertex_range: Default::default(),
            main_draw_info: Default::default(),
            position: position.into(),
            title: title.into(),
            title_text: None,
            combined_text: Some(CombinedRenderedText::new()),
            vertices: Some(Default::default()),
            indices: Default::default(),
            buttons: FxHashMap::default(),
            sliders: FxHashMap::default(),
            checkboxs: FxHashMap::default(),
            color_pickers: FxHashMap::default(),
            input_texts: FxHashMap::default(),
            drag_values: FxHashMap::default(),
            active_widgets: Some(Default::default()),
            prev_active_widgets: Some(Default::default()),
            collapsing_widgets: FxHashMap::default(),
            active_collapsing_widgets: Default::default(),
            prev_active_collapsing_widgets: Default::default(),
            hover_window: Some(HoverWindow::new()),
            last_triangulation: 0,
            last_frame: 0,
            min_width: 0.0,
            min_height: 0.0,
            focused_outline_width: 0.0,
            outline_width: 0.0,
            distance_from_edge: Default::default(),
            flags: Self::REQUIRES_TRIANGULATION | Self::APPEARING,
            _marker: PhantomData,
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

    #[inline(always)]
    fn cursor_in_window(&self) -> bool {
        self.flags & Self::CURSOR_IN_WINDOW == Self::CURSOR_IN_WINDOW
    }

    #[inline(always)]
    fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    fn resize_left(&self) -> bool {
        self.flags & Self::RESIZE_LEFT == Self::RESIZE_LEFT
    }

    #[inline(always)]
    fn resize_right(&self) -> bool {
        self.flags & Self::RESIZE_RIGHT == Self::RESIZE_RIGHT
    }

    #[inline(always)]
    fn resize_top(&self) -> bool {
        self.flags & Self::RESIZE_TOP == Self::RESIZE_TOP
    }

    #[inline(always)]
    fn resize_bottom(&self) -> bool {
        self.flags & Self::RESIZE_BOTTOM == Self::RESIZE_BOTTOM
    }

    #[inline(always)]
    fn resize_nw(&self) -> bool {
        self.resize_top() && self.resize_left()
    }

    #[inline(always)]
    fn resize_ne(&self) -> bool {
        self.resize_top() && self.resize_right()
    }

    #[inline(always)]
    fn resize_sw(&self) -> bool {
        self.resize_bottom() && self.resize_left()
    }

    #[inline(always)]
    fn resize_se(&self) -> bool {
        self.resize_bottom() && self.resize_right()
    }

    #[inline(always)]
    fn resize_blocked_col(&self) -> bool {
        self.flags & Self::RESIZE_BLOCKED_COL == Self::RESIZE_BLOCKED_COL
    }

    #[inline(always)]
    fn resize_blocked_row(&self) -> bool {
        self.flags & Self::RESIZE_BLOCKED_ROW == Self::RESIZE_BLOCKED_ROW
    }

    #[inline(always)]
    fn appearing(&self) -> bool {
        self.flags & Self::APPEARING == Self::APPEARING
    }

    #[inline(always)]
    fn any_resize(&self) -> bool {
        self.resize_left() ||
        self.resize_right() ||
        self.resize_top() ||
        self.resize_bottom()
    }

    #[inline(always)]
    fn hover_window_active(&self) -> bool {
        self.flags & Self::HOVER_WINDOW_ACTIVE == Self::HOVER_WINDOW_ACTIVE
    }

    #[inline(always)]
    fn get_widget(&self, widget: ActiveWidget) -> (u64, &dyn Widget<I, FontHash, Style>) {
        match widget {
            ActiveWidget::Slider(id) =>
                self.sliders.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            ActiveWidget::Button(id) =>
                self.buttons.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            ActiveWidget::Checkbox(id) =>
                self.checkboxs.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            ActiveWidget::ColorPicker(id) =>
                self.color_pickers.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            ActiveWidget::InputText(id) =>
                self.input_texts.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            ActiveWidget::DragValue(id) =>
                self.drag_values.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
        }
    }

    #[inline(always)]
    fn get_widget_mut(
        &mut self,
        widget: ActiveWidget
    ) -> (&mut u64, &mut dyn Widget<I, FontHash, Style>)
    {
        match widget {
            ActiveWidget::Slider(id) =>
                self.sliders.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            ActiveWidget::Button(id) =>
                self.buttons.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            ActiveWidget::Checkbox(id) =>
                self.checkboxs.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            ActiveWidget::ColorPicker(id) =>
                self.color_pickers.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style,>)
                ).unwrap(),
            ActiveWidget::InputText(id) =>
                self.input_texts.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            ActiveWidget::DragValue(id) =>
                self.drag_values.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
        }
    }

    #[inline(always)]
    pub fn set_last_frame(&mut self, frame: u64) {
        self.last_frame = frame;
    }

    #[inline(always)]
    pub fn last_frame(&self) -> u64 {
        self.last_frame
    }

    #[inline(always)]
    pub fn bounding_rect(&self, error_margin: f32) -> BoundingRect {
        let error_margin_2 = error_margin + error_margin;
        BoundingRect::from_position_size(
            self.position - vec2(error_margin, error_margin),
            self.main_rect.size() + vec2(error_margin_2, error_margin_2),
        )
    }

    #[inline(always)]
    pub fn begin(&mut self) {
        unsafe {
            self.prev_active_widgets
                .as_mut()
                .unwrap_unchecked()
                .move_from_vec(self.active_widgets.as_mut().unwrap_unchecked());
            self.prev_active_collapsing_widgets
                .move_from_vec(&mut self.active_collapsing_widgets);
        }
    }

    pub fn update(
        &mut self,
        nox: &mut Nox<I>,
        style: &Style,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        cursor_in_other_window: bool,
        win_size: Vec2,
        aspect_ratio: f32,
        unit_scale: f32,
    ) -> bool
        where 
            I: Interface,
            FontHash: Clone + Eq + Hash,
    {
        let mut cursor_in_this_window =
            !cursor_in_other_window &&
            self.bounding_rect(style.cursor_error_margin()).is_point_inside(cursor_pos);
        if cursor_in_this_window && style.override_cursor() && !self.any_resize() {
            nox.set_cursor(CursorIcon::Default);
        }
        let mut min_width: f32 = 0.0;
        let min_height = self.min_height;
        let mut cursor_in_some_widget = false;
        let window_size = self.main_rect.max;
        let window_pos = self.position;
        let (active_widgets, mut prev_active_widgets, mut vertices) = unsafe {(
            self.active_widgets.take().unwrap_unchecked(),
            self.prev_active_widgets.take().unwrap_unchecked(),
            self.vertices.take().unwrap_unchecked(),
        )};
        prev_active_widgets.retain(|v| !active_widgets.contains(v));
        self.prev_active_collapsing_widgets.retain(|v| !self.active_collapsing_widgets.contains(v));
        for collapsing_widgets in &self.prev_active_collapsing_widgets {
            let (_, collapsing_widgets) = &self.collapsing_widgets[collapsing_widgets];
            collapsing_widgets.hide(&mut vertices);
        }
        for &widget in &prev_active_widgets {
            let (_, widget) = self.get_widget_mut(widget);
            widget.hide(&mut vertices);
        }
        self.flags &= !(Self::CURSOR_IN_WINDOW | Self::HOVER_WINDOW_ACTIVE);
        self.prev_active_widgets = Some(prev_active_widgets);
        let item_pad_outer = style.item_pad_outer();
        let item_pad_inner = style.item_pad_inner();
        let font_scale = style.font_scale();
        let mut combined_text = unsafe {
            self.combined_text.take().unwrap_unchecked()
        };
        combined_text.clear();
        let mut hover_window = self.hover_window.take().unwrap();
        let mut active_widget = None;
        for (i, &widget) in active_widgets.iter().enumerate() {
            let (_, widget) = self.get_widget(widget);
            if widget.is_active(nox, style, window_pos, cursor_pos) {
                active_widget = Some(i);
                break
            }
        }
        let window_moving = self.held() || self.any_resize();
        for (i, &widget) in active_widgets.iter().enumerate() {
            let (_, widget) = self.get_widget_mut(widget);
            let UpdateResult {
                min_window_width,
                requires_triangulation,
                cursor_in_widget,
            } = widget.update(
                nox,
                style,
                text_renderer,
                window_size,
                window_pos,
                cursor_pos,
                delta_cursor_pos,
                cursor_in_this_window,
                if let Some(w) = active_widget {
                    w != i
                } else {
                    false
                },
                window_moving,
                &mut |text, offset, bounded_instance| {
                    combined_text.add_text(text, offset / font_scale, bounded_instance).unwrap();
                },
            );
            min_width = min_width.max(min_window_width);
            if cursor_in_widget && let Some(hover_text) = widget.hover_text() {
                hover_window.update(style, text_renderer, cursor_pos, hover_text);
                self.flags |= Self::HOVER_WINDOW_ACTIVE;
            }
            if requires_triangulation {
                self.flags |= Self::REQUIRES_TRIANGULATION;
            }
            cursor_in_some_widget |= cursor_in_widget;
        }
        for collapsing_widgets in &self.active_collapsing_widgets {
            let (_, collapsing_widgets) = self.collapsing_widgets.get_mut(collapsing_widgets).unwrap();
            collapsing_widgets.update(nox, window_pos, cursor_pos, style, |text, offset, bounded_text_instance| {
                combined_text.add_text(text, offset / font_scale, bounded_text_instance).unwrap();
            });
        }
        self.vertices = Some(vertices);
        cursor_in_this_window |= cursor_in_some_widget || active_widget.is_some();
        self.flags |= Self::CURSOR_IN_WINDOW * cursor_in_this_window as u32;
        self.active_widgets = Some(active_widgets);
        self.hover_window = Some(hover_window);
        let title_text = self.title_text.as_ref().unwrap();
        let title_add_scale = style.title_add_scale();
        min_width = min_width.max(
            style.calc_text_box_width_from_text_width(title_text.text_width * font_scale * title_add_scale) +
            item_pad_outer.x
        );
        self.min_width = min_width;
        if self.main_rect.max.x < min_width {
            self.main_rect.max.x = min_width;
        }
        let mut main_rect_max = self.main_rect.max;
        let override_cursor = style.override_cursor();
        if self.held() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HELD;
            } else {
                self.position += delta_cursor_pos;
            }
        }
        if self.held() || self.appearing() {
            let norm_pos = pos_to_norm_pos(self.position, unit_scale, aspect_ratio);
            self.distance_from_edge = vec2(norm_pos.x * win_size.x, norm_pos.y * win_size.y);
            self.flags &= !Self::APPEARING;
        }
        if self.resize_left() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_LEFT;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_col() {
                    if cursor_pos.x <= self.position.x {
                        self.flags &= !Self::RESIZE_BLOCKED_COL;
                    }
                } else {
                    let delta_width = cursor_pos.x - self.position.x;
                    let new_width = main_rect_max.x - delta_width;
                    if new_width < min_width {
                        self.position.x += main_rect_max.x - min_width;
                        main_rect_max.x = min_width;
                        self.flags |= Self::RESIZE_BLOCKED_COL;
                    } else {
                        main_rect_max.x = new_width;
                        self.position.x += delta_width;
                    }
                }
            }
        }
        if self.resize_right() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_RIGHT;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_col() {
                    if cursor_pos.x - self.position.x >= min_width {
                        self.flags &= !Self::RESIZE_BLOCKED_COL;
                    }
                } else {
                    let new_width = cursor_pos.x - self.position.x;
                    if new_width < min_width {
                        main_rect_max.x = min_width;
                        self.flags |= Self::RESIZE_BLOCKED_COL;
                    } else {
                        main_rect_max.x = new_width;
                    }
                }
            }
        }
        if self.resize_top() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_TOP;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_row() {
                    if cursor_pos.y <= self.position.y {
                        self.flags &= !Self::RESIZE_BLOCKED_ROW;
                    }
                }
                else {
                    let delta_height = cursor_pos.y - self.position.y;
                    let new_height = main_rect_max.y - delta_height;
                    if new_height < min_height {
                        self.position.y += main_rect_max.y - min_height;
                        main_rect_max.y = min_height;
                        self.flags |= Self::RESIZE_BLOCKED_ROW;
                    } else {
                        main_rect_max.y = new_height;
                        self.position.y = cursor_pos.y;
                    }
                } 
            }
        }
        if self.resize_bottom() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_BOTTOM;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_row() {
                    if cursor_pos.y - self.position.y >= min_height {
                        self.flags &= !Self::RESIZE_BLOCKED_ROW;
                    }
                } else {
                    let new_height = cursor_pos.y - self.position.y;
                    if new_height < self.min_height {
                        main_rect_max.y = min_height;
                        self.flags |= Self::RESIZE_BLOCKED_ROW;
                    } else {
                        main_rect_max.y = new_height;
                    }
                }
            }
        }
        if !self.held() && !self.any_resize() && cursor_in_this_window && !cursor_in_some_widget {
            let mut flags = self.flags;
            flags &= !Self::RESIZE_BLOCKED_COL;
            flags &= !Self::RESIZE_BLOCKED_ROW;
            let mouse_pressed = nox.was_mouse_button_pressed(MouseButton::Left);
            let error_margin = style.cursor_error_margin();
            if cursor_pos.x >= self.position.x - error_margin &&
                cursor_pos.x <= self.position.x + error_margin
            {
                flags |= Self::RESIZE_LEFT;
            }
            if cursor_pos.x >= self.position.x + self.main_rect.max.x - error_margin &&
                cursor_pos.x <= self.position.x + self.main_rect.max.x + error_margin
            {
                flags |= Self::RESIZE_RIGHT;
            }
            if cursor_pos.y >= self.position.y - error_margin * 0.5 &&
                cursor_pos.y <= self.position.y + error_margin * 0.5
            {
                flags |= Self::RESIZE_TOP;
            }
            if cursor_pos.y >= self.position.y + self.main_rect.max.y - error_margin &&
                cursor_pos.y <= self.position.y + self.main_rect.max.y + error_margin
            {
                flags |= Self::RESIZE_BOTTOM;
            }
            self.flags = flags;
            if !self.any_resize()
            {
                if BoundingRect
                    ::from_position_size(self.position, self.title_bar_rect.max)
                    .is_point_inside(cursor_pos)
                {
                    self.flags |= Self::HELD * mouse_pressed as u32;
                }
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            }
            else if override_cursor {
                if self.resize_nw() {
                    nox.set_cursor(CursorIcon::NwResize);
                }
                else if self.resize_ne() {
                    nox.set_cursor(CursorIcon::NeResize);
                }
                else if self.resize_sw() {
                    nox.set_cursor(CursorIcon::SwResize);
                }
                else if self.resize_se() {
                    nox.set_cursor(CursorIcon::SeResize);
                }
                else {
                    if self.resize_left() {
                        nox.set_cursor(CursorIcon::WResize);
                    }
                    if self.resize_right() {
                        nox.set_cursor(CursorIcon::EResize);
                    }
                    if self.resize_top() {
                        nox.set_cursor(CursorIcon::NResize);
                    }
                    if self.resize_bottom() {
                        nox.set_cursor(CursorIcon::SResize);
                    }
                }
            }
            self.flags &=
                !((Self::RESIZE_LEFT | Self::RESIZE_RIGHT | Self::RESIZE_TOP | Self::RESIZE_BOTTOM) *
                    !mouse_pressed as u32
                );
        }
        let mut title_bar_rect = self.title_bar_rect;
        title_bar_rect.max.x = self.main_rect.max.x;
        title_bar_rect.max.y = style.calc_text_box_height_from_text_height(
            title_text.row_height * font_scale * 1.5
        );
        title_bar_rect.rounding = style.rounding();
        combined_text
            .add_text(
                self.title_text.as_ref().unwrap(),
                vec2(item_pad_outer.x, item_pad_inner.y) / (font_scale * title_add_scale),
                BoundedTextInstance {
                    add_scale: vec2(title_add_scale, title_add_scale),
                    min_bounds: self.position,
                    max_bounds: self.position + title_bar_rect.max,
                    color: style.text_col(),
                }
            )
            .unwrap();
        self.combined_text = Some(combined_text);
        if main_rect_max.y < min_height {
            main_rect_max.y = min_height;
        }
        let requires_triangulation =
            (style.rounding() != self.main_rect.rounding ||
            self.focused_outline_width != style.focused_outline_width() ||
            self.outline_width != style.outline_width() ||
            main_rect_max != self.main_rect.max ||
            self.title_bar_rect != title_bar_rect
        ) as u32;
        self.flags |= Self::REQUIRES_TRIANGULATION * requires_triangulation;
        self.main_rect.rounding = style.rounding();
        self.main_rect.max = main_rect_max;
        self.title_bar_rect = title_bar_rect;
        self.outline_width = style.outline_width();
        self.focused_outline_width = style.focused_outline_width();
        self.title_bar_rect = title_bar_rect;
        cursor_in_this_window || self.any_resize()
    }

    #[inline(always)]
    pub fn refresh_position(&mut self, aspect_ratio: f32, unit_scale: f32, win_size: Vec2) {
        // pos = (2.0 * orig_pos - 1.0) * aspect_ratio.x / unit_scale   | * unit scale
        // pos * unit_scale = (2.0 * orig_pos - 1.0) * aspect_ratio.x   | / aspect_ratio.x
        // pos * unit_scale / aspect_ratio.x = 2.0 * orig_pos - 1.0     | + 1.0 
        // pos * unit_scale / aspect_ratio.x + 1.0 = orig_pos * 2.0     | / 2.0     
        // orig_pos = (pos * unit_scale / aspect_ratio.x + 1.0) / 2.0
        if !self.held() && !self.resize_left() && !self.resize_top() {
            let distance_from_edge = self.distance_from_edge;
            let dist = vec2(distance_from_edge.x / win_size.x, distance_from_edge.y / win_size.y);
            self.position = norm_pos_to_pos(dist, unit_scale, aspect_ratio);
        }
        let mut norm_pos = self.position * unit_scale;
        norm_pos.x /= aspect_ratio;
        norm_pos = (norm_pos + vec2(1.0, 1.0)) * 0.5;
        let mut norm_size = self.main_rect.max * unit_scale;
        norm_size.x /= aspect_ratio;
        norm_size = norm_size * 0.5;
        if norm_size.x >= 1.0 || norm_size.y >= 1.0 {
            let mut new_size = norm_size.clamp(vec2(0.0, 0.0), vec2(1.0, 1.0));
            new_size *= 2.0;
            new_size.x *= aspect_ratio;
            new_size /= unit_scale;
            if new_size.x >= self.min_width && new_size.y >= self.min_height {
                self.main_rect.max = new_size;
                norm_size = new_size * unit_scale;
                norm_size.x /= aspect_ratio;
                norm_size = norm_size * 0.5;
                self.flags |= Self::REQUIRES_TRIANGULATION;
            }
        }
        if norm_size.x < 1.0 && norm_size.y < 1.0 && (norm_pos.x < 0.0 || norm_pos.y < 0.0 ||
            norm_pos.x + norm_size.x >= 1.0 || norm_pos.y + norm_size.y >= 1.0)
        {
            norm_pos = norm_pos.clamp(vec2(0.0, 0.0), vec2(1.0 - norm_size.x, 1.0 - norm_size.y));
            let new_pos = norm_pos_to_pos(norm_pos, unit_scale, aspect_ratio);
            self.position = new_pos;
        } 
        self.distance_from_edge = vec2(norm_pos.x * win_size.x, norm_pos.y * win_size.y);
    }

    #[inline(always)]
    pub fn triangulate(&mut self) {
        if self.requires_triangulation() {
            let new_triangulation = self.last_triangulation + 1;
            self.flags |= Self::RENDERABLE;
            let mut vertices = self.vertices.take().unwrap();
            vertices.clear();
            self.indices.clear();
            let mut points = GlobalVec::new();
            let mut indices_usize = GlobalVec::new();
            self.main_rect.to_points(&mut |p| { points.push(p.into()); });
            let mut outline_points = GlobalVec::new();
            nox_geom::shapes::outline_points(&points,
                self.focused_outline_width, false, &mut |p| { outline_points.push(p.into()); }
            );
            if !earcut::earcut(&outline_points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.focused_outline_vertex_range = VertexRange::new(0..vertices.len());
            outline_points.clear();
            nox_geom::shapes::outline_points(&points,
                self.outline_width, false, &mut |p| { outline_points.push(p.into()); }
            );
            let mut vertex_begin = vertices.len();
            if !earcut::earcut(&outline_points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.outline_vertex_range = VertexRange::new(vertex_begin..vertices.len());
            vertex_begin = vertices.len();
            if !earcut::earcut(&points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.main_rect_vertex_range = VertexRange::new(vertex_begin..vertices.len());
            points.clear();
            self.title_bar_rect.to_points_partial_round(true, true, false, false,
                &mut |p| { points.push(p.into()); }
            );
            outline_points.clear();
            nox_geom::shapes::outline_points(&points,
                self.outline_width, false, &mut |p| { outline_points.push(p.into()); });
            vertex_begin = vertices.len();
            if !earcut::earcut(&outline_points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.title_outline_vertex_range = VertexRange::new(vertex_begin..vertices.len());
            vertex_begin = vertices.len();
            if !earcut::earcut(&points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.title_bar_vertex_range = VertexRange::new(vertex_begin..vertices.len());
            let active_widgets = self.active_widgets.take().unwrap();
            let mut flags = self.flags;
            for &widget in &active_widgets  {
                let (last_triangulation, widget) = self.get_widget_mut(widget);
                *last_triangulation = new_triangulation;
                points.clear();
                widget.triangulate(&mut points,
                    &mut |points: &[[f32; 2]]| {
                        let vertex_begin = vertices.len();
                        if !earcut::earcut(points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                            flags &= !Self::RENDERABLE;
                        }
                        VertexRange::new(vertex_begin..vertices.len())
                    }
                );
            }
            for collapsing_widgets in &mut self.active_collapsing_widgets {
                let (last_triangulation, collapsing_widgets) = self.collapsing_widgets.get_mut(collapsing_widgets).unwrap();
                *last_triangulation = new_triangulation;
                vertices.append(&[Default::default(); 3]);
                let n = vertices.len();
                indices_usize.append(&[n - 3, n - 2, n - 1]);
                collapsing_widgets.symbol_vertex_range = VertexRange::new(n - 3..n);
            }
            self.main_draw_info = DrawInfo {
                first_index: 0,
                index_count: indices_usize.len() as u32,
                ..Default::default()
            };
            self.active_widgets = Some(active_widgets);
            self.vertices = Some(vertices);
            self.flags = flags;
            self.flags &= !Self::REQUIRES_TRIANGULATION;
            self.indices.append_map(&indices_usize, |&i| i as u32);
            self.last_triangulation = new_triangulation;
        }
    }

    pub fn render_commands(
        &mut self,
        render_commands: &mut RenderCommands,
        style: &Style,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        get_custom_pipeline: &mut impl FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<(), Error>
    {
        if !self.renderable() {
            return Ok(())
        }
        let vert_total = unsafe {
            self.vertices
                .as_ref()
                .unwrap_unchecked().len()
        };
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_total)?
        };
        let idx_total = self.indices.len();
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, idx_total)?
        };
        let active_widgets = unsafe {
            self.active_widgets.take().unwrap_unchecked()
        };
        let mut vertices = unsafe {
            self.vertices.take().unwrap_unchecked()
        };
        for &widget in &active_widgets {
            let (_, widget) = self.get_widget_mut(widget);
            widget.set_vertex_params(style, &mut vertices);
        }
        for collapsing_widgets in &self.active_collapsing_widgets {
            let (_, collapsing_widgets) = self.collapsing_widgets.get_mut(collapsing_widgets).unwrap();
            collapsing_widgets.set_vertex_params(style, &mut vertices);
        }
        self.active_widgets = Some(active_widgets);
        let vertex_sample = vertices[self.main_rect_vertex_range.start()];
        if vertex_sample.color != style.window_bg_col() {
            let target_color = style.window_bg_col();
            for vertex in &mut vertices[self.main_rect_vertex_range.range()] {
                vertex.color = target_color;
            }
        }
        let vertex_sample = vertices[self.title_bar_vertex_range.start()];
        if vertex_sample.color != style.window_title_bar_col() {
            let target_color = style.window_title_bar_col();
            for vertex in &mut vertices[self.title_bar_vertex_range.range()] {
                vertex.color = target_color;
            }
        }
        let any_resize = self.any_resize();
        if self.cursor_in_window() || any_resize {
            let target_color = if any_resize || self.held() {
                style.window_outline_col()
            } else {
                style.focused_window_outline_col()
            };
            set_vertex_params(&mut vertices, self.focused_outline_vertex_range, vec2(0.0, 0.0), target_color);
            set_vertex_params(&mut vertices, self.title_outline_vertex_range, vec2(0.0, 0.0), target_color);
            hide_vertices(&mut vertices, self.outline_vertex_range);
        } else {
            let vertex_sample = vertices[self.focused_outline_vertex_range.start()];
            let target_color = ColorSRGBA::black(0.0);
            if vertex_sample.color != target_color {
                for vertex in &mut vertices[self.focused_outline_vertex_range.range()] {
                    vertex.color = target_color;
                }
            }
            let vertex_sample = vertices[self.outline_vertex_range.start()];
            let target_color = style.window_outline_col();
            if vertex_sample.color != target_color {
                for vertex in &mut vertices[self.outline_vertex_range.range()] {
                    vertex.color = target_color;
                }
            }
            let vertex_sample = vertices[self.title_outline_vertex_range.start()];
            if vertex_sample.color != style.window_outline_col() {
                let target_color = style.window_outline_col();
                for vertex in &mut vertices[self.title_outline_vertex_range.range()] {
                    vertex.color = target_color;
                }
            }
        }
        self.vertices = Some(vertices);
        unsafe {
            self.vertices
                .as_ref()
                .unwrap_unchecked()
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_total);
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_total);
        }
        render_commands.bind_pipeline(base_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            self.position,
            vec2(1.0, 1.0),
            inv_aspect_ratio,
            unit_scale,
        );
        let focused_outline_width = self.focused_outline_width;
        let pc_fragment = base_push_constants_fragment(
            self.position - vec2(focused_outline_width, focused_outline_width),
            self.position + self.main_rect.max + vec2(focused_outline_width, focused_outline_width),
        );
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            self.main_draw_info,
            [
                DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset),
            ],
            DrawBufferInfo {
                id: index_buffer.id(),
                offset: idx_mem.offset,
            },
        )?;
        let mut on_top_contents = None;
        let window_pos = self.position;
        for &widget in unsafe { self.active_widgets.as_ref().unwrap_unchecked() } {
            let (_, widget) = self.get_widget(widget);
            if let Some(contents) = widget.render_commands(
                render_commands,
                style,
                base_pipeline_id,
                text_pipeline_id,
                vertex_buffer,
                index_buffer,
                window_pos,
                inv_aspect_ratio,
                unit_scale,
                get_custom_pipeline,
            )? {
                on_top_contents = Some(contents);
            }
        }
        render_commands.bind_pipeline(text_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            self.position,
            vec2(style.font_scale(), style.font_scale()),
            inv_aspect_ratio,
            unit_scale,
        );
        render_text(render_commands,
            unsafe { self.combined_text
                .as_ref()
                .unwrap_unchecked()
                .iter()
                .map(|(&c, (t, b))| (c, t, b.as_slice()))
            },
            pc_vertex, vertex_buffer, index_buffer
        )?;
        if let Some(contents) = on_top_contents {
            contents.render_commands(
                render_commands,
                style,
                base_pipeline_id,
                text_pipeline_id,
                vertex_buffer,
                index_buffer,
                window_pos,
                inv_aspect_ratio,
                unit_scale,
                get_custom_pipeline,
            )?;
        }
        if self.hover_window_active() {
            let hover_window = unsafe { self.hover_window
                    .as_mut()
                    .unwrap_unchecked()
            };
            hover_window.set_vertex_params(style);
            hover_window.render_commands(
                render_commands,
                style,
                base_pipeline_id,
                text_pipeline_id,
                vertex_buffer,
                index_buffer,
                inv_aspect_ratio,
                unit_scale,
            )?;
        }
        Ok(())
    }
}

pub struct WindowContext<'a, 'b, I, FontHash, Style>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash, 
        Style: WindowStyle<FontHash>,
{
    style: &'a Style,
    window: &'a mut Window<I, FontHash, Style>,
    text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
    widget_off: Vec2,
    collapsed: bool,
}

impl<'a, 'b, I, FontHash, Style> WindowContext<'a, 'b, I, FontHash, Style>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
{

    pub(crate) fn new(
        title: &str,
        window: &'a mut Window<I, FontHash, Style>,
        style: &'a Style,
        text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
    ) -> Self {
        if title != window.title {
            window.title = title.into();
            window.title_text = None;
        }
        window.begin();
        let title_text = window.title_text.get_or_insert(text_renderer.render(
            &[text_segment(window.title.as_str(), &style.font_regular())],
            false,
            0.0,
        ).unwrap_or_default());
        Self {
            widget_off: vec2(
                style.item_pad_outer().x,
                style.calc_text_box_height_from_text_height(title_text.row_height * style.font_scale() * style.title_add_scale()) +
                    style.item_pad_outer().y,
            ),
            window,
            style,
            text_renderer,
            collapsed: false,
        }
    }

    pub(crate) fn new_collapsing(
        id: u32,
        title: &str,
        window: &'a mut Window<I, FontHash, Style>,
        style: &'a Style,
        text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
        widget_off: Vec2,
    ) -> Self {
        window.active_collapsing_widgets.push(id);
        let (last_triangulation, collapsing_widgets) = window.collapsing_widgets.entry(id).or_insert((0, CollapsedWidgets::new()));
        if *last_triangulation != window.last_triangulation {
            window.flags |= Window::<I, FontHash, Style>::REQUIRES_TRIANGULATION;
        }
        collapsing_widgets.set_title(style, text_renderer, title);
        collapsing_widgets.set_offset(widget_off);
        let collapsed = collapsing_widgets.collapsed();
        Self {
            widget_off: widget_off + vec2(style.item_pad_outer().x, style.calc_text_height(&collapsing_widgets.title_text) + style.item_pad_outer().y),
            window,
            style,
            text_renderer,
            collapsed,
        }
    }

    pub fn collapsing<F>(&mut self, id: u32, title: &str, mut f: F)
        where 
            F: FnMut(&mut WindowContext<I, FontHash, Style>)
    {
        if self.collapsed {
            return
        }
        let mut collapsing = WindowContext::new_collapsing(
            id, title, self.window, self.style, self.text_renderer,
            self.widget_off
        );
        f(&mut collapsing);
        self.widget_off.y = collapsing.widget_off.y;
    }

    pub fn update_button(
        &mut self,
        id: u32,
        title: &str,
    ) -> bool
    {
        if self.collapsed {
            return false
        }
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::Button(id));
        }
        let (last_triangulation, button) = self.window.buttons
            .entry(id)
            .or_insert((0, Button::new(title)));
        if *last_triangulation != self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash, Style>::REQUIRES_TRIANGULATION;
        }
        button.set_offset(self.widget_off);
        self.widget_off.y += button.calc_height(&self.style, self.text_renderer) +
            self.style.item_pad_outer().y;
        button.pressed()
    }

    pub fn update_slider<T: Sliderable>(
        &mut self,
        id: u32,
        title: &str,
        value: &mut T,
        min: T,
        max: T,
    )
    { 
        if self.collapsed {
            return
        }
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::Slider(id));
        }
        let (last_triangulation, slider) = self.window.sliders
            .entry(id)
            .or_insert((0, Slider::new(title)));
        if *last_triangulation != self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash, Style>::REQUIRES_TRIANGULATION;
        }
        if slider.held() {
            slider.quantized_t = value.slide_and_quantize_t(min, max, slider.t);
        } else {
            slider.t = value.calc_t(min, max);
            slider.quantized_t = slider.t;
        }
        slider.hover_text.clear();
        value
            .display(self.style, &mut slider.hover_text)
            .map_err(|e| {
                slider.hover_text.clear();
                write!(slider.hover_text, "{:?}", e).ok();
            }).ok();
        slider.set_offset(self.widget_off);
        self.widget_off.y += slider.calc_height(&self.style, self.text_renderer) +
            self.style.item_pad_outer().y;
    }

    pub fn update_checkbox(
        &mut self,
        id: u32,
        title: &str,
        value: &mut bool,
    ) -> bool
    {
        if self.collapsed {
            return false
        }
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::Checkbox(id));
        }
        let (last_triangulation, checkbox) = self.window.checkboxs
            .entry(id)
            .or_insert((0, Checkbox::new(title)));
        if *last_triangulation != self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash, Style>::REQUIRES_TRIANGULATION;
        }
        if checkbox.pressed() {
            *value = !*value;
        }
        checkbox.set_checked(*value);
        checkbox.set_offset(self.widget_off);
        self.widget_off.y += checkbox.calc_height(&self.style, self.text_renderer) +
            self.style.item_pad_outer().y;
        *value
    }

    pub fn update_color_picker<C: Color>(
        &mut self,
        id: u32,
        title: &str,
        value: &mut C,
    )
    {
        if self.collapsed {
            return
        }
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::ColorPicker(id));
        }
        let (last_triangulation, color_picker) = self.window.color_pickers
            .entry(id)
            .or_insert((0, ColorPicker::new(title)));
        if *last_triangulation != self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash, Style>::REQUIRES_TRIANGULATION;
        }
        if color_picker.picking() {
            *value = C::from_hsva(color_picker.calc_color(self.style));
        }
        else {
            color_picker.set_color(*value);
        }
        color_picker.set_offset(self.widget_off);
        self.widget_off.y += color_picker.calc_height(self.style, self.text_renderer) +
            self.style.item_pad_outer().y;
    }

    #[inline(always)]
    fn update_input_text_internal<T: core::fmt::Display + FromStr>(
        &mut self,
        id: u32,
        title: &str,
        value: &mut T,
        empty_input_prompt: &str,
        width_override: Option<f32>,
        skip_title: bool,
        center_text: bool,
        format_input: Option<fn(&mut dyn core::fmt::Write, &str) -> core::fmt::Result>
    )
    {
        if self.collapsed {
            return
        }
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::InputText(id));
        }
        let (last_triangulation, input_text) = self.window.input_texts
            .entry(id)
            .or_insert((0, InputText::new(title)));
        if *last_triangulation != self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash, Style>::REQUIRES_TRIANGULATION;
        }
        input_text.set_params(
            width_override, None, skip_title, center_text,
            empty_input_prompt, format_input
        );
        if input_text.active() {
            if let Some(v) = input_text.get_input() {
                *value = v;
            }
        } else {
            input_text.set_input(value);
        }
        input_text.set_offset(self.widget_off);
        self.widget_off.y += input_text.calc_height(self.style, self.text_renderer) +
            self.style.item_pad_outer().y;
    }

    #[inline(always)]
    pub fn update_input_text<T: core::fmt::Display + FromStr>(
        &mut self,
        id: u32,
        title: &str,
        value: &mut T,
        empty_input_prompt: &str,
        format_input: Option<fn(&mut dyn core::fmt::Write, &str) -> core::fmt::Result>
    )
    {
        self.update_input_text_internal(
            id,
            title, value, empty_input_prompt,
            None, false, false, format_input,
        );
    }
    
    #[inline(always)]
    pub fn update_input_text_with_width<T: core::fmt::Display + FromStr>(
        &mut self,
        id: u32,
        title: &str,
        value: &mut T,
        empty_input_prompt: &str,
        width_override: f32,
        skip_title: bool,
        center_text: bool,
        format_input: Option<fn(&mut dyn core::fmt::Write, &str) -> core::fmt::Result>
    )
    {
        self.update_input_text_internal(
            id,
            title, value, empty_input_prompt,
            Some(width_override), skip_title, center_text, format_input
        );
    }

    #[inline(always)]
    pub fn update_drag_value<T: Sliderable>(
        &mut self,
        id: u32,
        title: &str,
        value: &mut T,
        min: T,
        max: T,
        drag_speed: Option<f32>,
        min_width: f32,
        skip_title: bool,
        format_input: Option<fn(&mut dyn core::fmt::Write, &str) -> core::fmt::Result>,
    )
    {
        if self.collapsed {
            return
        }
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::DragValue(id));
        }
        let (last_triangulation, drag_value) = self.window.drag_values
            .entry(id)
            .or_insert((0, DragValue::new(title)));
        if *last_triangulation != self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash, Style>::REQUIRES_TRIANGULATION;
        }
        drag_value.set_input_params(self.style, min_width, skip_title, format_input);
        drag_value.calc_value(
            self.style, value, min, max,
            drag_speed.unwrap_or(self.style.default_value_drag_speed()),
        );
        drag_value.set_offset(self.widget_off);
        self.widget_off.y += drag_value.calc_height(self.style, self.text_renderer) +
            self.style.item_pad_outer().y;
    }
}

impl<'a, 'b, I, FontHash, Style> Drop for
        WindowContext<'a, 'b, I, FontHash, Style>
    where 
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
{
    fn drop(&mut self) {
        self.window.min_height = self.widget_off.y + self.style.item_pad_outer().y
    }
}
