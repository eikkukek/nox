use compact_str::CompactString;

use nox::*;

use nox_geom::*;

use crate::*;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReactionId(pub usize);

#[derive(Clone)]
pub struct Reaction {
    pub offset: Vec2,
    pub size: Vec2,
    rel_cursor_pos: Vec2,
    cursor: Option<CursorIcon>,
    id: ReactionId,
    hover_text: Option<CompactString>,
    flags: u32,
}

impl Reaction {

    const CLICKED: u32 = 0x1;
    const HELD: u32 = 0x2;
    const HOVERED: u32 = 0x4;
    const ANIMATED_BOOL: u32 = 0x8;
    const HOVER_BLOCKED: u32 = 0x10;

    #[inline(always)]
    pub fn new(
        id: ReactionId,
    ) -> Self {
        Self {
            offset: Default::default(),
            size: Default::default(),
            rel_cursor_pos: Default::default(),
            cursor: None,
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
    pub fn take_hover_text(&mut self) -> Option<CompactString> {
        self.hover_text.take()
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
    pub fn hover_blocked(&self) -> bool {
        self.flags & Self::HOVER_BLOCKED == Self::HOVER_BLOCKED
    }

    #[inline(always)]
    pub(crate) fn enable_animated_bool(&mut self) {
        self.flags |= Self::ANIMATED_BOOL;
    }

    #[inline(always)]
    pub fn size(&self) -> Vec2 {
        self.size
    }

    #[inline(always)]
    pub fn offset(&self) -> Vec2 {
        self.offset
    }

    #[inline(always)]
    pub fn rel_cursor_pos(&self) -> Vec2 {
        self.rel_cursor_pos
    }

    #[inline(always)]
    pub fn cursor(&mut self, cursor: CursorIcon) {
        self.cursor = Some(cursor);
    }

    #[inline(always)]
    pub fn take_cursor(&mut self) -> Option<CursorIcon> {
        self.cursor.take()
    }

    #[inline(always)]
    pub fn update(
        &mut self,
        ctx: &WindowCtx,
        cursor_pos: Vec2,
        surface_pos: Vec2,
        cursor_in_window: bool,
        hover_blocked: bool,
    ) -> Option<CompactString>
    {
        self.flags &= !(
            Self::CLICKED |
            Self::HOVERED |
            Self::ANIMATED_BOOL |
            Self::HOVER_BLOCKED
        );
        or_flag!(self.flags, Self::HOVER_BLOCKED, hover_blocked);
        let rel_cursor_pos = cursor_pos - surface_pos;
        let cursor_in_self = BoundingRect::from_position_size(
            self.offset, self.size
        ).is_point_inside(rel_cursor_pos);
        let mouse_left_state = ctx.mouse_button_state(MouseButton::Left);
        if self.held() {
            if mouse_left_state.released() {
                self.flags &= !Self::HELD;
                or_flag!(self.flags, Self::CLICKED, cursor_in_self);
            }
        } else if cursor_in_self && !hover_blocked && cursor_in_window {
            self.flags |= Self::HOVERED;
            or_flag!(self.flags, Self::HELD, mouse_left_state.pressed());
        }
        self.rel_cursor_pos = rel_cursor_pos - self.offset;
        if self.hovered() {
            if let Some(hover_text) = self.hover_text.take() {
                return Some(hover_text)
            }
        }
        None
    }
}
