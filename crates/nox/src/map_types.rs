mod iter;
mod alloc_map;

use iter::{Iter, IterMut};

pub use alloc_map::AllocMap;

pub type DynMap<'alloc, Key, Val, Alloc>
    = alloc_map::AllocMap<'alloc, Key, Val, Alloc, nox_mem::capacity_policy::Dyn>;

pub type FixedMap<'alloc, Key, Val, Alloc>
    = alloc_map::AllocMap<'alloc, Key, Val, Alloc, nox_mem::capacity_policy::Fixed>;
