mod iter;
mod alloc_map;

use iter::{Iter, IterMut};

pub use alloc_map::AllocMap;

pub type DynMap<'alloc, Key, Val, Alloc>
    = alloc_map::AllocMap<'alloc, Key, Val, Alloc, crate::vec_types::Dyn>;

pub type FixedMap<'alloc, Key, Val, Alloc>
    = alloc_map::AllocMap<'alloc, Key, Val, Alloc, crate::vec_types::Fixed>;
