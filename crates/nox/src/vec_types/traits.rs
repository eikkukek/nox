mod strategies;
mod capacity_policy;
mod vector;

pub use strategies::{MemoryStrategy, DuplicateStrategy};
pub use capacity_policy::{CapacityPolicy, CapacityError, Dyn, Fixed};
pub use vector::Vector;
