use core::{
    ptr::NonNull,
    any::TypeId,
};

use ahash::{AHashMap, AHashSet};
use compact_str::CompactString;

use nox::{
    alloc::arena::Arena,
    mem::{
        vec::Vec32,
        alloc::LocalAllocExt,
    },
    win,
    gpu,
    error::*,
    or_flag,
};
use nox_font::{RenderedText, CombinedRenderedText, text_segment};
use nox_geom::{
    shapes::*, *
};

use crate::{
    collapsing_header::*,
    surface::*,
    *
};

pub struct WindowEndResult {
    pub cursor_in_window: bool,
}

type FnReactionMove = dyn FnMut(Option<NonNull<u8>>, &Arena) -> Option<NonNull<u8>>;

struct ReactionData {
    ty: TypeId,
    ptr: Option<NonNull<u8>>,
    move_fn: Box<FnReactionMove>,
    drop_fn: Box<dyn FnMut(Option<NonNull<u8>>)>,
}

unsafe impl Send for ReactionData {}
unsafe impl Sync for ReactionData {}

#[derive(Clone, Copy)]
pub struct Row {
    pub height: f32,
    pub height_halved: f32,
}

pub struct Window
{
    main_rect: Rect,
    title_bar_rect: Rect,
    main_rect_vertex_range: Option<VertexRange>,
    title_bar_vertex_range: Option<VertexRange>,
    focused_stroke_vertex_range: Option<VertexRange>,
    stroke_vertex_range: Option<VertexRange>,
    title_stroke_vertex_range: Option<VertexRange>,
    window_draw_info: gpu::IndexedDrawInfo,
    content_draw_info: gpu::IndexedDrawInfo,
    position: Vec2,
    pub title: CompactString,
    pub title_text: Option<RenderedText>,
    combined_text: CombinedRenderedText<BoundedTextInstance>,
    vertices: Vec32<Vertex>,
    indices: Vec32<u32>,
    text: Vec32<SharedText>,
    reactions: AHashMap<ReactionId, Reaction>,
    active_reactions: AHashSet<ReactionId>,
    prev_active_reactions: Vec32<ReactionId>,
    reaction_data: AHashMap<ReactionId, ReactionData>,
    reaction_text: AHashMap<ReactionId, (CompactString, SharedText)>,
    animated_bools: AHashMap<ReactionId, (f32, bool)>,
    collapsing_headers: AHashMap<CollapsingHeaderId, (u64, CollapsingHeader)>,
    active_collapsing_headers: AHashSet<CollapsingHeaderId>,
    prev_active_collapsing_headers: Vec32<CollapsingHeaderId>,
    painter_storage: PainterStorage,
    hover_window: HoverWindow,
    scroll_bar_vertices: Vec32<Vertex>,
    scroll_bar_indices: Vec32<u32>,
    ver_scroll_bar: VerScrollBar,
    hor_scroll_bar: HorScrollBar,
    last_triangulation: u64,
    last_frame: u64,
    widget_rect_max: Vec2,
    min_size: Vec2,
    scroll_y: f32,
    scroll_x: f32,
    widget_scroll_off: Vec2,
    focused_stroke_thickness: f32,
    stroke_thickness: f32,
    distance_from_edge: Vec2,
    reaction_data_alloc_0: Arena,
    reaction_data_alloc_1: Arena,
    flags: u32,
}

impl Window
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
    const RESIZEABLE: u32 = 0x1000;
    const CLAMP_HEIGHT: u32 = 0x2000;
    const CLAMP_WIDTH: u32 = 0x4000;
    const VER_SCROLL_BAR_VISIBLE: u32 = 0x8000;
    const HOR_SCROLL_BAR_VISIBLE: u32 = 0x0001_0000;
    const VER_SCROLL_BAR_RENDERABLE: u32 = 0x0002_0000;
    const HOR_SCROLL_BAR_RENDERABLE: u32 = 0x0004_0000;
    const CONTENT_HELD: u32 = 0x0008_0000;
    const USING_REACTION_DATA_ALLOC_1: u32 = 0x0010_0000;

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
            focused_stroke_vertex_range: Default::default(),
            title_stroke_vertex_range: Default::default(),
            stroke_vertex_range: Default::default(),
            window_draw_info: Default::default(),
            content_draw_info: Default::default(),
            position: position.into(),
            title: title.into(),
            title_text: None,
            combined_text: CombinedRenderedText::new(),
            vertices: Default::default(),
            indices: Default::default(),
            text: Default::default(),
            reactions: AHashMap::default(),
            active_reactions: AHashSet::default(),
            prev_active_reactions: Default::default(),
            reaction_data: AHashMap::default(),
            reaction_text: AHashMap::default(),
            animated_bools: AHashMap::default(),
            collapsing_headers: AHashMap::default(),
            active_collapsing_headers: Default::default(),
            prev_active_collapsing_headers: Default::default(),
            painter_storage: PainterStorage::new(1 << 14),
            hover_window: HoverWindow::new(),
            scroll_bar_vertices: Default::default(),
            scroll_bar_indices: Default::default(),
            ver_scroll_bar: VerScrollBar::new(),
            hor_scroll_bar: HorScrollBar::new(),
            last_triangulation: 0,
            last_frame: 0,
            widget_rect_max: vec2(0.0, 0.0),
            min_size: vec2(0.0, 0.0),
            scroll_y: 0.0,
            scroll_x: 0.0,
            widget_scroll_off: Default::default(),
            focused_stroke_thickness: 0.0,
            stroke_thickness: 0.0,
            distance_from_edge: Default::default(),
            reaction_data_alloc_0: Arena::new(1 << 20).unwrap(),
            reaction_data_alloc_1: Arena::new(1 << 20).unwrap(),
            flags:
                Self::REQUIRES_TRIANGULATION |
                Self::APPEARING |
                Self::RESIZEABLE |
                Self::CLAMP_HEIGHT |
                Self::CLAMP_WIDTH,
        }
    }

    #[inline(always)]
    fn activate_reaction<T: RefAddr>(
        &mut self,
        value: &T,
    ) -> &mut Reaction
    {
        let mut id = ReactionId(value.addr());
        while !self.active_reactions.insert(id) {
            id.0 = id.0.wrapping_add(1);
        }
        self.reactions.entry(id).or_insert_with(|| Reaction::new(id))
    }

    #[inline(always)]
    pub fn size(&self) -> Vec2 {
        self.main_rect.max
    }

    #[inline(always)]
    pub fn is_resizeable(&self) -> bool {
        self.flags & Self::RESIZEABLE == Self::RESIZEABLE
    }

    #[inline(always)]
    pub fn clamping_height(&self) -> bool {
        self.flags & Self::CLAMP_HEIGHT == Self::CLAMP_HEIGHT
    }

    #[inline(always)]
    pub fn clamping_width(&self) -> bool {
        self.flags & Self::CLAMP_WIDTH == Self::CLAMP_WIDTH
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
    fn ver_scroll_bar_visible(&self) -> bool {
        self.flags & Self::VER_SCROLL_BAR_VISIBLE == Self::VER_SCROLL_BAR_VISIBLE
    }

    #[inline(always)]
    fn hor_scroll_bar_visible(&self) -> bool {
        self.flags & Self::HOR_SCROLL_BAR_VISIBLE == Self::HOR_SCROLL_BAR_VISIBLE
    }

    #[inline(always)]
    fn ver_scroll_bar_renderable(&self) -> bool {
        self.flags & Self::VER_SCROLL_BAR_RENDERABLE == Self::VER_SCROLL_BAR_RENDERABLE
    }

    #[inline(always)]
    fn hor_scroll_bar_renderable(&self) -> bool {
        self.flags & Self::HOR_SCROLL_BAR_RENDERABLE == Self::HOR_SCROLL_BAR_RENDERABLE
    }

    #[inline(always)]
    fn content_held(&self) -> bool {
        self.flags & Self::CONTENT_HELD == Self::CONTENT_HELD
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
    fn using_reaction_data_alloc_1(&self) -> bool {
        self.flags & Self::USING_REACTION_DATA_ALLOC_1 == Self::USING_REACTION_DATA_ALLOC_1
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
        self.prev_active_collapsing_headers.clear();
        for &id in &self.active_collapsing_headers {
            self.prev_active_collapsing_headers.push(id);
        }
        self.prev_active_reactions.clear();
        for &id in &self.active_reactions {
            self.prev_active_reactions.push(id);
        }
        self.active_collapsing_headers.clear();
        self.active_reactions.clear();
        self.painter_storage.begin();
    }

    pub fn end(
        &mut self,
        rec: &mut RecordCmd<'_>,
        style: &UiStyle,
        cached_data: CachedUiData,
    ) -> Result<WindowEndResult>
    {
        let override_cursor = style.override_cursor;
        let mut cursor_in_this_window =
            !cached_data.cursor_in_other_window &&
            self.bounding_rect(style.cursor_error_margin)
                .is_point_inside(cached_data.cursor_pos);
        let mut title_bar_rect = self.title_bar_rect;

        let title_text = self.title_text.as_ref().unwrap();
        let font_scale = style.font_scale;
        let title_text_box_size = style.calc_text_box_size_from_text_size(vec2(
            title_text.text_width * font_scale * style.title_add_scale,
            title_text.row_height * font_scale * style.title_add_scale,
        ));
        title_bar_rect.max.y = title_text_box_size.y;
        let title_add_scale = style.title_add_scale;
        let size = self.size();
        let item_pad_outer = style.item_pad_outer;
        let item_pad_inner = style.item_pad_inner;
        let mut min_size = self.widget_rect_max.max(title_text_box_size + item_pad_outer);
        let mut widget_off = vec2(0.0, 0.0);
        self.flags &= !(Self::VER_SCROLL_BAR_VISIBLE | Self::HOR_SCROLL_BAR_VISIBLE);
        let mut delta_lines = cached_data.mouse_delta_lines;
        let mut delta_pixels = cached_data.mouse_delta_pixels;
        if !style.natural_scroll {
            delta_lines = (-delta_lines.0, -delta_lines.1);
            delta_pixels = (-delta_pixels.0, -delta_pixels.1);
        }
        if !self.clamping_height() {
            if min_size.y > size.y {
                self.flags |= Self::VER_SCROLL_BAR_VISIBLE;
                if !self.ver_scroll_bar.held() {
                    let unit_delta =
                        if delta_lines.1 != 0.0 {
                            delta_lines.1 * item_pad_outer.y * style.scroll_speed
                        } else {
                            delta_pixels.1 as f32 / style.pixels_per_unit * style.scroll_speed
                        };
                    self.scroll_y += unit_delta;
                }
                self.scroll_y = self.scroll_y.clamp(0.0, 1.0);
                widget_off.y = self.scroll_y * self.widget_rect_max.y;
                let delta = self.widget_rect_max.y - widget_off.y;
                if delta < size.y {
                    widget_off.y += delta - size.y;
                    self.scroll_y = widget_off.y / self.widget_rect_max.y;
                }
                widget_off.y = -widget_off.y;
            } else {
                self.scroll_y = 0.0;
                self.ver_scroll_bar.deactivate();
            }
            min_size.y = title_bar_rect.max.y + item_pad_outer.y;
        }
        if !self.clamping_width() {
            if min_size.x > size.x {
                self.flags |= Self::HOR_SCROLL_BAR_VISIBLE;
                if !self.hor_scroll_bar.held() {
                    let unit_delta =
                        if delta_lines.0 != 0.0 {
                            delta_lines.0 * item_pad_outer.y * style.scroll_speed
                        } else {
                            delta_pixels.0 as f32 / style.pixels_per_unit * style.scroll_speed
                        };
                    self.scroll_x += unit_delta;
                }
                self.scroll_x = self.scroll_x.clamp(0.0, 1.0);
                widget_off.x = self.scroll_x * self.widget_rect_max.x;
                let delta = self.widget_rect_max.x - widget_off.x;
                if delta < size.x {
                    widget_off.x += delta - size.x;
                    self.scroll_x = widget_off.x / self.widget_rect_max.x;
                }
                widget_off.x = -widget_off.x;
            } else {
                self.scroll_x = 0.0;
                self.hor_scroll_bar.deactivate();
            }
            min_size.x = 
                style.calc_text_box_width_from_text_width(
                    title_text.text_width * font_scale * title_add_scale
                ) + item_pad_outer.x;
        }
        let pos = self.position;
        self.prev_active_collapsing_headers.retain(|v| !self.active_collapsing_headers.contains(v));
        for collapsing_headers in &self.prev_active_collapsing_headers {
            let (_, collapsing_headers) = &self.collapsing_headers[collapsing_headers];
            collapsing_headers.hide(&mut self.vertices);
        }
        self.prev_active_reactions.retain(|v| !self.active_reactions.contains(v));
        for id in &self.prev_active_reactions {
            if let Some(mut data) = self.reaction_data.remove(id) {
                (data.drop_fn)(data.ptr);
            }
        }
        self.flags &= !(Self::CURSOR_IN_WINDOW | Self::HOVER_WINDOW_ACTIVE);
        self.combined_text.clear();
        let mut hover_blocked =
            !cursor_in_this_window ||
            self.ver_scroll_bar.held() ||
            self.hor_scroll_bar.held() ||
            self.content_held();
        let mouse_left_state = cached_data.mouse_button_left_state;
        if !self.held() && !self.any_resize() && !hover_blocked {
            if cursor_in_this_window {
                let mut flags = self.flags;
                flags &= !Self::RESIZE_BLOCKED_COL;
                flags &= !Self::RESIZE_BLOCKED_ROW;
                let error_margin = style.cursor_error_margin;
                if self.is_resizeable() {
                    if cached_data.cursor_pos.x >= self.position.x - error_margin &&
                        cached_data.cursor_pos.x <= self.position.x + error_margin
                    {
                        flags |= Self::RESIZE_LEFT;
                    }
                    if cached_data.cursor_pos.x >= self.position.x + self.main_rect.max.x - error_margin &&
                        cached_data.cursor_pos.x <= self.position.x + self.main_rect.max.x + error_margin
                    {
                        flags |= Self::RESIZE_RIGHT;
                    }
                    if cached_data.cursor_pos.y >= self.position.y - error_margin * 0.5 &&
                        cached_data.cursor_pos.y <= self.position.y + error_margin * 0.5
                    {
                        flags |= Self::RESIZE_TOP;
                    }
                    if cached_data.cursor_pos.y >= self.position.y + self.main_rect.max.y - error_margin &&
                        cached_data.cursor_pos.y <= self.position.y + self.main_rect.max.y + error_margin
                    {
                        flags |= Self::RESIZE_BOTTOM;
                    }
                }
                self.flags = flags;
                if !self.any_resize()
                {
                    if BoundingRect
                        ::from_position_size(self.position, self.title_bar_rect.max)
                        .is_point_inside(cached_data.cursor_pos)
                    {
                        hover_blocked = true;
                        or_flag!(self.flags, Self::HELD, mouse_left_state.pressed());
                    }
                    if override_cursor {
                        rec.set_cursor(win::CursorIcon::Default);
                    }
                }
                else {
                    hover_blocked = true;
                    if override_cursor {
                        if self.resize_nw() {
                            rec.set_cursor(win::CursorIcon::NwResize);
                        }
                        else if self.resize_ne() {
                            rec.set_cursor(win::CursorIcon::NeResize);
                        }
                        else if self.resize_sw() {
                            rec.set_cursor(win::CursorIcon::SwResize);
                        }
                        else if self.resize_se() {
                            rec.set_cursor(win::CursorIcon::SeResize);
                        }
                        else {
                            if self.resize_left() {
                                rec.set_cursor(win::CursorIcon::WResize);
                            }
                            if self.resize_right() {
                                rec.set_cursor(win::CursorIcon::EResize);
                            }
                            if self.resize_top() {
                                rec.set_cursor(win::CursorIcon::NResize);
                            }
                            if self.resize_bottom() {
                                rec.set_cursor(win::CursorIcon::SResize);
                            }
                        }
                    }
                }
                self.flags &=
                    !((Self::RESIZE_LEFT | Self::RESIZE_RIGHT | Self::RESIZE_TOP | Self::RESIZE_BOTTOM) *
                        !mouse_left_state.pressed() as u32
                    );
            }
        } else {
            hover_blocked = true;
        }
        let reaction_blocked = hover_blocked;
        let mut held_reaction = None;
        for &id in &self.active_reactions {
            let reaction = self.reactions.get_mut(&id).unwrap();
            if reaction.held() {
                held_reaction = Some(id);
                break;
            }
        }
        self.flags &= !(Self::HELD & (held_reaction.is_some() as u32) << Self::HELD.trailing_zeros());
        cursor_in_this_window |= held_reaction.is_some();
        self.flags ^= Self::USING_REACTION_DATA_ALLOC_1;
        let reaction_data_alloc =
            if self.using_reaction_data_alloc_1() {
                unsafe {
                    self.reaction_data_alloc_1.clear();
                }
                &self.reaction_data_alloc_1
            } else {
                unsafe {
                    self.reaction_data_alloc_0.clear();
                }
                &self.reaction_data_alloc_0
            };
        let scroll_bar_hovered = self.ver_scroll_bar.hovering() || self.hor_scroll_bar.hovering();
        for &id in &self.active_reactions {
            let reaction = self.reactions.get_mut(&id).unwrap();
            reaction.offset += widget_off;
            if reaction.animated_bool() &&
                let Some((t, value)) = self.animated_bools.get_mut(&id)
            {
                if *value {
                    *t = (*t + style.animation_speed * cached_data.delta_time).clamp(0.0, 1.0);
                } else {
                    *t = (*t - style.animation_speed * cached_data.delta_time).clamp(0.0, 1.0);
                }
            }
            if let Some(data) = self.reaction_data.get_mut(&id) {
                data.ptr = (data.move_fn)(data.ptr, reaction_data_alloc);
            }
            if let Some(text) = reaction.update(
                    &cached_data,
                    pos,
                    cursor_in_this_window,
                    reaction_blocked || scroll_bar_hovered ||
                    if let Some(held_id) = held_reaction {
                        held_id != id
                    } else {
                        false
                    },
                )
            {
                self.hover_window.update(rec, style, &cached_data, &text);
                self.flags |= Self::HOVER_WINDOW_ACTIVE;
            }
            if let Some(cursor_override) = reaction.take_cursor() &&
                override_cursor
            {
                rec.set_cursor(cursor_override);
            }
        }
        let window_moving = self.held() || self.any_resize();
        let content_area = 
        (
            pos + vec2(item_pad_inner.x, title_bar_rect.max.y + item_pad_inner.y),
            pos + self.main_rect.max - item_pad_inner
        );
        for collapsing_header in &self.active_collapsing_headers {
            let (_, collapsing_header) = self.collapsing_headers.get_mut(collapsing_header).unwrap();
            collapsing_header.offset += widget_off;
            let width = collapsing_header.update(
                &cached_data, pos, content_area, style,
                window_moving,
                |text, offset, bounded_text_instance| {
                    self.combined_text.add_text(text, offset / font_scale, bounded_text_instance);
                }
            );
            if self.clamping_width() {
                min_size.x = min_size.x.max(width);
            }
        }
        self.widget_scroll_off = widget_off;
        let ver_scroll_bar_width = self.ver_scroll_bar.calc_width(style);
        let hor_scroll_bar_height = self.hor_scroll_bar.calc_height(style);
        if self.ver_scroll_bar_visible() && !hover_blocked {
            min_size.x += ver_scroll_bar_width + item_pad_outer.x;
        }
        if self.hor_scroll_bar_visible() && !hover_blocked {
            min_size.y += hor_scroll_bar_height + item_pad_outer.y;
        }
        or_flag!(self.flags, Self::CURSOR_IN_WINDOW, cursor_in_this_window);
        if self.main_rect.max.x < min_size.x {
            self.main_rect.max.x = min_size.x;
        }
        let mut main_rect_max = self.main_rect.max;
        if self.held() {
            if !mouse_left_state.held() {
                self.flags &= !Self::HELD;
            } else {
                self.position += cached_data.delta_cursor_pos;
            }
        }
        if !self.is_resizeable() {
            self.flags &= !(Self::RESIZE_LEFT | Self::RESIZE_RIGHT | Self::RESIZE_TOP | Self::RESIZE_BOTTOM);
        }
        if self.held() || self.appearing() {
            let norm_pos = pos_to_norm_pos(self.position, cached_data.unit_scale, cached_data.aspect_ratio);
            self.distance_from_edge = vec2(
                norm_pos.x * cached_data.window_size.x,
                norm_pos.y * cached_data.window_size.y
            );
            self.flags &= !Self::APPEARING;
        }
        if self.resize_left() {
            if !mouse_left_state.held() {
                self.flags &= !Self::RESIZE_LEFT;
                if override_cursor {
                    rec.set_cursor(win::CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_col() {
                    if cached_data.cursor_pos.x <= self.position.x {
                        self.flags &= !Self::RESIZE_BLOCKED_COL;
                    }
                } else {
                    let delta_width = cached_data.cursor_pos.x - self.position.x;
                    let new_width = main_rect_max.x - delta_width;
                    if new_width < min_size.x {
                        self.position.x += main_rect_max.x - min_size.x;
                        main_rect_max.x = min_size.x;
                        self.flags |= Self::RESIZE_BLOCKED_COL;
                    } else {
                        main_rect_max.x = new_width;
                        self.position.x += delta_width;
                    }
                }
            }
        }
        if self.resize_right() {
            if !mouse_left_state.held() {
                self.flags &= !Self::RESIZE_RIGHT;
                if override_cursor {
                    rec.set_cursor(win::CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_col() {
                    if cached_data.cursor_pos.x - self.position.x >= min_size.x {
                        self.flags &= !Self::RESIZE_BLOCKED_COL;
                    }
                } else {
                    let new_width = cached_data.cursor_pos.x - self.position.x;
                    if new_width < min_size.x {
                        main_rect_max.x = min_size.x;
                        self.flags |= Self::RESIZE_BLOCKED_COL;
                    } else {
                        main_rect_max.x = new_width;
                    }
                }
            }
        }
        if self.resize_top() {
            if !mouse_left_state.held() {
                self.flags &= !Self::RESIZE_TOP;
                if override_cursor {
                    rec.set_cursor(win::CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_row() {
                    if cached_data.cursor_pos.y <= self.position.y {
                        self.flags &= !Self::RESIZE_BLOCKED_ROW;
                    }
                }
                else {
                    let delta_height = cached_data.cursor_pos.y - self.position.y;
                    let new_height = main_rect_max.y - delta_height;
                    if new_height < min_size.y {
                        self.position.y += main_rect_max.y - min_size.y;
                        main_rect_max.y = min_size.y;
                        self.flags |= Self::RESIZE_BLOCKED_ROW;
                    } else {
                        main_rect_max.y = new_height;
                        self.position.y = cached_data.cursor_pos.y;
                    }
                } 
            }
        }
        if self.resize_bottom() {
            if !mouse_left_state.held() {
                self.flags &= !Self::RESIZE_BOTTOM;
                if override_cursor {
                    rec.set_cursor(win::CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_row() {
                    if cached_data.cursor_pos.y - self.position.y >= min_size.y {
                        self.flags &= !Self::RESIZE_BLOCKED_ROW;
                    }
                } else {
                    let new_height = cached_data.cursor_pos.y - self.position.y;
                    if new_height < min_size.y {
                        main_rect_max.y = min_size.y;
                        self.flags |= Self::RESIZE_BLOCKED_ROW;
                    } else {
                        main_rect_max.y = new_height;
                    }
                }
            }
        }
        title_bar_rect.max.x = self.main_rect.max.x; 
        title_bar_rect.rounding = style.rounding; 
        let title_text = self.title_text.as_ref().unwrap();
        self.combined_text
            .add_text(
                title_text,
                vec2(item_pad_outer.x, item_pad_inner.y) / (font_scale * title_add_scale),
                BoundedTextInstance {
                    add_scale: vec2(title_add_scale, title_add_scale),
                    min_bounds: self.position,
                    max_bounds: self.position + title_bar_rect.max,
                    color:
                        if self.held() || self.any_resize() {
                            style.active_text_col
                        }
                        else if self.cursor_in_window() {
                            style.focused_text_col
                        } else {
                            style.inactive_text_col
                        },
                },
            );
        if main_rect_max.y < min_size.y {
            main_rect_max.y = min_size.y;
        }
        self.min_size = min_size;
        let mut triangulate_scroll_bars = false;
        let content_held = self.content_held();
        if self.ver_scroll_bar_visible() {
            let offset = vec2(title_bar_rect.max.x - item_pad_outer.x -
                ver_scroll_bar_width, title_bar_rect.max.y + item_pad_outer.y
            );
            let height = main_rect_max.y - offset.y - item_pad_outer.y - if self.hor_scroll_bar_visible() {
                hor_scroll_bar_height + item_pad_outer.y
            } else {
                0.0
            };
            let res = self.ver_scroll_bar.update(
                style, &cached_data,
                self.scroll_y, offset,
                pos, height,
                self.widget_rect_max.y,
                size.y,
                false,
                hover_blocked && !content_held && !self.hor_scroll_bar.held(),
            );
            triangulate_scroll_bars |= res.requires_triangulation;
            self.scroll_y = res.new_t;
        }
        if self.hor_scroll_bar_visible() {
            let offset = vec2(item_pad_outer.x, main_rect_max.y - item_pad_outer.y - hor_scroll_bar_height);
            let width = main_rect_max.x - offset.x - item_pad_outer.x - if self.ver_scroll_bar_visible() {
                ver_scroll_bar_width + item_pad_outer.x
            } else {
                0.0
            };
            let res = self.hor_scroll_bar.update(
                style, &cached_data,
                self.scroll_x, offset,
                pos, width,
                self.widget_rect_max.x,
                size.x,
                false,
                hover_blocked && !content_held && !self.ver_scroll_bar.held()
            );
            triangulate_scroll_bars |= res.requires_triangulation;
            self.scroll_x = res.new_t;
        }
        hover_blocked |= self.hor_scroll_bar.held() || self.ver_scroll_bar.held();
        if self.content_held() {
            if mouse_left_state.released() {
                self.flags &= !Self::CONTENT_HELD;
            } else if held_reaction.is_none() {
                if
                    !self.ver_scroll_bar_visible() &&
                    !self.hor_scroll_bar_visible() &&
                    !cached_data.delta_cursor_pos.is_zero()
                {
                    self.flags |= Self::HELD;
                } else {
                    if self.ver_scroll_bar_visible()  {
                        self.scroll_y -= cached_data.delta_cursor_pos.y / self.widget_rect_max.y;
                    }
                    if self.hor_scroll_bar_visible() {
                        self.scroll_x -= cached_data.delta_cursor_pos.x / self.widget_rect_max.x;
                    }
                }
            }
        } else if
            !hover_blocked &&
            cursor_in_this_window &&
            held_reaction.is_none() &&
            mouse_left_state.pressed()
        {
            self.flags |= Self::CONTENT_HELD;
        }
        if triangulate_scroll_bars {
            let mut points = Vec32::new();
            self.scroll_bar_vertices.clear();
            self.scroll_bar_indices.clear();
            self.flags |= Self::VER_SCROLL_BAR_RENDERABLE | Self::HOR_SCROLL_BAR_RENDERABLE;
            self.ver_scroll_bar.triangulate(&mut points, |points| {
                let vertex_offset = self.scroll_bar_vertices.len();
                if !earcut::earcut(
                        points, &[], false, &mut self.scroll_bar_vertices,
                        &mut self.scroll_bar_indices
                    )
                {
                    self.flags &= !Self::VER_SCROLL_BAR_RENDERABLE;
                }
                VertexRange::new(vertex_offset..self.scroll_bar_vertices.len())
            });
            points.clear();
            self.hor_scroll_bar.triangulate(&mut points, |points| {
                let vertex_offset = self.scroll_bar_vertices.len();
                if !earcut::earcut(
                        points, &[], false, &mut self.scroll_bar_vertices,
                        &mut self.scroll_bar_indices
                    )
                {
                    self.flags &= !Self::HOR_SCROLL_BAR_RENDERABLE;
                }
                VertexRange::new(vertex_offset..self.scroll_bar_vertices.len())
            });
        }
        for text in &self.text {
            let text = text.load();
            let offset = text.offset + widget_off;
            self.combined_text.add_text(
                &text.text,
                offset / font_scale,
                BoundedTextInstance {
                    add_scale: text.scale,
                    min_bounds: pos + (vec2(item_pad_inner.x, title_bar_rect.max.y + item_pad_inner.y)
                        .max(text.bounds.min + widget_off)
                    ),
                    max_bounds: pos + (main_rect_max - item_pad_inner).min(text.bounds.max + widget_off),
                    color: text.color, 
                }
            );
        }
        self.text.clear();
        let requires_triangulation =
            (style.rounding != self.main_rect.rounding ||
            self.focused_stroke_thickness != style.focused_window_stroke_thickness ||
            self.stroke_thickness != style.window_stroke_thickness ||
            main_rect_max != self.main_rect.max ||
            self.title_bar_rect != title_bar_rect
        ) as u32;
        self.flags |= Self::REQUIRES_TRIANGULATION * requires_triangulation;
        self.main_rect.rounding = style.rounding;
        self.main_rect.max = main_rect_max;
        self.title_bar_rect = title_bar_rect;
        self.stroke_thickness = style.window_stroke_thickness;
        self.focused_stroke_thickness = style.focused_window_stroke_thickness;
        self.title_bar_rect = title_bar_rect;
        let mut norm_size = self.main_rect.max * cached_data.unit_scale;
        norm_size.x /= cached_data.aspect_ratio;
        norm_size *= 0.5;
        self.painter_storage
            .end()
            .context("failed to update painter storage")?;
        Ok(WindowEndResult {
            cursor_in_window: cursor_in_this_window || self.any_resize(),
        })
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
        norm_size *= 0.5;
        if norm_size.x >= 1.0 || norm_size.y >= 1.0 {
            let mut new_size = norm_size.clamp(vec2(0.0, 0.0), vec2(1.0, 1.0));
            new_size *= 2.0;
            new_size.x *= aspect_ratio;
            new_size /= unit_scale;
            if new_size.x >= self.min_size.x && new_size.y >= self.min_size.y {
                self.main_rect.max = new_size;
                norm_size = new_size * unit_scale;
                norm_size.x /= aspect_ratio;
                norm_size *= 0.5;
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
            self.vertices.clear();
            self.indices.clear();
            let mut points = Vec32::new();
            self.main_rect.to_points(&mut |p| { points.push(p.into()); });
            let mut helper_points = Vec32::new();
            outline_points(&points,
                self.focused_stroke_thickness, false, &mut |p| { helper_points.push(p.into()); }
            );
            if !earcut::earcut(&helper_points, &[], false, &mut self.vertices, &mut self.indices) {
                self.flags &= !Self::RENDERABLE;
            }
            self.focused_stroke_vertex_range = VertexRange::new(0..self.vertices.len());
            helper_points.clear();
            outline_points(&points,
                self.stroke_thickness, false, &mut |p| { helper_points.push(p.into()); }
            );
            let mut vertex_begin = self.vertices.len();
            if !earcut::earcut(&helper_points, &[], false, &mut self.vertices, &mut self.indices) {
                self.flags &= !Self::RENDERABLE;
            }
            self.stroke_vertex_range = VertexRange::new(vertex_begin..self.vertices.len());
            vertex_begin = self.vertices.len();
            if !earcut::earcut(&points, &[], false, &mut self.vertices, &mut self.indices) {
                self.flags &= !Self::RENDERABLE;
            }
            self.main_rect_vertex_range = VertexRange::new(vertex_begin..self.vertices.len());
            points.clear();
            self.title_bar_rect.to_points_partial_round(true, true, false, false,
                &mut |p| { points.push(p.into()); }
            );
            helper_points.clear();
            outline_points(&points,
                self.stroke_thickness, false, &mut |p| { helper_points.push(p.into()); });
            vertex_begin = self.vertices.len();
            if !earcut::earcut(&helper_points, &[], false, &mut self.vertices, &mut self.indices) {
                self.flags &= !Self::RENDERABLE;
            }
            self.title_stroke_vertex_range = VertexRange::new(vertex_begin..self.vertices.len());
            vertex_begin = self.vertices.len();
            if !earcut::earcut(&points, &[], false, &mut self.vertices, &mut self.indices) {
                self.flags &= !Self::RENDERABLE;
            }
            self.title_bar_vertex_range = VertexRange::new(vertex_begin..self.vertices.len());
            self.window_draw_info = gpu::IndexedDrawInfo {
                first_index: 0,
                index_count: self.indices.len(),
                ..Default::default()
            };
            let first_index = self.indices.len();
            for collapsing_headers in &self.active_collapsing_headers {
                let (last_triangulation, collapsing_headers)
                    = self.collapsing_headers.get_mut(collapsing_headers).unwrap();
                *last_triangulation = new_triangulation;
                self.vertices.append(&[Default::default(); 3]);
                let n = self.vertices.len();
                self.indices.append(&[n - 3, n - 2, n - 1]);
                collapsing_headers.symbol_vertex_range = VertexRange::new(n - 3..n);
                self.vertices.append(&[Default::default(); 4]);
                let n = self.vertices.len();
                self.indices.append(&[
                    n - 4, n - 1, n - 3,
                    n - 3, n - 1, n - 2,
                ]);
                collapsing_headers.beam_vertex_range = VertexRange::new(n - 4..n);
            }
            self.flags &= !Self::REQUIRES_TRIANGULATION;
            self.last_triangulation = new_triangulation;
            self.content_draw_info = gpu::IndexedDrawInfo {
                first_index,
                index_count: self.indices.len() - first_index,
                ..Default::default()
            };
        }
        self.painter_storage.triangulate();
    } 

    pub fn draw(
        &mut self,
        cmd: &mut gpu::DrawCommands,
        rec: &mut RecordCmd<'_>,
        cached_data: CachedUiData,
        style: &UiStyle,
        sampler: gpu::Sampler,
    ) -> Result<()>
    {
        if !self.renderable() {
            return Ok(())
        }
        let vert_total = self.vertices.len();
        let vert_mem = rec.allocate_vertices(vert_total)?;
        let idx_total = self.indices.len();
        let idx_mem = rec.allocate_indices(idx_total)?;
        let vert_id = rec.vertex_buffer_id();
        let idx_id = rec.index_buffer_id();
        if self.ver_scroll_bar_visible() {
            self.ver_scroll_bar.set_vertex_params(style, &mut self.scroll_bar_vertices);
        }
        if self.hor_scroll_bar_visible() {
            self.hor_scroll_bar.set_vertex_params(style, &mut self.scroll_bar_vertices);
        }
        for collapsing_headers in &self.active_collapsing_headers {
            let (_, collapsing_headers) = self.collapsing_headers.get_mut(collapsing_headers).unwrap();
            collapsing_headers.set_vertex_params(style, &mut self.vertices);
        }
        color_vertices(&mut self.vertices, self.main_rect_vertex_range, style.window_bg_col);
        color_vertices(&mut self.vertices, self.title_bar_vertex_range, style.window_title_bar_col);
        let any_resize = self.any_resize();
        if self.cursor_in_window() || any_resize {
            let target_color = if any_resize || self.held() {
                style.window_stroke_col
            } else {
                style.focused_window_stroke_col
            };
            color_vertices(&mut self.vertices, self.focused_stroke_vertex_range, target_color);
            color_vertices(&mut self.vertices, self.title_stroke_vertex_range, target_color);
            hide_vertices(&mut self.vertices, self.stroke_vertex_range);
        } else {
            hide_vertices(&mut self.vertices, self.focused_stroke_vertex_range);
            color_vertices(&mut self.vertices, self.title_stroke_vertex_range, style.window_stroke_col);
            color_vertices(&mut self.vertices, self.stroke_vertex_range, style.window_stroke_col);
        }
        unsafe {
            self.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_total as usize);
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_total as usize);
        }
        let pos = self.position;
        let (viewport, scissor) = cached_data.viewport_and_scissor();
        let mut pipeline_cmd = cmd.bind_pipeline(rec.base_pipeline(), &[viewport], &[scissor])?;
        let pc_vertex = push_constants_vertex(
            pos,
            vec2(1.0, 1.0),
            cached_data.inv_aspect_ratio,
            cached_data.unit_scale,
        );
        let focused_stroke_thickness = self.focused_stroke_thickness;
        let pc_fragment = base_push_constants_fragment(
            pos - vec2(focused_stroke_thickness, focused_stroke_thickness),
            pos + self.main_rect.max + vec2(focused_stroke_thickness, focused_stroke_thickness),
        );
        pipeline_cmd.push_constants(pc_vertex.0, &[pc_vertex.1])?;
        pipeline_cmd.push_constants(pc_fragment.0, &[pc_fragment.1])?;
        pipeline_cmd.begin_drawing_indexed(
            self.window_draw_info,
            gpu::IndexBufferInfo::new(idx_id, idx_mem.offset),
            &[gpu::DrawBufferRange::new(vert_id, vert_mem.offset, vert_mem.size)],
            None, |cmd| { cmd.draw_indexed()?; Ok(()) }
        )?;
        let size = self.size();
        let item_pad_inner = style.item_pad_inner;
        let content_bounds = BoundingRect::from_min_max(
            pos + vec2(item_pad_inner.x, self.title_bar_rect.max.y + item_pad_inner.y),
            pos + size - item_pad_inner,
        );
        let pc_fragment = base_push_constants_fragment(
            content_bounds.min,
            content_bounds.max,
        );
        pipeline_cmd.push_constants(pc_fragment.0, &[pc_fragment.1])?;
        pipeline_cmd.begin_drawing_indexed(
            self.content_draw_info,
            gpu::IndexBufferInfo::new(idx_id, idx_mem.offset),
            &[gpu::DrawBufferRange::new(vert_id, vert_mem.offset, vert_mem.size)],
            None, |cmd| { cmd.draw_indexed()?; Ok(()) }
        )?;
        self.painter_storage.draw(
            cmd, rec, &cached_data, sampler.clone(),
            pos + self.widget_scroll_off, content_bounds,
        ).context("failed to draw painter")?;
        let mut pipeline_cmd =  cmd.bind_pipeline(
            rec.text_pipeline(),
            &[viewport], &[scissor],
        )?;
        let font_scale = style.font_scale;
        let pc_vertex = push_constants_vertex(
            pos,
            vec2(font_scale, font_scale),
            cached_data.inv_aspect_ratio,
            cached_data.unit_scale
        );
        render_text(
            &mut pipeline_cmd,
            rec,
            self.combined_text.iter(),
            pc_vertex.1,
        ).context("failed to render text")?;
        if (self.ver_scroll_bar_visible() && self.ver_scroll_bar_renderable()) ||
            (self.hor_scroll_bar_visible() && self.hor_scroll_bar_renderable())
        {
            let mut pipeline_cmd = cmd.bind_pipeline(
                rec.base_pipeline(),
                &[(viewport)], &[scissor],
            )?;
            let pc_vertex = push_constants_vertex(
                pos,
                vec2(1.0, 1.0),
                cached_data.inv_aspect_ratio,
                cached_data.unit_scale,
            );
            pipeline_cmd.push_constants(pc_vertex.0, &[pc_vertex.1])?;
            pipeline_cmd.push_constants(pc_fragment.0, &[pc_fragment.1])?;
            let vert_count = self.scroll_bar_vertices.len();
            let idx_count = self.scroll_bar_indices.len();
            let vert_mem = rec.allocate_vertices(vert_count)?;
            let idx_mem = rec.allocate_indices(idx_count)?;
            unsafe {
                self.scroll_bar_vertices
                    .as_ptr()
                    .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_count as usize);
                self.scroll_bar_indices
                    .as_ptr()
                    .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_count as usize);
            }
            pipeline_cmd.begin_drawing_indexed(
                gpu::IndexedDrawInfo
                    ::default()
                    .index_count(idx_count),
                gpu::IndexBufferInfo::new(idx_id, idx_mem.offset),
                &[gpu::DrawBufferRange::new(vert_id, vert_mem.offset, vert_mem.size)],
                None,
                |cmd| { cmd.draw_indexed()?; Ok(()) }
            )?;
        }
        if self.hover_window_active() {
            self.hover_window.set_vertex_params(style);
            self.hover_window
                .draw(cmd, rec, style, &cached_data)
                .context("hover window render work failed")?;
        }
        Ok(())
    }
}

impl UiSurface for Window
{

    #[inline(always)]
    fn set_resizeable(&mut self, value: bool) {
        self.flags &= !Self::RESIZEABLE;
        or_flag!(self.flags, Self::RESIZEABLE, value);
    }

    #[inline(always)]
    fn set_clamp_height(&mut self, value: bool) {
        self.flags &= !Self::CLAMP_HEIGHT;
        or_flag!(self.flags, Self::CLAMP_HEIGHT, value);
    }

    #[inline(always)]
    fn set_clamp_width(&mut self, value: bool) {
        self.flags &= !Self::CLAMP_WIDTH;
        or_flag!(self.flags, Self::CLAMP_WIDTH, value);
    }

    #[inline(always)]
    fn moving(&self) -> bool {
        self.held() || self.any_resize()
    }

    #[inline(always)]
    fn set_widget_rect_max(&mut self, max: Vec2) {
        self.widget_rect_max = max;
    }

    #[inline(always)]
    fn painter_storage(&mut self) -> &mut PainterStorage {
        &mut self.painter_storage
    }

    #[inline(always)]
    fn activate_collapsing_header(
        &mut self,
        label: &str,
    ) -> (&mut CollapsingHeader, CollapsingHeaderId)
    {
        let mut id = CollapsingHeaderId((label as *const str).addr());
        while !self.active_collapsing_headers.insert(id) {
            id.0 = id.0.wrapping_add(1);
        }
        let (last_triangulation, collapsing_headers) =
            self.collapsing_headers.entry(id).or_insert_with(|| (0, CollapsingHeader::new()));
        if *last_triangulation != self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        (collapsing_headers, id)
    }

    #[inline(always)]
    fn get_collapsing_header(&self, id: CollapsingHeaderId) -> Option<&CollapsingHeader> {
        self.collapsing_headers.get(&id).map(|v| &v.1)
    }

    #[inline(always)]
    fn get_collapsing_header_mut(&mut self, id: CollapsingHeaderId) -> Option<&mut CollapsingHeader> {
        self.collapsing_headers.get_mut(&id).map(|v| &mut v.1)
    }

    fn reaction_text(
        &mut self,
        style: &UiStyle,
        text_renderer: &mut TextRenderer,
        id: ReactionId,
        text: &str,
    ) -> SharedText
    {
        let entry = self.reaction_text
            .entry(id)
            .or_default();
        if entry.0 != text {
            entry.0 = text.into();
            let mut row = RowOffsets::new();
            let text = text_renderer
                .render_and_collect_offsets(
                    &[text_segment(text, &style.regular_font)],
                    false,
                    0.0,
                    0.0,
                    |offset| {
                        row.offsets.push(offset);
                    }
                ).unwrap_or_default();
            row.row_height = text.row_height;
            entry.1 = SharedText::new(Text::new(
                text,
                Vec32::with_len(1, row),
                Default::default(),
                Default::default(),
                vec2(1.0, 1.0),
                None,
                0,
                1, 
                None,
                None
            ));
        }
        entry.1.clone()
    }

    fn reaction_data_or_insert_with<T: 'static>(
        &mut self,
        id: ReactionId,
        mut f: impl FnMut() -> T,
    ) -> Option<NonNull<T>> {
        let alloc =
            if self.using_reaction_data_alloc_1() {
                &self.reaction_data_alloc_1
            } else {
                &self.reaction_data_alloc_0
            };
        let ty = TypeId::of::<T>();
        let entry = self.reaction_data
            .entry(id)
            .or_insert_with(|| {
                let ptr = unsafe {
                    alloc
                        .alloc_uninit::<T>(1)
                };
                if let Ok(ptr) = ptr {
                    unsafe {
                        ptr.write(f());
                    }
                }
                ReactionData {
                    ty,
                    ptr: ptr.ok().map(|v| v.cast()),
                    move_fn: Box::new(|ptr, alloc| unsafe {
                        let ptr = ptr?.cast::<T>();
                        let new_ptr = alloc.alloc_uninit::<T>(1).ok()?;
                        ptr.copy_to_nonoverlapping(new_ptr, 1);
                        Some(new_ptr.cast())
                    }),
                    drop_fn: Box::new(|ptr| unsafe {
                        if let Some(ptr) = ptr {
                            ptr.cast::<T>().drop_in_place();
                        }
                    }),
                }
            });
        if entry.ty != ty {
            (entry.drop_fn)(entry.ptr);
            let ptr = unsafe {
                alloc.alloc_uninit::<T>(1)
            };
            if let Ok(ptr) = ptr {
                unsafe {
                    ptr.write(f());
                }
            }
            *entry = ReactionData {
                ty,
                ptr: ptr.ok().map(|v| v.cast()),
                move_fn: Box::new(|ptr, alloc| unsafe {
                    let ptr = ptr?.cast::<T>();
                    let new_ptr = alloc.alloc_uninit::<T>(1).ok()?;
                    ptr.copy_to_nonoverlapping(new_ptr, 1);
                    Some(new_ptr.cast())
                }),
                drop_fn: Box::new(|ptr| unsafe {
                    if let Some(ptr) = ptr {
                        ptr.cast::<T>().drop_in_place();
                    }
                }),
            }
        }
        Some(entry.ptr?.cast())
    }

    #[inline(always)]
    fn add_text(
        &mut self,
        text: SharedText,
    ) -> u32
    {
        self.text.push(text);
        self.text.len() - 1
    }

    #[inline(always)]
    fn get_text(&mut self, index: u32) -> Option<SharedText> {
        self.text.get(index as usize).cloned()
    }

    #[inline(always)]
    fn animated_bool(&mut self, id: ReactionId, value: bool) -> f32 {
        let entry = self.animated_bools
            .entry(id)
            .or_insert_with(|| (
                if value {
                    1.0
                } else {
                    0.0
                },
                value
            ));
        entry.1 = value;
        if let Some(reaction) = self.reactions.get_mut(&id) {
            reaction.enable_animated_bool();
        }
        entry.0
    }

    #[inline(always)]
    fn tmp_data<T>(&self, count: usize) -> Option<NonNull<T>> {
        let alloc =
            if self.using_reaction_data_alloc_1() {
                &self.reaction_data_alloc_1
            } else {
                &self.reaction_data_alloc_0
            };
        unsafe {
            alloc.alloc_uninit(count).ok()
        }
    }
}

impl UiReactSurface for Window {

    type Surface = Self;

    fn ui_surface(&self) -> &Self::Surface {
        self
    }

    fn ui_surface_mut(&mut self) -> &mut Self::Surface {
        self
    }

    fn reaction_from_addr<T: RefAddr>(
        &mut self,
        value: T,
        mut f: impl for<'a> FnMut(&mut Self::Surface, &'a mut Reaction, T),
    ) -> &mut Reaction {
        let reaction: *mut Reaction = self.activate_reaction(&value);
        f(self, unsafe { &mut *reaction }, value);
        unsafe { &mut *reaction }
    }

    fn get_reaction(&self, id: ReactionId) -> Option<&Reaction> {
        self.reactions.get(&id)
    }

    fn get_reaction_mut(&mut self, id: ReactionId) -> Option<&mut Reaction> {
        self.reactions.get_mut(&id)
    }
}

impl Drop for Window {

    fn drop(&mut self) {
        for (_, data) in &mut self.reaction_data {
            (data.drop_fn)(data.ptr);
        }
    }
}
