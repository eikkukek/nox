use core::{
    ptr::NonNull,
    ops::{Deref, DerefMut},
    marker::PhantomData,
    cell::UnsafeCell,
};

use nox::{
    mem::{
        vec_types::{GlobalVec, Vector},
    },
    *
};

use nox_geom::{
    shapes::*,
    *,
};

use crate::{
    surface::*,
    collapsing_header::*,
    image::*,
    *
};

pub struct RowText {
    pub index: usize,
    pub row_index: usize,
    pub selectable_index: usize,
    pub reaction_id: Option<ReactionId>,
}

impl RowText {

    pub fn new(
        index: usize,
        row_index: usize,
        selectable_index: usize,
        reaction_id: Option<ReactionId>
    ) -> Self {
        Self {
            index,
            row_index,
            selectable_index,
            reaction_id,
        }
    }
}

pub struct ReactionRef<'a, Surface: UiReactSurface> {
    ptr: *mut Reaction,
    _marker: PhantomData<&'a mut Surface>
}

impl<'a, Surface: UiReactSurface> Deref for ReactionRef<'a, Surface> {

    type Target = Reaction;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.ptr
        }
    }
}

impl<'a, Surface: UiReactSurface> DerefMut for ReactionRef<'a, Surface> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.ptr
        }
    }
}

enum CtxSurface<'a, Surface> {
    Regular(&'a mut Surface),
    Collapsing(NonNull<&'a mut Surface>),
}

impl<'a, Surface> CtxSurface<'a, Surface> {

    fn as_mut(&mut self) -> &mut Surface {
        match self {
            Self::Regular(s) => s,
            Self::Collapsing(s) => unsafe {
                s.read()
            }
        }
    }
}

pub struct UiCtx<'a, 'b, Surface, Style>
    where
        Surface: UiReactSurface,
        Style: UiStyle,
{
    win_ctx: &'a mut WindowCtx,
    style: &'a Style,
    surface: UnsafeCell<CtxSurface<'a, Surface>>,
    text_renderer: &'a mut TextRenderer<'b>,
    current_row_text: GlobalVec<RowText>,
    current_row_reactions: GlobalVec<(ReactionId, Vec2)>,
    current_row_paints: Option<GlobalVec<NonNull<dyn FnMut(&mut Painter, Row)>>>,
    collapsing_header_id: CollapsingHeaderId,
    image_loader: &'a mut ImageLoader,
    widget_off: Vec2,
    beam_height: f32,
    min_width: f32,
    min_width_sub: f32,
    row_widget_off_x: f32,
    current_height: f32,
    slider_width: f32,
    input_text_width: f32,
    flags: u32,
}

impl<'a, 'b, Surface, Style> UiCtx<'a, 'b, Surface, Style>
    where
        Surface: UiReactSurface,
        Style: UiStyle,
{

    const IS_COLLAPSING: u32 = 0x1;
    const IS_COLLAPSED: u32 = 0x2;

    pub fn new(
        win_ctx: &'a mut WindowCtx,
        surface: &'a mut Surface,
        style: &'a Style,
        start_off: Vec2,
        text_renderer: &'a mut TextRenderer<'b>,
        image_loader: &'a mut ImageLoader,
    ) -> Self {
        Self {
            win_ctx,
            surface: UnsafeCell::new(CtxSurface::Regular(surface)),
            style,
            text_renderer: text_renderer,
            current_row_text: Default::default(),
            current_row_reactions: Default::default(),
            current_row_paints: Some(GlobalVec::new()),
            collapsing_header_id: Default::default(),
            image_loader,
            widget_off: start_off,
            min_width: 0.0,
            min_width_sub: 0.0,
            beam_height: 0.0,
            slider_width: style.default_slider_width(),
            input_text_width: style.default_input_text_width(),
            current_height: 0.0,
            row_widget_off_x: style.item_pad_outer().x,
            flags: 0,
        }
    }

    fn new_collapsing(
        label: &str,
        mut surface: NonNull<&'a mut Surface>,
        win_ctx: &'a mut WindowCtx,
        style: &'a Style,
        text_renderer: &'a mut TextRenderer<'b>,
        widget_off: Vec2,
        slider_width: f32,
        input_text_width: f32,
        image_loader: &'a mut ImageLoader,
    ) -> Self {
        let surface_ref = unsafe {
            surface.as_mut()
        };
        let (collapsing_header, id) = surface_ref.ui_surface_mut().activate_collapsing_header(label);
        collapsing_header.set_label(style, text_renderer, label);
        collapsing_header.offset = widget_off;
        let collapsed = collapsing_header.collapsed();
        let item_pad_outer = style.item_pad_outer();
        let base_off = widget_off +
            vec2(item_pad_outer.x, style.calc_text_height(&collapsing_header.label_text()) + style.item_pad_outer().y);
        let mut flags = Self::IS_COLLAPSING;
        or_flag!(flags, Self::IS_COLLAPSED, collapsed);
        Self {
            widget_off: base_off,
            row_widget_off_x: widget_off.x + item_pad_outer.x,
            win_ctx,
            surface: UnsafeCell::new(CtxSurface::Collapsing(surface)),
            style,
            text_renderer: text_renderer,
            current_row_text: Default::default(),
            current_row_reactions: Default::default(),
            current_row_paints: Some(GlobalVec::new()),
            collapsing_header_id: id,
            image_loader,
            min_width: 0.0,
            min_width_sub: 0.0,
            beam_height: 0.0,
            slider_width,
            input_text_width,
            current_height: 0.0,
            flags,
        }
    }

    #[inline(always)]
    fn is_collapsed(&self) -> bool {
        self.flags & Self::IS_COLLAPSED == Self::IS_COLLAPSED
    }

    #[inline(always)]
    fn is_collapsing(&self) -> bool {
        self.flags & Self::IS_COLLAPSING == Self::IS_COLLAPSING
    }

    #[inline(always)]
    pub fn resizeable(&mut self, value: bool) {
        self.surface
            .get_mut()
            .as_mut()
            .ui_surface_mut()
            .set_resizeable(value);
    }

    #[inline(always)]
    pub fn clamp_height(&mut self, value: bool) {
        self.surface
            .get_mut()
            .as_mut()
            .ui_surface_mut()
            .set_clamp_height(value);
    }

    #[inline(always)]
    pub fn clamp_width(&mut self, value: bool) {
        self.surface
            .get_mut()
            .as_mut()
            .ui_surface_mut()
            .set_clamp_width(value);
    }

    #[inline(always)]
    pub fn win_ctx(&mut self) -> &mut WindowCtx {
        self.win_ctx
    }

    #[inline(always)]
    pub fn style(&self) -> &Style {
        self.style
    }

    #[inline(always)]
    pub fn painter(&mut self) -> Painter<'_> {
        Painter::new(
            self.surface
                .get_mut()
                .as_mut()
                .ui_surface_mut()
                .painter_storage(),
            self.style,
            self.text_renderer,
            self.image_loader
        )
    }

    #[inline(always)]
    pub fn add<'c, T>(
        &mut self,
        (value, mut f): (T, impl FnMut(&mut UiReactCtx<Surface, Style>, &mut Reaction, T)),
    ) -> ReactionRef<'_, Surface>
        where 
            T: RefAddr,
    {
        let surface = unsafe {
            &mut *self.surface.get()
        };
        let reaction = surface.as_mut().reaction_from_addr(value, |surface, reaction, value| {
            reaction.offset = self.widget_off;
            f(&mut UiReactCtx { ui: self, surface }, reaction.into(), value)
        });
        let size = reaction.size;
        self.current_height = self.current_height.max(size.y);
        self.widget_off.x += size.x + self.style.item_pad_outer().x;
        self.current_row_reactions.push((reaction.id(), size));
        let ptr = reaction as _;
        ReactionRef { ptr, _marker: PhantomData }
    }

    #[inline(always)]
    pub fn render_text(&mut self, mut f: impl FnMut(&Style, &mut TextRenderer)) {
        f(self.style, self.text_renderer);
    }

    pub fn collapsing<'c, F>(&'c mut self, label: &str, mut f: F)
        where 
            F: FnMut(&mut UiCtx<'c, 'b, Surface, Style>),
    {
        if self.is_collapsed() {
            return
        }
        if self.current_height != 0.0 {
            self.end_row();
        }
        self.widget_off.x = self.row_widget_off_x;
        let item_pad_outer = self.style.item_pad_outer();
        let mut surface = self.surface.get_mut().as_mut();
        let mut collapsing = UiCtx::new_collapsing(
            label, NonNull::new(&mut surface).unwrap(), self.win_ctx, self.style, self.text_renderer,
            self.widget_off, self.slider_width, self.input_text_width,
            self.image_loader,
        );
        if !collapsing.is_collapsed() {
            f(&mut collapsing);
            if collapsing.current_height != 0.0 {
                collapsing.end_row();
            }
            let surface = collapsing.surface.get_mut().as_mut();
            if let Some(c) = surface.ui_surface_mut().get_collapsing_header_mut(collapsing.collapsing_header_id) {
                c.set_beam_height(collapsing.beam_height);
            }
        } else {
            let surface = collapsing.surface.get_mut().as_mut();
            if let Some(c) = surface.ui_surface_mut().get_collapsing_header_mut(collapsing.collapsing_header_id) {
                c.set_beam_height(0.0);
            }
        }
        self.min_width = self.min_width.max(collapsing.widget_off.x.max(collapsing.min_width));
        self.beam_height += collapsing.widget_off.y - self.widget_off.y;
        self.widget_off.y = collapsing.widget_off.y;
        if !collapsing.is_collapsed() && collapsing.current_height != 0.0 {
            let height_add = collapsing.current_height + item_pad_outer.y;
            self.widget_off.y += height_add;
            self.beam_height += height_add;
        }
    }

    pub fn end_row(&mut self) {
        if self.current_height == 0.0 {
            return
        }
        let item_pad_outer = self.style.item_pad_outer();
        self.min_width = (self.widget_off.x - self.min_width_sub).max(self.min_width);
        self.widget_off.x = self.row_widget_off_x;
        let height_add = self.current_height + item_pad_outer.y;
        self.beam_height += height_add;
        self.widget_off.y += height_add;
        let current_height = self.current_height;
        let current_height_halved = current_height * 0.5;
        let surface = self.surface.get_mut().as_mut();
        for &(reaction, size) in &self.current_row_reactions {
            if let Some(reaction) = surface.get_reaction_mut(reaction) {
                let offset = reaction.offset;
                reaction.offset = vec2(offset.x, offset.y + current_height_halved - size.y * 0.5);
            }
        }
        let mut paints = self.current_row_paints.take().unwrap();
        let mut painter = self.painter();
        let row = Row {
            height: current_height,
            height_halved: current_height_halved,
        };
        for paint in &mut paints {
            unsafe {
                paint.as_mut()(&mut painter, row);
            }
        }
        paints.clear();
        self.current_row_paints = Some(paints);
        let current_height_halved_scaled = current_height_halved / self.style.font_scale();
        let surface = self.surface.get_mut().as_mut();
        for &RowText { index, row_index, selectable_index: _, reaction_id: _ } in &self.current_row_text {
            if let Some(text) = surface.ui_surface_mut().get_text(index) {
                let text = &mut *text.as_mut();
                let row_height_halved = text.text.row_height * 0.5;
                let row = text.rows[row_index - text.row_offset as usize].clone();
                for &offset in &row.offsets {
                    if let Some(offset) = text.text.get_offset_mut(offset) {
                        let mut vec: Vec2 = offset.offset.into();
                        vec.y += current_height_halved_scaled - row_height_halved;
                        offset.offset = vec.into();
                    }
                }
                let delta_off = current_height_halved - row_height_halved * self.style.font_scale();
                text.bounds.min.y += delta_off;
                text.bounds.max.y += delta_off;
                /*
                if let Some(id) = reaction_id {
                    self.surface.edit_selectable_text(id, |text| {
                        let text = &mut text.as_text_mut()[selectable_index];
                        if text.row_offset > row_index as u32 {
                            panic!("should not happen")
                        }
                        else if let Some(RowOffsets { offsets, row_height, max_x: _, min_x: _ }) =
                            &mut text.rows.get_mut(row_index - text.row_offset as usize)
                        {
                            let row_height_halved = *row_height * 0.5;
                            for offset in offsets {
                                offset.offset[1] += current_height_halved_scaled - row_height_halved;
                            }
                        }
                    });
                }
                */
            }
        }
        self.current_row_text.clear();
        self.current_row_reactions.clear();
        self.current_height = 0.0;
    }

    /*

    fn tag_internal(&mut self, tag: &str, color: impl Color, tool_tip: Option<&str>)
    {
        let window_width = self.window.size().x;
        let (selectable_text, id) = self.window
            .activate_widget(
                tag,
                |id| WidgetId::SelectableText(id),
                |win, id| win.get_selectable_text(id)
            );
        selectable_text.set_base_and_start_offset(
            vec2(self.row_widget_off_x, self.widget_off.y),
            self.widget_off,
        );
        selectable_text.set_current_height(self.current_height);
        let mut builder = selectable_text.as_builder(
            window_width, self.style,
            self.text_renderer,
        );
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
                self.current_row_text.push(RowText::new(
                    index,
                    i + text.row_offset as usize,
                    text.selectable_index.unwrap(),
                    Some(id)
                ));
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

    pub fn text(&mut self, label: &str, truncate: bool, mut f: impl FnMut(&mut SelectableTextBuilder<Style>))
    {
        let window_width = self.window.size().x;
        let (selectable_text, id) = self.window.activate_widget(
            label,
            |id| WidgetId::SelectableText(id),
            |win, id| win.get_selectable_text(id)
        );
        selectable_text.set_trunc_to_window_width(truncate);
        selectable_text.set_current_height(self.current_height);
        selectable_text.set_base_and_start_offset(
            vec2(self.row_widget_off_x, self.widget_off.y),
            self.widget_off
        );
        let mut builder = selectable_text.as_builder(
            window_width, self.style,
            self.text_renderer,
        );
        f(&mut builder);
        let offset = selectable_text.current_offset();
        if truncate {
            self.min_width_sub = offset.x - self.widget_off.x;
        }
        self.widget_off = offset;
        self.current_height = selectable_text.current_height();
        self.window.add_selectable_text(id, |index, text|
            for(i, _) in text.rows.iter().enumerate() {
                self.current_row_text.push(RowText::new(
                    index,
                    i + text.row_offset as usize,
                    text.selectable_index.unwrap(),
                    Some(id)
                ));
            }
        );
    }
    */

    pub fn button(
        &mut self,
        label: &str,
    ) -> ReactionRef<'_, Surface>
    {
        self.add((label, |ui, reaction, label| {
            let id = reaction.id();
            let offset = reaction.offset();
            let visuals = ui.style().interact_visuals(&reaction);
            let mut size = Default::default();
            let text = ui
                .reaction_text(id, label)
                .edit(|text| {
                    text.offset = offset;
                    text.offset.x += ui.style().item_pad_inner().x;
                    text.color = visuals.fg_stroke_col();
                    size = ui.style().calc_text_box_size(&text.text);
                });
            reaction.size = size;
            ui.add_text(text);
            let rounding = self.style.rounding();
            ui.paint(move |painter, row| {
                painter
                    .rect(
                        id,
                        rect(Default::default(), size, rounding),
                        offset + vec2(0.0, row.height_halved - size.y * 0.5),
                        visuals.fill_col,
                        visuals.bg_strokes.clone(),
                        visuals.bg_stroke_idx,
                    );
            });
        })) 
    }

    pub fn checkbox(
        &mut self,
        value: &mut bool,
        label: &str,
    ) -> bool
    {
        self.add((&mut *value, |ui, reaction, value| {
            let item_pad_inner = ui.style().item_pad_inner();
            let id = reaction.id();
            let offset = reaction.offset();
            let size_max = ui.font_height();
            let rect_size = vec2(size_max, size_max);
            let mut text_width = 0.0;
            let visuals = ui.style().interact_visuals(&reaction);
            let fg_col = visuals.fg_stroke_col();
            let text = ui
                .reaction_text(id, label)
                .edit(|text| {
                    text_width = ui.style().calc_text_width(&text.text);
                    text.offset = offset;
                    text.offset.x += size_max + item_pad_inner.x;
                    text.color = fg_col;
                }); 
            ui.add_text(text);
            let size = vec2(size_max + text_width + item_pad_inner.x, size_max);
            reaction.size = size;
            let rounding = ui.style().rounding();
            if reaction.clicked() {
                *value = !*value;
            }
            let value = *value;
            ui.paint(move |painter, row| {
                let checkbox_col =
                    if value {
                        fg_col
                    } else {
                        ColorSRGBA::black(0.0)
                    };
                painter
                    .rect(
                        id,
                        rect(Default::default(), rect_size, rounding),
                        offset + vec2(0.0, row.height_halved - size.y * 0.5),
                        visuals.fill_col,
                        visuals.bg_strokes.clone(),
                        visuals.bg_stroke_idx
                    )
                    .checkmark(
                        id,
                        1.0,
                        offset + rect_size * 0.5 + vec2(0.0, row.height_halved - rect_size.y * 0.5),
                        checkbox_col,
                        Default::default(),
                        0
                    );
            });
        }));
        *value
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
        self.add((&mut *value, |ui, reaction, value| {
            let id = reaction.id();
            ui.reaction_data(id, || SliderData::new(), |ui, slider| {
                slider.update_value(ui.ui.slider_width, value, min, max);
                slider.update(ui, reaction);
            });
        }));
        self.add((&mut *value, |ui, reaction, value| {
            let id = reaction.id();
            ui.reaction_data(id, || DragValueData::new(), |ui, drag_value| {
                drag_value.set_input_params(
                    ui.style(),
                    ui.style().min_input_text_width(),
                    None,
                    false,
                );
                drag_value.calc_value(ui.style(), value, T::MIN, T::MAX, drag_speed);
                drag_value.update(ui, reaction);
            });
        }));
    }

/*
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
*/

    #[inline(always)]
    fn input_text_internal<T: core::fmt::Display + core::str::FromStr>(
        &mut self,
        value: &mut T,
        empty_input_prompt: &str,
        width: f32,
        center_text: bool,
        format_input: Option<fn(&mut dyn core::fmt::Write, &str) -> core::fmt::Result>,
    ) -> ReactionRef<'_, Surface> {
        self.add((value, |ui, reaction, value| {
            let id = reaction.id();
            ui.reaction_data(id, || InputTextData::new(), |ui, input_text| {
                input_text.set_params(
                    width, None, center_text,
                    empty_input_prompt, format_input,
                    false,
                );
                if input_text.active() {
                    if let Some(v) = input_text.get_input() {
                        *value = v;
                    }
                } else {
                    input_text.set_input(value);
                }
                input_text.update(
                    ui,
                    reaction,
                )
            });
        }))
    }

    #[inline(always)]
    pub fn input_text<T: core::fmt::Display + core::str::FromStr>(
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
    pub fn centered_input_text<T: core::fmt::Display + core::str::FromStr>(
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
    ) -> ReactionRef<'_, Surface>
    {
        self.add((value, |ui, reaction, value| {
            let id = reaction.id();
            ui.reaction_data(id, || DragValueData::new(), |ui, drag_value| {
                drag_value.set_input_params(
                    ui.style(),
                    min_width,
                    format_input,
                    false
                );
                drag_value.calc_value(ui.style(), value, min, max, drag_speed);
                drag_value.update(ui, reaction);
            });
        }))
    }

    #[inline(always)]
    pub fn radio_button<'c, T: Clone + Eq + 'static>(
        &'c mut self,
        value: &'c mut T,
        radio_value: T,
        label: &str,
    ) -> ReactionRef<'c, Surface>
    {
        self.add((value, |ui, reaction, value| {
            let item_pad_inner = ui.style().item_pad_inner();
            let id = reaction.id();
            let offset = reaction.offset;
            let radius = ui.style().default_handle_radius();
            let diameter = radius * 2.0;
            let visuals = ui.style().interact_visuals(reaction);
            let fg_col = visuals.fg_stroke_col();
            let mut text_size = Default::default();
            let text = ui
                .reaction_text(id, label)
                .edit(|text| {
                    text.offset = offset;
                    text.offset.x += diameter + item_pad_inner.x;
                    text.color = fg_col;
                    text_size = ui.style().calc_text_size(&text.text);
                });
            ui.add_text(text);
            let size = vec2(diameter + text_size.x + item_pad_inner.x, diameter.max(text_size.y));
            reaction.size = size;
            if reaction.clicked() {
                *value = radio_value.clone();
            }
            let value = value.clone();
            let radio_value = radio_value.clone();
            ui.paint(move |painter, row| {
                let size_y_half = size.y * 0.5;
                let radio_col =
                    if value == radio_value {
                        fg_col
                    } else {
                        ColorSRGBA::black(0.0)
                    };
                let inner_radius = radius * 0.4;
                painter
                    .circle(
                        id,
                        circle(vec2(radius, radius), radius),
                        16,
                        offset + vec2(0.0, row.height_halved - size_y_half),
                        visuals.fill_col,
                        visuals.bg_strokes.clone(),
                        visuals.bg_stroke_idx
                    )
                    .circle(
                        id,
                        circle(vec2(radius, radius), inner_radius),
                        16,
                        offset + vec2(0.0, row.height_halved - size_y_half),
                        radio_col,
                        Default::default(),
                        0
                    );
            });
        }))
    }

    #[inline(always)]
    pub fn selectable_tag<T: Clone + Eq + 'static>(
        &mut self,
        value: &mut T,
        target: T,
        label: &str,
    ) -> ReactionRef<'_, Surface>
    {
        self.add((value, |ui, reaction, value| {
            let item_pad_inner = ui.style().item_pad_inner();
            let id = reaction.id();
            let visuals = ui.style().interact_visuals(reaction);
            let mut size = Default::default();
            let text = ui
                .reaction_text(id, label).clone()
                .edit(|text| {
                    size = ui.style().calc_text_box_size(&text.text);
                    text.offset = reaction.offset();
                    text.offset.x += item_pad_inner.x;
                    text.color = visuals.fg_stroke_col();
                });
            reaction.size = size;
            ui.add_text(text);
            if reaction.clicked() {
                *value = target.clone();
            }
            let rounding = ui.style().rounding();
            let value = value.clone();
            let selected_col = ui.style().selection_col();
            let offset = reaction.offset();
            let target = target.clone();
            ui.paint(move |painter, row| {
                let fill_col =
                    if value == target {
                        selected_col
                    } else {
                        visuals.fill_col
                    };
                painter
                    .rect(
                        id,
                        rect(Default::default(), size, rounding),
                        offset + vec2(0.0, row.height_halved - size.y * 0.5),
                        fill_col,
                        visuals.bg_strokes.clone(),
                        visuals.bg_stroke_idx
                    );
            });
        }))
    }

    /*

    #[inline(always)]
    pub fn combo_box<T: Clone + Eq>(
        &mut self,
        label: &str,
        f: impl FnMut(&mut ComboBoxBuilder<T, Style>)
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
    */

    #[inline(always)]
    pub fn image(
        &mut self,
        label: &str,
        source: ImageSource,
        size: Vec2,
    ) -> ReactionRef<'_, Surface> {
        let reaction = self.add((label, |ui, reaction, _| {
            let offset = reaction.offset;
            reaction.size = size;
            let source = match source {
                ImageSource::Path(p) => unsafe {
                    let len = p.len();
                    if let Some(data) = ui.tmp_data(len) {
                        p.as_ptr()
                            .copy_to_nonoverlapping(data.as_ptr(), len);
                        ImageSourceUnsafe::Path(data, len)
                    } else {
                        ImageSourceUnsafe::Path(NonNull::dangling(), 0)
                    }
                },
                ImageSource::Id(id) => ImageSourceUnsafe::Id(id)
            };
            let id = reaction.id();
            ui.paint(move |painter, row| {
                painter
                    .image(
                        id,
                        unsafe { source.as_image_source() },
                        offset + vec2(0.0, row.height * 0.5 - size.y * 0.5),
                        size
                    );
            });
        }));
        reaction
    }

    #[inline(always)]
    pub fn standard_interact_height(&self) -> f32 {
        (self.style.default_handle_radius() + self.style.item_pad_inner().y) * 2.0
    }
}

impl<'a, 'b, Surface, Style> Drop for UiCtx<'a, 'b, Surface, Style>
    where 
        Surface: UiReactSurface,
        Style: UiStyle,
{
    fn drop(&mut self) {
        if !self.is_collapsing() {
            self.end_row();
            self.surface.get_mut().as_mut().ui_surface_mut().set_widget_rect_max(vec2(self.min_width, self.widget_off.y));
        }
    }
}

pub struct UiReactCtx<'a, 'b, 'c, Surface, Style>
    where 
        Surface: UiReactSurface,
        Style: UiStyle,
{
    ui: &'a mut UiCtx<'b, 'c, Surface, Style>,
    surface: &'a mut Surface::Surface
}

pub trait UiReact {

    type Style: UiStyle;

    fn win_ctx(&mut self) -> &mut WindowCtx;

    fn style(&self) -> &Self::Style;

    fn surface_moving(&self) -> bool;

    fn paint(&mut self, f: impl FnMut(&mut Painter, Row) + 'static);

    fn reaction_text(&mut self, id: ReactionId, text: &str) -> SharedText;

    fn add_text(&mut self, text: SharedText);
    
    fn render_text(&mut self, f: impl FnMut(&Self::Style, &mut TextRenderer));

    fn reaction_data<T: 'static>(&mut self, id: ReactionId, new: impl FnMut() -> T, modify: impl FnMut(&mut Self, &mut T));

    fn animated_bool(&mut self, id: ReactionId, value: bool) -> f32;

    fn font_height(&mut self) -> f32;

    fn standard_interact_height(&self) -> f32;

    fn tmp_data<T>(&mut self, count: usize) -> Option<NonNull<T>>;
}

impl<'a, 'b, 'c, Surface, Style> UiReact for UiReactCtx<'a, 'b, 'c, Surface, Style>
    where
        Surface: UiReactSurface,
        Style: UiStyle,
{

    type Style = Style;

    #[inline(always)]
    fn win_ctx(&mut self) -> &mut WindowCtx {
        self.ui.win_ctx
    }

    #[inline(always)]
    fn style(&self) -> &Self::Style {
        self.ui.style
    }

    #[inline(always)]
    fn surface_moving(&self) -> bool {
        self.surface.moving()
    }

    #[inline(always)]
    fn paint(&mut self, f: impl FnMut(&mut Painter, Row) + 'static) {
        unsafe {
            if let Some(ptr) = self.surface.tmp_data(1) {
                ptr.write(f);
                self.ui.current_row_paints
                    .as_mut()
                    .unwrap()
                    .push(ptr);
            }
        }
    }

    #[inline(always)]
    fn reaction_text(
        &mut self,
        id: ReactionId,
        text: &str,
    ) -> SharedText
    {
        let text = self.surface.reaction_text(self.ui.style, self.ui.text_renderer, id, text);
        text.edit(|text| text.reset())
    }

    #[inline(always)]
    fn add_text(&mut self, text: SharedText) {
        let index = self.surface.add_text(text);
        self.ui.current_row_text.push(RowText::new(index, 0, 0, None));
    }

    #[inline(always)]
    fn render_text(&mut self, mut f: impl FnMut(&Style, &mut TextRenderer)) {
        f(self.ui.style, self.ui.text_renderer);
    }

    #[inline(always)]
    fn reaction_data<T: 'static>(
        &mut self,
        id: ReactionId,
        new: impl FnMut() -> T,
        mut modify: impl FnMut(&mut Self, &mut T)
    )
    {
        if let Some(data) = unsafe {
            self.surface
                .reaction_data_or_insert_with(id, new)
                .map(|mut v| v.as_mut())
            }
        {
            modify(self, data);
        }
    }

    #[inline(always)]
    fn animated_bool(&mut self, id: ReactionId, value: bool) -> f32 {
        self.surface.animated_bool(id, value)
    }

    #[inline(always)]
    fn font_height(&mut self) -> f32 {
        self.ui.style.calc_font_height(self.ui.text_renderer)
    }

    #[inline(always)]
    fn standard_interact_height(&self) -> f32 {
        (self.ui.style.default_handle_radius() + self.ui.style.item_pad_inner().y) * 2.0
    }

    #[inline(always)]
    fn tmp_data<T>(&mut self, count: usize) -> Option<NonNull<T>> {
        self.surface.tmp_data(count)
    }
}

#[macro_export]
macro_rules! impl_widget {
    ($show:ident, $fn:ident( 
        $first:ident: $first_ty:ty,
        $($arg:ident: $arg_ty:ty),*
        $(,)?
    )) => {
        fn $fn<'a, Surface: $crate::UiReactSurface, Style: $crate::UiStyle>(
            $first: $first_ty,
            $($arg: $arg_ty),*
        ) -> ($first_ty, impl FnMut(&mut UiReactCtx<Surface, Style>, &mut Reaction, $first_ty))
        {
            (
                $first,
                |ui, reaction, $first|
                    $show(
                        ui,
                        reaction,
                        $first,
                        $($arg),*
                    )
            )
        }
    };
}
