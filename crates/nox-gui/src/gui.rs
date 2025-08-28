use nox_mem::slot_map::{GlobalSlotMap, SlotIndex};

use crate::*;

pub enum Var {
    Rect(SlotIndex<Rect>),
}

pub struct Gui {
    vars_rect: GlobalSlotMap<Rect>,
}
