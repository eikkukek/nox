pub use core::{
    ops::{
        BitOr, BitOrAssign,
        BitAnd, BitAndAssign,
    },
    fmt::{self, Debug, Display},
};

use nox_ash::vk;

use nox_mem::num::Integer;

/// This struct describes where [`Command`] dependencies are waited on. Specifically, what the
/// wait stage mask will be for the wait semaphore signaled by the dependency.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct MemoryDependencyHint(u64);

impl MemoryDependencyHint {

    /// Setting this flag means that the wait stage will be set to the earliest possible value.
    ///
    /// The default value of [`MemoryDependencyHint`].
    pub const NONE: MemoryDependencyHint =
        MemoryDependencyHint(0);

    /// The stage where vertex and index buffers are consumed.
    pub const VERTEX_INPUT: MemoryDependencyHint =
        MemoryDependencyHint(vk::PipelineStageFlags2::VERTEX_INPUT.as_raw());

    /// The stage where vertex shaders execute.
    pub const VERTEX_SHADER: MemoryDependencyHint =
        MemoryDependencyHint(vk::PipelineStageFlags2::VERTEX_SHADER.as_raw());

    /// The stage where fragment shaders execute.
    pub const FRAGMENT_SHADER: MemoryDependencyHint =
        MemoryDependencyHint(vk::PipelineStageFlags2::FRAGMENT_SHADER.as_raw());

    /// The stage where late fragment tests and depth/stencil store operations take place.
    pub const DEPTH_STENCIL_OUTPUT: MemoryDependencyHint =
        MemoryDependencyHint(vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS.as_raw());

    /// The stage where colors are output from a graphics pipeline.
    pub const COLOR_OUTPUT: MemoryDependencyHint =
        MemoryDependencyHint(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT.as_raw());

    /// The stage where compute shaders execute.
    pub const COMPUTE_SHADER: MemoryDependencyHint =
        MemoryDependencyHint(vk::PipelineStageFlags2::COMPUTE_SHADER.as_raw());

    /// The stage where all transfer commands execute.
    pub const TRANSFER: MemoryDependencyHint =
        MemoryDependencyHint(vk::PipelineStageFlags2::TRANSFER.as_raw());

    const FLAG_LOOK_UP: [&'static str; 64] = {
        let mut names = [""; 64];
        names[Self::VERTEX_INPUT.0.trailing_zeros() as usize]
            = "Vertex Input";
        names[Self::VERTEX_SHADER.0.trailing_zeros() as usize]
            = "Vertex Shader";
        names[Self::FRAGMENT_SHADER.0.trailing_zeros() as usize]
            = "Fragment Shader";
        names[Self::DEPTH_STENCIL_OUTPUT.0.trailing_zeros() as usize]
            = "Depth Stencil Output";
        names[Self::COLOR_OUTPUT.0.trailing_zeros() as usize]
            = "Color Output";
        names[Self::COMPUTE_SHADER.0.trailing_zeros() as usize]
            = "Compute Shader";
        names[Self::TRANSFER.0.trailing_zeros() as usize]
            = "Transfer";
        names
    };

    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for MemoryDependencyHint {
    
    type Output = MemoryDependencyHint;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        MemoryDependencyHint(self.0 | rhs.0)
    }
}

impl BitOrAssign for MemoryDependencyHint {

    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for MemoryDependencyHint {

    type Output = MemoryDependencyHint;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        MemoryDependencyHint(self.0 & rhs.0)
    }
}

impl BitAndAssign for MemoryDependencyHint {

    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Debug for MemoryDependencyHint {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#b}", self.0)
    }
}

impl Display for MemoryDependencyHint {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "[None]")
        } else {
            let mut iter = self.0.bit_iter();
            let bit = unsafe {
                iter.next().unwrap_unchecked()
            };
            write!(f, "[{}", Self::FLAG_LOOK_UP[bit.trailing_zeros() as usize])?;
            for bit in iter {
                write!(f, " | {}", Self::FLAG_LOOK_UP[bit.trailing_zeros() as usize])?;
            }
            write!(f, "]")?;
            Ok(())
        }
    }
}
