mod alloc_map;
//mod fixed_map;

pub use alloc_map::AllocMap;
//pub use fixed_map::FixedMap;

pub type DynMap<'alloc, Key, Val, Alloc>
    = alloc_map::AllocMap<'alloc, Key, Val, Alloc, crate::vec_types::Dyn>;

pub type FixedMap<'alloc, Key, Val, Alloc>
    = alloc_map::AllocMap<'alloc, Key, Val, Alloc, crate::vec_types::Fixed>;
