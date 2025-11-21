use nox::mem::vec_types::GlobalVec;
use rustc_hash::FxHashMap;

use crate::*;

pub struct OnTopContents {
    reactions: FxHashMap<ReactionId, >
    painter_storage: PainterStorage,
    children: GlobalVec<OnTopContents>,
    flags: u32,
}
