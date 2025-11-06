use compact_str::CompactString;

use nox::{mem::Hashable, *};

use nox_geom::*;

use crate::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReactionId(pub Hashable<f64>);

#[derive(Clone)]
pub struct Reaction {
    pub offset: Vec2,
    pub size: Vec2,
    id: ReactionId,
    hover_text: Option<CompactString>,
    flags: u32,
}

impl Reaction {

    const CLICKED: u32 = 0x1;
    const HELD: u32 = 0x2;
    const HOVERED: u32 = 0x4;
    const ANIMATED_BOOL: u32 = 0x8;

    #[inline(always)]
    pub fn new(
        id: ReactionId,
    ) -> Self {
        Self {
            offset: Default::default(),
            size: Default::default(),
            id,
            hover_text: None,
            flags: 0,
        }
    }

    #[inline(always)]
    pub fn hover_text(&mut self, text: &str) {
        self.hover_text = Some(CompactString::new(text));
    }

    #[inline(always)]
    pub fn id(&self) -> ReactionId {
        self.id
    }

    #[inline(always)]
    pub fn clicked(&self) -> bool {
        self.flags & Self::CLICKED == Self::CLICKED
    }

    #[inline(always)]
    pub fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    pub fn hovered(&self) -> bool {
        self.flags & Self::HOVERED == Self::HOVERED
    }

    #[inline(always)]
    pub fn animated_bool(&self) -> bool {
        self.flags & Self::ANIMATED_BOOL == Self::ANIMATED_BOOL
    }

    #[inline(always)]
    pub fn enable_animated_bool(&mut self) {
        self.flags |= Self::ANIMATED_BOOL;
    }

    #[inline(always)]
    pub fn get_size(&self) -> Vec2 {
        self.size
    }

    #[inline(always)]
    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    #[inline(always)]
    pub fn get_offset(&self) -> Vec2 {
        self.offset
    }

    #[inline(always)]
    pub fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }

    #[inline(always)]
    pub fn update<I: Interface>(
        &mut self,
        nox: &mut Nox<I>,
        cursor_pos: Vec2,
        window_pos: Vec2,
        cursor_in_window: bool,
        hover_blocked: bool,
    ) -> Option<CompactString>
    {
        self.flags &= !(Self::CLICKED | Self::HOVERED | Self::ANIMATED_BOOL);
        let cursor_in_self = BoundingRect::from_position_size(
            window_pos + self.offset, self.size
        ).is_point_inside(cursor_pos);
        if self.held() {
            if nox.was_mouse_button_released(MouseButton::Left) {
                self.flags &= !Self::HELD;
                or_flag!(self.flags, Self::CLICKED, cursor_in_self);
            }
        } else if cursor_in_self && !hover_blocked && cursor_in_window {
            self.flags |= Self::HOVERED;
            or_flag!(self.flags, Self::HELD, nox.was_mouse_button_pressed(MouseButton::Left));
        }
        if self.hovered() {
            if let Some(hover_text) = self.hover_text.take() {
                return Some(hover_text)
            }
        }
        None
    }
}
