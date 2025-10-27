use core::{
    hash::Hash,
    str::FromStr,
    marker::PhantomData,
    f32::consts::FRAC_PI_2,
};

use nox::{
    mem::{
        vec_types::{GlobalVec, Vector},
        Hashable,
    },
    *,
};

use rustc_hash::{FxHashMap, FxHashSet};

use compact_str::CompactString;

use nox_font::{VertexTextRenderer, text_segment, RenderedText, CombinedRenderedText};

use nox_geom::{
    shapes::*, *
};

use crate::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WidgetId {
    SelectableText(Hashable<f64>),
    Slider(Hashable<f64>),
    Button(Hashable<f64>),
    Checkbox(Hashable<f64>),
    ColorPicker(Hashable<f64>),
    InputText(Hashable<f64>),
    DragValue(Hashable<f64>),
    RadioButton(Hashable<f64>),
    SelectabelTag(Hashable<f64>),
    ComboBox(Hashable<f64>),
}

pub struct CollapsedWidgets {
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
    fn set_label<FontHash>(&mut self, style: &impl WindowStyle<FontHash>, text_renderer: &mut VertexTextRenderer<FontHash>, title: &str)
        where 
            FontHash: UiFontHash,
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
        widget_active: bool,
        mut collect_text: impl FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> f32
        where
            I: Interface,
            FontHash: UiFontHash,
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
        self.flags |= Self::HOVERED * bounding_rect.is_point_inside(cursor_pos) as u32 * !widget_active as u32;
        if !widget_active && nox.was_mouse_button_pressed(MouseButton::Left) && self.hovered() {
            self.flags ^= Self::COLLAPSED;
        }
        if self.collapsed() {
            self.rotation = (self.rotation - FRAC_PI_2 * style.animation_speed() * nox.delta_time_secs_f32()).clamp(0.0, FRAC_PI_2);
        } else {
            self.rotation = (self.rotation + FRAC_PI_2 * style.animation_speed() * nox.delta_time_secs_f32()).clamp(0.0, FRAC_PI_2);
        }
        collect_text(&self.title_text, offset + vec2(collapse_scale + style.item_pad_inner().x, 0.0), BoundedTextInstance {
            add_scale: vec2(1.0, 1.0),
            min_bounds: vec2(f32::MIN, f32::MIN),
            max_bounds: vec2(f32::MAX, f32::MAX),
            color: if self.hovered() {
                style.focused_text_col()
            } else {
                style.inactive_text_col()
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
                    style.inactive_text_col(),
                )
            };
        let offset = self.offset + vec2(scale * 0.5, style.calc_text_height(&self.title_text) * 0.5);
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

struct WidgetTables<I, FontHash, Style> {
    selectable_texts: FxHashMap<Hashable<f64>, (u64, SelectableText<I, FontHash, Style>)>,
    buttons: FxHashMap<Hashable<f64>, (u64, Button<I, FontHash, Style>)>,
    sliders: FxHashMap<Hashable<f64>, (u64, Slider<I, FontHash, Style>)>,
    checkboxes: FxHashMap<Hashable<f64>, (u64, Checkbox<I, FontHash, Style>)>,
    input_texts: FxHashMap<Hashable<f64>, (u64, InputText<I, FontHash, Style>)>,
    drag_values: FxHashMap<Hashable<f64>, (u64, DragValue<I, FontHash, Style>)>,
    color_pickers: FxHashMap<Hashable<f64>, (u64, ColorPicker<I, FontHash, Style>)>,
    radio_buttons: FxHashMap<Hashable<f64>, (u64, RadioButton<I, FontHash, Style>)>,
    selectable_tags: FxHashMap<Hashable<f64>, (u64, SelectableTag<I, FontHash, Style>)>,
    combo_boxes: FxHashMap<Hashable<f64>, (u64, ComboBox<I, FontHash, Style>)>,
}

impl<I, FontHash, Style> WidgetTables<I, FontHash, Style>
    where 
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
{

    fn new() -> Self {
        Self {
            selectable_texts: FxHashMap::default(),
            buttons: FxHashMap::default(),
            sliders: FxHashMap::default(),
            checkboxes: FxHashMap::default(),
            input_texts: FxHashMap::default(),
            drag_values: FxHashMap::default(),
            color_pickers: FxHashMap::default(),
            radio_buttons: FxHashMap::default(),
            selectable_tags: FxHashMap::default(),
            combo_boxes: FxHashMap::default(),
        }
    }

    #[inline(always)]
    fn get_widget(&self, widget: WidgetId) -> (u64, &dyn Widget<I, FontHash, Style>) {
        match widget {
            WidgetId::Slider(id) =>
                self.sliders.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::Button(id) =>
                self.buttons.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::Checkbox(id) =>
                self.checkboxes.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::ColorPicker(id) =>
                self.color_pickers.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::InputText(id) =>
                self.input_texts.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::DragValue(id) =>
                self.drag_values.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::SelectableText(id) =>
                 self.selectable_texts.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::RadioButton(id) =>
                 self.radio_buttons.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::SelectabelTag(id) =>
                 self.selectable_tags.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::ComboBox(id) =>
                 self.combo_boxes.get(&id).map(
                    |(l, w)| (*l, w as &dyn Widget<I, FontHash, Style>)
                ).unwrap(),
        }
    }

    #[inline(always)]
    fn get_widget_mut(
        &mut self,
        widget: WidgetId
    ) -> (&mut u64, &mut dyn Widget<I, FontHash, Style>)
    {
        match widget {
            WidgetId::Slider(id) =>
                self.sliders.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::Button(id) =>
                self.buttons.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::Checkbox(id) =>
                self.checkboxes.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::ColorPicker(id) =>
                self.color_pickers.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style,>)
                ).unwrap(),
            WidgetId::InputText(id) =>
                self.input_texts.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::DragValue(id) =>
                self.drag_values.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::SelectableText(id) =>
                 self.selectable_texts.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::RadioButton(id) =>
                 self.radio_buttons.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::SelectabelTag(id) =>
                 self.selectable_tags.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
            WidgetId::ComboBox(id) =>
                 self.combo_boxes.get_mut(&id).map(
                    |(l, w)| (l, w as &mut dyn Widget<I, FontHash, Style>)
                ).unwrap(),
        }
    }
}

pub struct Window<I, FontHash, Style>
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
    combined_text: CombinedRenderedText<BoundedTextInstance, GlobalVec<BoundedTextInstance>>,
    vertices: GlobalVec<Vertex>,
    indices: GlobalVec<u32>,
    text: GlobalVec<Text>,
    widgets: Option<WidgetTables<I, FontHash, Style>>,
    active_widgets: FxHashSet<WidgetId>,
    prev_active_widgets: GlobalVec<WidgetId>,
    collapsed_widgets: FxHashMap<Hashable<f64>, (u64, CollapsedWidgets)>,
    active_collapsed_widgets: FxHashSet<Hashable<f64>>,
    prev_active_collapsed_widgets: GlobalVec<Hashable<f64>>,
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
        FontHash: UiFontHash,
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
            combined_text: CombinedRenderedText::new(),
            vertices: Default::default(),
            indices: Default::default(),
            text: Default::default(),
            widgets: Some(WidgetTables::new()),
            active_widgets: Default::default(),
            prev_active_widgets: Default::default(),
            collapsed_widgets: FxHashMap::default(),
            active_collapsed_widgets: Default::default(),
            prev_active_collapsed_widgets: Default::default(),
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
    pub fn add_text(
        &mut self,
        text: Text
    ) -> usize 
    {
        self.text.push(text);
        self.text.len() - 1
    }

    #[inline(always)]
    pub fn get_text_mut(
        &mut self,
        text_index: usize,
    ) -> &mut Text
    {
        &mut self.text[text_index]
    }

    #[inline(always)]
    pub fn edit_selectable_text(
        &mut self,
        id: WidgetId,
        mut f: impl FnMut(&mut SelectableText<I, FontHash, Style>)
    )
    {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        match id {
            WidgetId::SelectableText(id) => {
                if let Some((_, text)) = widgets.selectable_texts.get_mut(&id) {
                    f(text);
                }
            }
            _ => {}
        }
    }

    #[inline(always)]
    pub fn add_selectable_text(
        &mut self,
        id: WidgetId,
        mut f: impl FnMut(usize, &Text),
    ) {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        match id {
            WidgetId::SelectableText(id) => {
                let (_, text) = widgets.selectable_texts.get(&id).unwrap();
                for text in text.as_text() {
                    let index = self.text.len();
                    let text = self.text.push(text.clone());
                    f(index, text)
                }
            }
            _ => {}
        }
    }

    #[inline(always)]
    pub fn activate_widget<'a, W: Widget<I, FontHash, Style>, T: ?Sized>(
        &'a mut self,
        value: &T,
        mut make_id: impl FnMut(Hashable<f64>) -> WidgetId,
        get_widget: impl FnOnce(&'a mut Self, Hashable<f64>) -> &'a mut W,
    ) -> (&'a mut W, WidgetId)
    {
        let mut id = Hashable((value as *const T).addr() as f64);
        while !self.active_widgets.insert(make_id(id)) {
            id.0 += 0.01;
        }
        let widget = get_widget(self, id);
        (widget, make_id(id))
    }

    #[inline(always)]
    pub fn get_selectable_text(&mut self, id: Hashable<f64>) -> &mut SelectableText<I, FontHash, Style> {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        let entry = widgets.selectable_texts
           .entry(id)
           .or_insert((0, SelectableText::new()));
        if entry.0 < self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        &mut entry.1
    }

    #[inline(always)]
    pub fn get_button(&mut self, id: Hashable<f64>) -> &mut Button<I, FontHash, Style> {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        let entry = widgets.buttons
           .entry(id)
           .or_insert((0, Button::new()));
        if entry.0 < self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        &mut entry.1
    }

    #[inline(always)]
    pub fn get_slider(&mut self, id: Hashable<f64>) -> &mut Slider<I, FontHash, Style> {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        let entry = widgets.sliders
           .entry(id)
           .or_insert((0, Slider::new()));
        if entry.0 < self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        &mut entry.1
    }

    #[inline(always)]
    pub fn get_checkbox(&mut self, id: Hashable<f64>) -> &mut Checkbox<I, FontHash, Style> {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        let entry = widgets.checkboxes
           .entry(id)
           .or_insert((0, Checkbox::new()));
        if entry.0 < self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        &mut entry.1
    }

    #[inline(always)]
    pub fn get_input_text(&mut self, id: Hashable<f64>) -> &mut InputText<I, FontHash, Style> {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        let entry = widgets.input_texts
           .entry(id)
           .or_insert((0, InputText::new()));
        if entry.0 < self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        &mut entry.1
    }

    #[inline(always)]
    pub fn get_drag_value(&mut self, id: Hashable<f64>) -> &mut DragValue<I, FontHash, Style> {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        let entry = widgets.drag_values
           .entry(id)
           .or_insert((0, DragValue::new()));
        if entry.0 < self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        &mut entry.1
    }

    #[inline(always)]
    pub fn get_color_picker(&mut self, id: Hashable<f64>) -> &mut ColorPicker<I, FontHash, Style> {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        let entry = widgets.color_pickers
           .entry(id)
           .or_insert((0, ColorPicker::new()));
        if entry.0 < self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        &mut entry.1
    }

    #[inline(always)]
    pub fn get_radio_button(&mut self, id: Hashable<f64>) -> &mut RadioButton<I, FontHash, Style> {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        let entry = widgets.radio_buttons
           .entry(id)
           .or_insert((0, RadioButton::new()));
        if entry.0 < self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        &mut entry.1
    }

    #[inline(always)]
    pub fn get_selectable_tag(&mut self, id: Hashable<f64>) -> &mut SelectableTag<I, FontHash, Style> {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        let entry = widgets.selectable_tags
            .entry(id)
            .or_insert((0, SelectableTag::new()));
        if entry.0 < self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        &mut entry.1
    }

    #[inline(always)]
    pub fn get_combo_box(&mut self, id: Hashable<f64>) -> &mut ComboBox<I, FontHash, Style> {
        let widgets = unsafe {
            self.widgets
                .as_mut()
                .unwrap_unchecked()
        };
        let entry = widgets.combo_boxes
            .entry(id)
            .or_insert((0, ComboBox::new()));
        if entry.0 < self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        &mut entry.1
    }

    #[inline(always)]
    pub fn activate_collapsed_widgets(
        &mut self,
        label: &str,
    ) -> &mut CollapsedWidgets
    {
        let mut id = Hashable((label as *const str).addr() as f64);
        while !self.active_collapsed_widgets.insert(id) {
            id.0 += 0.01;
        }
        let (last_triangulation, collapsed_widgets) = self.collapsed_widgets.entry(id).or_insert((0, CollapsedWidgets::new()));
        if *last_triangulation != self.last_triangulation {
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        collapsed_widgets
    }

    #[inline(always)]
    pub fn size(&self) -> Vec2 {
        self.main_rect.max
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
        self.prev_active_widgets.clear();
        for &widget in &self.active_widgets {
            self.prev_active_widgets.push(widget);
        }
        self.active_widgets.clear();
        self.prev_active_collapsed_widgets.clear();
        for &c in &self.active_collapsed_widgets {
            self.prev_active_collapsed_widgets.push(c);
        }
        self.active_collapsed_widgets.clear();
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
            FontHash: UiFontHash,
    {
        let mut cursor_in_this_window =
            !cursor_in_other_window &&
            self.bounding_rect(style.cursor_error_margin()).is_point_inside(cursor_pos);
        if cursor_in_this_window && style.override_cursor() && !self.any_resize() {
            nox.set_cursor(CursorIcon::Default);
        }
        let mut min_width: f32 = self.min_width;
        let min_height = self.min_height;
        let window_pos = self.position;
        self.prev_active_widgets.retain(|v| !self.active_widgets.contains(v));
        let mut widgets = unsafe {
            self.widgets.take().unwrap_unchecked()
        };
        for &widget in &self.prev_active_widgets {
            let (_, widget) = widgets.get_widget_mut(widget);
            widget.hide(&mut self.vertices);
        }
        self.prev_active_collapsed_widgets.retain(|v| !self.active_collapsed_widgets.contains(v));
        for collapsed_widgets in &self.prev_active_collapsed_widgets {
            let (_, collapsed_widgets) = &self.collapsed_widgets[collapsed_widgets];
            collapsed_widgets.hide(&mut self.vertices);
        }
        self.flags &= !(Self::CURSOR_IN_WINDOW | Self::HOVER_WINDOW_ACTIVE);
        let item_pad_outer = style.item_pad_outer();
        let item_pad_inner = style.item_pad_inner();
        let font_scale = style.font_scale();
        self.combined_text.clear();
        let mut hover_window = self.hover_window.take().unwrap();
        let mut active_widget = None;
        let mut hovered_widget = None;
        let mut cursor_in_some_widget = false;
        for (i, &widget) in self.active_widgets.iter().enumerate() {
            let (_, widget) = widgets.get_widget(widget);
            match widget.status(nox, style, window_pos, cursor_pos) {
                WidgetStatus::Inactive => {},
                WidgetStatus::Hovered(text) => {
                    if let Some(text) = text {
                        hover_window.update(style, text_renderer, cursor_pos, text);
                        self.flags |= Self::HOVER_WINDOW_ACTIVE;
                    }
                    cursor_in_some_widget = true;
                    hovered_widget = Some(i)
                },
                WidgetStatus::Active => {
                    cursor_in_some_widget = true;
                    active_widget = Some(i)
                },
            }
        }
        for collapsed_widgets in &self.active_collapsed_widgets {
            let (_, collapsed_widgets) = self.collapsed_widgets.get_mut(collapsed_widgets).unwrap();
            let width = collapsed_widgets.update(nox, window_pos, cursor_pos, style, active_widget.is_some(), |text, offset, bounded_text_instance| {
                self.combined_text.add_text(text, offset / font_scale, bounded_text_instance).unwrap();
            });
            min_width = min_width.max(width);
            if collapsed_widgets.hovered() && active_widget.is_none() {
                hovered_widget = Some(self.active_widgets.len());
            }
        }
        let window_moving = self.held() || self.any_resize();
        let size = self.size();
        for (i, &widget) in self.active_widgets.iter().enumerate() {
            let (_, widget) = widgets.get_widget_mut(widget);
            let UpdateResult {
                requires_triangulation,
                cursor_in_widget,
            } = widget.update(
                nox,
                style,
                text_renderer,
                size,
                window_pos,
                cursor_pos,
                delta_cursor_pos,
                cursor_in_this_window,
                if let Some(w) = active_widget {
                    w != i
                } else {
                    false
                },
                if let Some(w) = hovered_widget {
                    w != i
                } else {
                    false
                }, window_moving,
                &mut |text, offset, bounded_instance| {
                    self.combined_text.add_text(text, offset / font_scale, bounded_instance).unwrap();
                },
            );
            if requires_triangulation {
                self.flags |= Self::REQUIRES_TRIANGULATION;
            }
            cursor_in_some_widget |= cursor_in_widget;
        }
        cursor_in_this_window |= cursor_in_some_widget;
        self.widgets = Some(widgets);
        self.flags |= Self::CURSOR_IN_WINDOW * cursor_in_this_window as u32;
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
        self.combined_text
            .add_text(
                self.title_text.as_ref().unwrap(),
                vec2(item_pad_outer.x, item_pad_inner.y) / (font_scale * title_add_scale),
                BoundedTextInstance {
                    add_scale: vec2(title_add_scale, title_add_scale),
                    min_bounds: self.position,
                    max_bounds: self.position + title_bar_rect.max,
                    color:
                        if self.held() || self.any_resize() {
                            style.active_text_col()
                        }
                        else if self.cursor_in_window() {
                            style.focused_text_col()
                        } else {
                            style.inactive_text_col()
                        },
                }
            )
            .unwrap();
        if main_rect_max.y < min_height {
            main_rect_max.y = min_height;
        }
        for text in &self.text {
            self.combined_text.add_text(
                &text.text,
                text.offset / font_scale,
                BoundedTextInstance {
                    add_scale: text.scale,
                    min_bounds: self.position,
                    max_bounds: self.position + main_rect_max,
                    color: text.color, 
                }
            ).unwrap();
        }
        self.text.clear();
        let requires_triangulation =
            (style.rounding() != self.main_rect.rounding ||
            self.focused_outline_width != style.focused_window_outline_width() ||
            self.outline_width != style.window_outline_width() ||
            main_rect_max != self.main_rect.max ||
            self.title_bar_rect != title_bar_rect
        ) as u32;
        self.flags |= Self::REQUIRES_TRIANGULATION * requires_triangulation;
        self.main_rect.rounding = style.rounding();
        self.main_rect.max = main_rect_max;
        self.title_bar_rect = title_bar_rect;
        self.outline_width = style.window_outline_width();
        self.focused_outline_width = style.focused_window_outline_width();
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
            self.vertices.clear();
            self.indices.clear();
            let mut points = GlobalVec::new();
            let mut indices_usize = GlobalVec::new();
            self.main_rect.to_points(&mut |p| { points.push(p.into()); });
            let mut helper_points = GlobalVec::new();
            outline_points(&points,
                self.focused_outline_width, false, &mut |p| { helper_points.push(p.into()); }
            );
            if !earcut::earcut(&helper_points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.focused_outline_vertex_range = VertexRange::new(0..self.vertices.len());
            helper_points.clear();
            outline_points(&points,
                self.outline_width, false, &mut |p| { helper_points.push(p.into()); }
            );
            let mut vertex_begin = self.vertices.len();
            if !earcut::earcut(&helper_points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.outline_vertex_range = VertexRange::new(vertex_begin..self.vertices.len());
            vertex_begin = self.vertices.len();
            if !earcut::earcut(&points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.main_rect_vertex_range = VertexRange::new(vertex_begin..self.vertices.len());
            points.clear();
            self.title_bar_rect.to_points_partial_round(true, true, false, false,
                &mut |p| { points.push(p.into()); }
            );
            helper_points.clear();
            outline_points(&points,
                self.outline_width, false, &mut |p| { helper_points.push(p.into()); });
            vertex_begin = self.vertices.len();
            if !earcut::earcut(&helper_points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.title_outline_vertex_range = VertexRange::new(vertex_begin..self.vertices.len());
            vertex_begin = self.vertices.len();
            if !earcut::earcut(&points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.title_bar_vertex_range = VertexRange::new(vertex_begin..self.vertices.len());
            let mut flags = self.flags;
            let mut widgets = unsafe {
                self.widgets.take().unwrap_unchecked()
            };
            for &widget in &self.active_widgets  {
                let (last_triangulation, widget) = widgets.get_widget_mut(widget);
                *last_triangulation = new_triangulation;
                points.clear();
                helper_points.clear();
                widget.triangulate(
                    &mut points,
                    &mut helper_points,
                    &mut |points: &[[f32; 2]]| {
                        let vertex_begin = self.vertices.len();
                        if !earcut::earcut(points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                            flags &= !Self::RENDERABLE;
                        }
                        VertexRange::new(vertex_begin..self.vertices.len())
                    }
                );
            }
            for collapsed_widgets in &self.active_collapsed_widgets {
                let (last_triangulation, collapsed_widgets) = self.collapsed_widgets.get_mut(collapsed_widgets).unwrap();
                *last_triangulation = new_triangulation;
                self.vertices.append(&[Default::default(); 3]);
                let n = self.vertices.len();
                indices_usize.append(&[n - 3, n - 2, n - 1]);
                collapsed_widgets.symbol_vertex_range = VertexRange::new(n - 3..n);
            }
            self.main_draw_info = DrawInfo {
                first_index: 0,
                index_count: indices_usize.len() as u32,
                ..Default::default()
            };
            self.widgets = Some(widgets);
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
        let vert_total = self.vertices.len();
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_total)?
        };
        let idx_total = self.indices.len();
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, idx_total)?
        };
        let mut widgets = unsafe {
            self.widgets.take().unwrap_unchecked()
        };
        for &widget in &self.active_widgets {
            let (_, widget) = widgets.get_widget_mut(widget);
            widget.set_vertex_params(style, &mut self.vertices);
        }
        for collapsed_widgets in &self.active_collapsed_widgets {
            let (_, collapsed_widgets) = self.collapsed_widgets.get_mut(collapsed_widgets).unwrap();
            collapsed_widgets.set_vertex_params(style, &mut self.vertices);
        }
        let vertex_sample = self.vertices[self.main_rect_vertex_range.start()];
        if vertex_sample.color != style.window_bg_col() {
            let target_color = style.window_bg_col();
            for vertex in &mut self.vertices[self.main_rect_vertex_range.range()] {
                vertex.color = target_color;
            }
        }
        let vertex_sample = self.vertices[self.title_bar_vertex_range.start()];
        if vertex_sample.color != style.window_title_bar_col() {
            let target_color = style.window_title_bar_col();
            for vertex in &mut self.vertices[self.title_bar_vertex_range.range()] {
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
            set_vertex_params(&mut self.vertices, self.focused_outline_vertex_range, vec2(0.0, 0.0), target_color);
            set_vertex_params(&mut self.vertices, self.title_outline_vertex_range, vec2(0.0, 0.0), target_color);
            hide_vertices(&mut self.vertices, self.outline_vertex_range);
        } else {
            let vertex_sample = self.vertices[self.focused_outline_vertex_range.start()];
            let target_color = ColorSRGBA::black(0.0);
            if vertex_sample.color != target_color {
                for vertex in &mut self.vertices[self.focused_outline_vertex_range.range()] {
                    vertex.color = target_color;
                }
            }
            let vertex_sample = self.vertices[self.outline_vertex_range.start()];
            let target_color = style.window_outline_col();
            if vertex_sample.color != target_color {
                for vertex in &mut self.vertices[self.outline_vertex_range.range()] {
                    vertex.color = target_color;
                }
            }
            let vertex_sample = self.vertices[self.title_outline_vertex_range.start()];
            if vertex_sample.color != style.window_outline_col() {
                let target_color = style.window_outline_col();
                for vertex in &mut self.vertices[self.title_outline_vertex_range.range()] {
                    vertex.color = target_color;
                }
            }
        }
        unsafe {
            self.vertices
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
        for &widget in &self.active_widgets {
            let (_, widget) = widgets.get_widget(widget);
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
            self.combined_text
                .iter()
                .map(|(&c, (t, b))| (c, t, b.as_slice())),
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
                get_custom_pipeline
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
        self.widgets = Some(widgets);
        Ok(())
    }
}

pub struct WindowContext<'a, 'b, I, FontHash, Style>
    where
        I: Interface,
        FontHash: UiFontHash, 
        Style: WindowStyle<FontHash>,
{
    style: &'a Style,
    window: &'a mut Window<I, FontHash, Style>,
    text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
    current_row_widgets: GlobalVec<(WidgetId, Vec2)>,
    current_row_text: GlobalVec<(usize, usize, usize, WidgetId)>,
    min_triangulation: u64,
    widget_off: Vec2,
    min_width: f32,
    min_width_sub: f32,
    row_widget_off_x: f32,
    current_height: f32,
    slider_width: f32,
    input_text_width: f32,
    collapsed: bool,
}

impl<'a, 'b, I, FontHash, Style> WindowContext<'a, 'b, I, FontHash, Style>
    where
        I: Interface,
        FontHash: UiFontHash,
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
            &[text_segment(window.title.as_str(), style.font_regular())],
            false,
            0.0,
        ).unwrap_or_default());
        Self {
            min_triangulation: window.last_triangulation,
            widget_off: vec2(
                style.item_pad_outer().x,
                style.calc_text_box_height_from_text_height(title_text.row_height * style.font_scale() * style.title_add_scale()) +
                    style.item_pad_outer().y,
            ),
            window,
            style,
            text_renderer,
            current_row_widgets: Default::default(),
            current_row_text: Default::default(),
            min_width: 0.0,
            min_width_sub: 0.0,
            slider_width: style.default_slider_width(),
            input_text_width: style.default_input_text_width(),
            current_height: 0.0,
            row_widget_off_x: style.item_pad_outer().x,
            collapsed: false,
        }
    }

    pub(crate) fn new_collapsing(
        label: &str,
        window: &'a mut Window<I, FontHash, Style>,
        style: &'a Style,
        text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
        widget_off: Vec2,
        slider_width: f32,
        input_text_width: f32,
        min_triangulation: u64,
    ) -> Self {
        let collapsed_widgets = window.activate_collapsed_widgets(label);
        collapsed_widgets.set_label(style, text_renderer, label);
        collapsed_widgets.set_offset(widget_off);
        let collapsed = collapsed_widgets.collapsed();
        let item_pad_outer = style.item_pad_outer();
        Self {
            widget_off: widget_off + vec2(item_pad_outer.x, style.calc_text_height(&collapsed_widgets.title_text) + style.item_pad_outer().y),
            row_widget_off_x: widget_off.x + item_pad_outer.x,
            window,
            style,
            text_renderer,
            current_row_widgets: Default::default(),
            current_row_text: Default::default(),
            min_triangulation,
            min_width: 0.0,
            min_width_sub: 0.0,
            slider_width,
            input_text_width,
            current_height: 0.0,
            collapsed,
        }
    }

    pub fn collapsing<F>(&mut self, label: &str, mut f: F)
        where 
            F: FnMut(&mut WindowContext<I, FontHash, Style>)
    {
        if self.collapsed {
            return
        }
        if self.current_height != 0.0 {
            self.end_row();
        }
        self.widget_off.x = self.row_widget_off_x;
        let mut collapsing = WindowContext::new_collapsing(
            label, self.window, self.style, self.text_renderer,
            self.widget_off, self.slider_width, self.input_text_width, self.min_triangulation,
        );
        if !collapsing.collapsed {
            f(&mut collapsing);
        }
        self.min_width = self.min_width.max(collapsing.widget_off.x.max(collapsing.min_width));
        self.widget_off.y = collapsing.widget_off.y;
        if !collapsing.collapsed && collapsing.current_height != 0.0 {
            self.widget_off.y += collapsing.current_height + self.style.item_pad_outer().y;
        }
    }

    pub fn end_row(&mut self) {
        let item_pad_outer = self.style.item_pad_outer();
        self.min_width = (self.widget_off.x - self.min_width_sub).max(self.min_width);
        self.widget_off.x = self.row_widget_off_x;
        self.widget_off.y += self.current_height + item_pad_outer.y;
        let current_height_half = self.current_height * 0.5;
        let widgets = unsafe {
            self.window.widgets.as_mut().unwrap_unchecked()
        };
        for &(widget, size) in &self.current_row_widgets {
            let (_, widget) = widgets.get_widget_mut(widget);
            let offset = widget.get_offset();
            widget.set_offset(vec2(offset.x, offset.y + current_height_half - size.y * 0.5));
        }
        let current_height_half_scaled = current_height_half / self.style.font_scale();
        for &(index, row_index, selectable_index, id) in &self.current_row_text {
            let text = &mut self.window.get_text_mut(index);
            let row_height_halved = text.text.row_height * 0.5;
            let row = &text.rows[row_index - text.row_offset as usize];
            for &offset in &row.offsets {
                if let Some(offset) = text.text.get_offset_mut(offset) {
                    let mut vec: Vec2 = offset.offset.into();
                    vec.y += current_height_half_scaled - row_height_halved;
                    offset.offset = vec.into();
                }
            }
            self.window.edit_selectable_text(id, |text| {
                let text = &mut text.as_text_mut()[selectable_index];
                if text.row_offset > row_index as u32 {
                    panic!("should not happen")
                }
                else if let Some(RowOffsets { offsets, row_height, max_x: _, min_x: _ }) = &mut text.rows.get_mut(row_index - text.row_offset as usize) {
                    let row_height_halved = *row_height * 0.5;
                    for offset in offsets {
                        offset.offset[1] += current_height_half_scaled - row_height_halved;
                    }
                }
            });
        }
        self.current_row_widgets.clear();
        self.current_row_text.clear();
        self.current_height = 0.0;
    }

    fn tag_internal(&mut self, tag: &str, color: impl Color, tool_tip: Option<&str>)
    {
        let window_width = self.window.size().x;
        let (selectable_text, id) = self.window
            .activate_widget(
                tag,
                |id| WidgetId::SelectableText(id),
                |win, id| win.get_selectable_text(id)
            );
        selectable_text.set_base_offset(vec2(self.row_widget_off_x, self.widget_off.y));
        selectable_text.set_offset(self.widget_off);
        selectable_text.set_current_height(self.current_height);
        let mut builder = selectable_text.as_builder(window_width, self.style, self.text_renderer);
        builder
            .color(color)
            .with_text(tool_tip, |b| {
                b.with_segment(tag, Some(self.style.font_regular()));
            }
        );
        self.widget_off = selectable_text.current_offset() + vec2(self.style.item_pad_outer().x, 0.0);
        self.current_height = selectable_text.current_height();
        self.window.add_selectable_text(id, |index, text|
            for (i, _) in text.rows.iter().enumerate() {
                self.current_row_text.push((index, i + text.row_offset as usize, text.selectable_index.unwrap(), id));
            }
        );
    }
    
    #[inline(always)]
    pub fn tag(&mut self, tag: &str)
    {
        self.tag_internal(tag, self.style.inactive_text_col(), None);
    }

    #[inline(always)]
    pub fn tag_with_color(&mut self, tag: &str, color: impl Color)
    {
        self.tag_internal(tag, color, None);
    }

    #[inline(always)]
    pub fn tag_with_tooltip(&mut self, tag: &str, tool_tip: &str)
    {
        self.tag_internal(tag, self.style.inactive_text_col(), Some(tool_tip));
    }

    #[inline(always)]
    pub fn tag_with_color_and_tooltip(&mut self, tag: &str, color: impl Color, tool_tip: &str)
    {
        self.tag_internal(tag, color, Some(tool_tip));
    }

    pub fn text(&mut self, label: &str, truncate: bool, mut f: impl FnMut(&mut SelectableTextBuilder<I, FontHash, Style>))
    {
        let window_width = self.window.size().x;
        let (selectable_text, id) = self.window.activate_widget(
            label,
            |id| WidgetId::SelectableText(id),
            |win, id| win.get_selectable_text(id)
        );
        selectable_text.set_trunc_to_window_width(truncate);
        selectable_text.set_offset(self.widget_off);
        selectable_text.set_current_height(self.current_height);
        selectable_text.set_base_offset(vec2(self.row_widget_off_x, self.widget_off.y));
        let mut builder = selectable_text.as_builder(window_width, self.style, self.text_renderer);
        f(&mut builder);
        let offset = selectable_text.current_offset();
        if truncate {
            self.min_width_sub = offset.x - self.widget_off.x;
        }
        self.widget_off = offset;
        self.current_height = selectable_text.current_height();
        self.window.add_selectable_text(id, |index, text|
            for(i, _) in text.rows.iter().enumerate() {
                self.current_row_text.push((index, i + text.row_offset as usize, text.selectable_index.unwrap(), id));
            }
        );
    }

    pub fn button(
        &mut self,
        label: &str,
    ) -> bool
    {
        let (button, id) = self.window.activate_widget(
            label,
            |id| WidgetId::Button(id),
            |win, id| win.get_button(id)
        );
        button.set_label(label, self.text_renderer, self.style);
        let size = button.calc_size(self.style, self.text_renderer);
        button.set_offset(self.widget_off);
        self.current_height = self.current_height.max(size.y);
        self.widget_off.x += size.x + self.style.item_pad_outer().x;
        self.current_row_widgets.push((id, size));
        button.pressed()
    }

    pub fn slider_width(&mut self, width: f32) {
        self.slider_width = width.clamp(self.style.slider_min_width(), f32::MAX);
    }

    pub fn slider<T: Sliderable>(
        &mut self,
        value: &mut T,
        min: T,
        max: T,
        drag_speed: f32,
    )
    { 
        let (slider, id) = self.window.activate_widget(
            value,
            |id| WidgetId::Slider(id),
            |win, id| win.get_slider(id)
        );
        let size = slider.calc_size(self.style, self.text_renderer);
        slider.update_value(self.style, self.slider_width, value, min, max, drag_speed);
        slider.set_offset(self.widget_off);
        self.current_height = self.current_height.max(size.y);
        self.widget_off.x += size.x + self.style.item_pad_outer().x;
        self.current_row_widgets.push((id, size));
    }

    pub fn checkbox(
        &mut self,
        value: &mut bool,
        label: &str,
    ) -> bool
    {
        let (checkbox, id) = self.window.activate_widget(
            value,
            |id| WidgetId::Checkbox(id),
            |win, id| win.get_checkbox(id)
        );
        checkbox.update_value(value);
        let size = checkbox.calc_size(self.style, self.text_renderer);
        checkbox.set_label(label, self.text_renderer, self.style);
        checkbox.set_offset(self.widget_off);
        self.current_height = self.current_height.max(size.y);
        self.widget_off.x += size.x + self.style.item_pad_outer().x;
        self.current_row_widgets.push((id, size));
        *value
    }

    pub fn color_picker<C: Color>(
        &mut self,
        value: &mut C,
    )
    {
        let (color_picker, id) = self.window.activate_widget(
            value,
            |id| WidgetId::ColorPicker(id),
            |win, id| win.get_color_picker(id)
        );
        if color_picker.picking() {
            *value = C::from_hsva(color_picker.calc_color(self.style));
        }
        else {
            color_picker.set_color(*value);
        }
        let size = color_picker.calc_size(self.style, self.text_renderer);
        color_picker.set_offset(self.widget_off);
        self.current_height = self.current_height.max(size.y);
        self.widget_off.x += size.x + self.style.item_pad_outer().x;
        self.current_row_widgets.push((id, size));
    }

    #[inline(always)]
    fn input_text_internal<T: core::fmt::Display + FromStr>(
        &mut self,
        value: &mut T,
        empty_input_prompt: &str,
        width: f32,
        center_text: bool,
        format_input: Option<fn(&mut dyn core::fmt::Write, &str) -> core::fmt::Result>
    )
    {
        let (input_text, id) = self.window.activate_widget(
            value,
            |id| WidgetId::InputText(id),
            |win, id| win.get_input_text(id)
        );
        let size = input_text.calc_size(self.style, self.text_renderer);
        input_text.set_params(
            width, None, center_text,
            empty_input_prompt, format_input, false
        );
        if input_text.active() {
            if let Some(v) = input_text.get_input() {
                *value = v;
            }
        } else {
            input_text.set_input(value);
        }
        input_text.set_offset(self.widget_off);
        self.current_height = self.current_height.max(size.y);
        self.widget_off.x += size.x + self.style.item_pad_outer().x;
        self.current_row_widgets.push((id, size));
    }

    #[inline(always)]
    pub fn input_text<T: core::fmt::Display + FromStr>(
        &mut self,
        value: &mut T,
        empty_input_prompt: &str,
        format_input: Option<fn(&mut dyn core::fmt::Write, &str) -> core::fmt::Result>
    )
    {
        self.input_text_internal(
            value, empty_input_prompt,
            self.input_text_width, false, format_input,
        );
    }
    
    #[inline(always)]
    pub fn centered_input_text<T: core::fmt::Display + FromStr>(
        &mut self,
        value: &mut T,
        empty_input_prompt: &str,
        width: f32,
        format_input: Option<fn(&mut dyn core::fmt::Write, &str) -> core::fmt::Result>
    )
    {
        self.input_text_internal(
            value, empty_input_prompt,
            width, true, format_input
        );
    }

    #[inline(always)]
    pub fn drag_value<T: Sliderable>(
        &mut self,
        value: &mut T,
        min: T,
        max: T,
        drag_speed: f32,
        min_width: f32,
        format_input: Option<fn(&mut dyn core::fmt::Write, &str) -> core::fmt::Result>,
    )
    {
        let (drag_value, id) = self.window.activate_widget(
            value,
            |id| WidgetId::DragValue(id),
            |win, id| win.get_drag_value(id)
        );
        let size = drag_value.calc_size(self.style, self.text_renderer);
        drag_value.set_input_params(self.style, min_width, format_input, false);
        drag_value.calc_value(
            self.style, value, min, max,
            drag_speed,
        );
        drag_value.set_offset(self.widget_off);
        self.current_height = self.current_height.max(size.y);
        self.widget_off.x += size.x + self.style.item_pad_outer().x;
        self.current_row_widgets.push((id, size));
    }

    #[inline(always)]
    pub fn radio_button<T: Clone + Eq>(
        &mut self,
        value: &mut T,
        radio_value: T,
        label: &str,
    )
    {
        let (radio_button, id) = self.window.activate_widget(
            label,
            |id| WidgetId::RadioButton(id),
            |win, id| win.get_radio_button(id)
        );
        radio_button.set_label(label, self.text_renderer, self.style);
        let size = radio_button.calc_size(self.style, self.text_renderer);
        radio_button.update_value(value, radio_value);
        radio_button.set_offset(self.widget_off);
        self.current_height = self.current_height.max(size.y);
        self.widget_off.x += size.x + self.style.item_pad_outer().x;
        self.current_row_widgets.push((id, size));
    }

    #[inline(always)]
    pub fn selectable_tag<T: Clone + Eq>(
        &mut self,
        value: &mut T,
        target: T,
        label: &str,
    )
    {
        let (selectable_tag, id) = self.window.activate_widget(
            label,
            |id| WidgetId::SelectabelTag(id),
            |win, id| win.get_selectable_tag(id)
        );
        selectable_tag.set_label(label, self.text_renderer, self.style);
        let size = selectable_tag.calc_size(self.style, self.text_renderer);
        selectable_tag.update_value(value, target);
        selectable_tag.set_offset(self.widget_off);
        self.current_height = self.current_height.max(size.y);
        self.widget_off.x += size.x + self.style.item_pad_outer().x;
        self.current_row_widgets.push((id, size));
    }

    #[inline(always)]
    pub fn combo_box<T: Clone + Eq>(
        &mut self,
        label: &str,
        f: impl FnMut(&mut ComboBoxBuilder<T, I, FontHash, Style>)
    ) {
        let (combo_box, id) = self.window.activate_widget(
            label,
            |id| WidgetId::ComboBox(id),
            |win, id| win.get_combo_box(id)
        );
        combo_box.update_values::<T>(self.style, self.text_renderer, f);
        let size = combo_box.calc_size(self.style, self.text_renderer);
        combo_box.set_offset(self.widget_off);
        self.current_height = self.current_height.max(size.y);
        self.widget_off.x += size.x + self.style.item_pad_outer().x;
        self.current_row_widgets.push((id, size));
    }
}

impl<'a, 'b, I, FontHash, Style> Drop for
        WindowContext<'a, 'b, I, FontHash, Style>
    where 
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>,
{
    fn drop(&mut self) {
        self.end_row();
        if self.min_triangulation < self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash, Style>::REQUIRES_TRIANGULATION;
        }
        self.window.min_width = self.min_width;
        self.window.min_height = self.widget_off.y;
    }
}
