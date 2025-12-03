#[derive(Clone, Copy, Debug)]
pub enum CapacityError {
    FixedCapacity {
        capacity: usize,
    },
    AllocFailed {
        new_capacity: usize,
    },
    IndexOutOfBounds {
        index: usize,
        len: usize,
    },
    MaxCapacityExceeded { max_capacity: usize, },
    ZeroSizedElement,
}

impl core::fmt::Display for CapacityError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FixedCapacity { capacity } => {
                write!(f, "exceeded fixed capacity of {}", capacity)
            },
            Self::AllocFailed { new_capacity } => {
                write!(f, "allocation failed with new capacity {}", new_capacity)
            },
            Self::IndexOutOfBounds { index, len } => {
                write!(f, "index {} was out of bounds of len {}", index, len)
            },
            Self::MaxCapacityExceeded { max_capacity } => {
                write!(f, "exceeded maximum capacity of {}", max_capacity)
            },
            Self::ZeroSizedElement => {
                write!(f, "size of element type is zero")
            },
        }
    }
}

impl core::error::Error for CapacityError {}
