#![doc = r" SPIR-V enums"]
#![doc = r""]
#![doc = r" This file is auto-generated, do not modify manually."]
use crate::{core::*, module::*, op::*, stream::*};
use core::fmt::{self, Display};
use core::ops::{BitAnd, BitOr};
struct BitFmt(u32);
impl BitFmt {
    pub fn fmt(&mut self, value: u32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 != 0 {
            write!(f, "|")?;
        }
        self.0 |= value;
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageOperands(pub(crate) u32);
impl ImageOperands {
    pub const NONE: Self = Self(0x0000);
    pub const BIAS: Self = Self(0x0001);
    pub const LOD: Self = Self(0x0002);
    pub const GRAD: Self = Self(0x0004);
    pub const CONST_OFFSET: Self = Self(0x0008);
    pub const OFFSET: Self = Self(0x0010);
    pub const CONST_OFFSETS: Self = Self(0x0020);
    pub const SAMPLE: Self = Self(0x0040);
    pub const MIN_LOD: Self = Self(0x0080);
    pub const MAKE_TEXEL_AVAILABLE: Self = Self(0x0100);
    pub const MAKE_TEXEL_VISIBLE: Self = Self(0x0200);
    pub const NON_PRIVATE_TEXEL: Self = Self(0x0400);
    pub const VOLATILE_TEXEL: Self = Self(0x0800);
    pub const SIGN_EXTEND: Self = Self(0x1000);
    pub const ZERO_EXTEND: Self = Self(0x2000);
    pub const NONTEMPORAL: Self = Self(0x4000);
    pub const OFFSETS: Self = Self(0x10000);
}
impl Word for ImageOperands {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for ImageOperands {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for ImageOperands {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for ImageOperands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::BIAS == Self::BIAS {
            bitfmt.fmt(Self::BIAS.0, f)?;
            write!(f, "Bias")?;
        }
        if *self & Self::LOD == Self::LOD {
            bitfmt.fmt(Self::LOD.0, f)?;
            write!(f, "Lod")?;
        }
        if *self & Self::GRAD == Self::GRAD {
            bitfmt.fmt(Self::GRAD.0, f)?;
            write!(f, "Grad")?;
        }
        if *self & Self::CONST_OFFSET == Self::CONST_OFFSET {
            bitfmt.fmt(Self::CONST_OFFSET.0, f)?;
            write!(f, "ConstOffset")?;
        }
        if *self & Self::OFFSET == Self::OFFSET {
            bitfmt.fmt(Self::OFFSET.0, f)?;
            write!(f, "Offset")?;
        }
        if *self & Self::CONST_OFFSETS == Self::CONST_OFFSETS {
            bitfmt.fmt(Self::CONST_OFFSETS.0, f)?;
            write!(f, "ConstOffsets")?;
        }
        if *self & Self::SAMPLE == Self::SAMPLE {
            bitfmt.fmt(Self::SAMPLE.0, f)?;
            write!(f, "Sample")?;
        }
        if *self & Self::MIN_LOD == Self::MIN_LOD {
            bitfmt.fmt(Self::MIN_LOD.0, f)?;
            write!(f, "MinLod")?;
        }
        if *self & Self::MAKE_TEXEL_AVAILABLE == Self::MAKE_TEXEL_AVAILABLE {
            bitfmt.fmt(Self::MAKE_TEXEL_AVAILABLE.0, f)?;
            write!(f, "MakeTexelAvailable")?;
        }
        if *self & Self::MAKE_TEXEL_VISIBLE == Self::MAKE_TEXEL_VISIBLE {
            bitfmt.fmt(Self::MAKE_TEXEL_VISIBLE.0, f)?;
            write!(f, "MakeTexelVisible")?;
        }
        if *self & Self::NON_PRIVATE_TEXEL == Self::NON_PRIVATE_TEXEL {
            bitfmt.fmt(Self::NON_PRIVATE_TEXEL.0, f)?;
            write!(f, "NonPrivateTexel")?;
        }
        if *self & Self::VOLATILE_TEXEL == Self::VOLATILE_TEXEL {
            bitfmt.fmt(Self::VOLATILE_TEXEL.0, f)?;
            write!(f, "VolatileTexel")?;
        }
        if *self & Self::SIGN_EXTEND == Self::SIGN_EXTEND {
            bitfmt.fmt(Self::SIGN_EXTEND.0, f)?;
            write!(f, "SignExtend")?;
        }
        if *self & Self::ZERO_EXTEND == Self::ZERO_EXTEND {
            bitfmt.fmt(Self::ZERO_EXTEND.0, f)?;
            write!(f, "ZeroExtend")?;
        }
        if *self & Self::NONTEMPORAL == Self::NONTEMPORAL {
            bitfmt.fmt(Self::NONTEMPORAL.0, f)?;
            write!(f, "Nontemporal")?;
        }
        if *self & Self::OFFSETS == Self::OFFSETS {
            bitfmt.fmt(Self::OFFSETS.0, f)?;
            write!(f, "Offsets")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FPFastMathMode(pub(crate) u32);
impl FPFastMathMode {
    pub const NONE: Self = Self(0x0000);
    pub const NOT_NA_N: Self = Self(0x0001);
    pub const NOT_INF: Self = Self(0x0002);
    pub const NSZ: Self = Self(0x0004);
    pub const ALLOW_RECIP: Self = Self(0x0008);
    pub const FAST: Self = Self(0x0010);
    pub const ALLOW_CONTRACT: Self = Self(0x10000);
    pub const ALLOW_REASSOC: Self = Self(0x20000);
    pub const ALLOW_TRANSFORM: Self = Self(0x40000);
}
impl Word for FPFastMathMode {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for FPFastMathMode {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for FPFastMathMode {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for FPFastMathMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::NOT_NA_N == Self::NOT_NA_N {
            bitfmt.fmt(Self::NOT_NA_N.0, f)?;
            write!(f, "NotNaN")?;
        }
        if *self & Self::NOT_INF == Self::NOT_INF {
            bitfmt.fmt(Self::NOT_INF.0, f)?;
            write!(f, "NotInf")?;
        }
        if *self & Self::NSZ == Self::NSZ {
            bitfmt.fmt(Self::NSZ.0, f)?;
            write!(f, "NSZ")?;
        }
        if *self & Self::ALLOW_RECIP == Self::ALLOW_RECIP {
            bitfmt.fmt(Self::ALLOW_RECIP.0, f)?;
            write!(f, "AllowRecip")?;
        }
        if *self & Self::FAST == Self::FAST {
            bitfmt.fmt(Self::FAST.0, f)?;
            write!(f, "Fast")?;
        }
        if *self & Self::ALLOW_CONTRACT == Self::ALLOW_CONTRACT {
            bitfmt.fmt(Self::ALLOW_CONTRACT.0, f)?;
            write!(f, "AllowContract")?;
        }
        if *self & Self::ALLOW_REASSOC == Self::ALLOW_REASSOC {
            bitfmt.fmt(Self::ALLOW_REASSOC.0, f)?;
            write!(f, "AllowReassoc")?;
        }
        if *self & Self::ALLOW_TRANSFORM == Self::ALLOW_TRANSFORM {
            bitfmt.fmt(Self::ALLOW_TRANSFORM.0, f)?;
            write!(f, "AllowTransform")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SelectionControl(pub(crate) u32);
impl SelectionControl {
    pub const NONE: Self = Self(0x0000);
    pub const FLATTEN: Self = Self(0x0001);
    pub const DONT_FLATTEN: Self = Self(0x0002);
}
impl Word for SelectionControl {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for SelectionControl {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for SelectionControl {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for SelectionControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::FLATTEN == Self::FLATTEN {
            bitfmt.fmt(Self::FLATTEN.0, f)?;
            write!(f, "Flatten")?;
        }
        if *self & Self::DONT_FLATTEN == Self::DONT_FLATTEN {
            bitfmt.fmt(Self::DONT_FLATTEN.0, f)?;
            write!(f, "DontFlatten")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LoopControl(pub(crate) u32);
impl LoopControl {
    pub const NONE: Self = Self(0x0000);
    pub const UNROLL: Self = Self(0x0001);
    pub const DONT_UNROLL: Self = Self(0x0002);
    pub const DEPENDENCY_INFINITE: Self = Self(0x0004);
    pub const DEPENDENCY_LENGTH: Self = Self(0x0008);
    pub const MIN_ITERATIONS: Self = Self(0x0010);
    pub const MAX_ITERATIONS: Self = Self(0x0020);
    pub const ITERATION_MULTIPLE: Self = Self(0x0040);
    pub const PEEL_COUNT: Self = Self(0x0080);
    pub const PARTIAL_COUNT: Self = Self(0x0100);
    pub const INITIATION_INTERVAL_ALTERA: Self = Self(0x10000);
    pub const MAX_CONCURRENCY_ALTERA: Self = Self(0x20000);
    pub const DEPENDENCY_ARRAY_ALTERA: Self = Self(0x40000);
    pub const PIPELINE_ENABLE_ALTERA: Self = Self(0x80000);
    pub const LOOP_COALESCE_ALTERA: Self = Self(0x100000);
    pub const MAX_INTERLEAVING_ALTERA: Self = Self(0x200000);
    pub const SPECULATED_ITERATIONS_ALTERA: Self = Self(0x400000);
    pub const NO_FUSION_ALTERA: Self = Self(0x800000);
    pub const LOOP_COUNT_ALTERA: Self = Self(0x1000000);
    pub const MAX_REINVOCATION_DELAY_ALTERA: Self = Self(0x2000000);
}
impl Word for LoopControl {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for LoopControl {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for LoopControl {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for LoopControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::UNROLL == Self::UNROLL {
            bitfmt.fmt(Self::UNROLL.0, f)?;
            write!(f, "Unroll")?;
        }
        if *self & Self::DONT_UNROLL == Self::DONT_UNROLL {
            bitfmt.fmt(Self::DONT_UNROLL.0, f)?;
            write!(f, "DontUnroll")?;
        }
        if *self & Self::DEPENDENCY_INFINITE == Self::DEPENDENCY_INFINITE {
            bitfmt.fmt(Self::DEPENDENCY_INFINITE.0, f)?;
            write!(f, "DependencyInfinite")?;
        }
        if *self & Self::DEPENDENCY_LENGTH == Self::DEPENDENCY_LENGTH {
            bitfmt.fmt(Self::DEPENDENCY_LENGTH.0, f)?;
            write!(f, "DependencyLength")?;
        }
        if *self & Self::MIN_ITERATIONS == Self::MIN_ITERATIONS {
            bitfmt.fmt(Self::MIN_ITERATIONS.0, f)?;
            write!(f, "MinIterations")?;
        }
        if *self & Self::MAX_ITERATIONS == Self::MAX_ITERATIONS {
            bitfmt.fmt(Self::MAX_ITERATIONS.0, f)?;
            write!(f, "MaxIterations")?;
        }
        if *self & Self::ITERATION_MULTIPLE == Self::ITERATION_MULTIPLE {
            bitfmt.fmt(Self::ITERATION_MULTIPLE.0, f)?;
            write!(f, "IterationMultiple")?;
        }
        if *self & Self::PEEL_COUNT == Self::PEEL_COUNT {
            bitfmt.fmt(Self::PEEL_COUNT.0, f)?;
            write!(f, "PeelCount")?;
        }
        if *self & Self::PARTIAL_COUNT == Self::PARTIAL_COUNT {
            bitfmt.fmt(Self::PARTIAL_COUNT.0, f)?;
            write!(f, "PartialCount")?;
        }
        if *self & Self::INITIATION_INTERVAL_ALTERA == Self::INITIATION_INTERVAL_ALTERA {
            bitfmt.fmt(Self::INITIATION_INTERVAL_ALTERA.0, f)?;
            write!(f, "InitiationIntervalALTERA")?;
        }
        if *self & Self::MAX_CONCURRENCY_ALTERA == Self::MAX_CONCURRENCY_ALTERA {
            bitfmt.fmt(Self::MAX_CONCURRENCY_ALTERA.0, f)?;
            write!(f, "MaxConcurrencyALTERA")?;
        }
        if *self & Self::DEPENDENCY_ARRAY_ALTERA == Self::DEPENDENCY_ARRAY_ALTERA {
            bitfmt.fmt(Self::DEPENDENCY_ARRAY_ALTERA.0, f)?;
            write!(f, "DependencyArrayALTERA")?;
        }
        if *self & Self::PIPELINE_ENABLE_ALTERA == Self::PIPELINE_ENABLE_ALTERA {
            bitfmt.fmt(Self::PIPELINE_ENABLE_ALTERA.0, f)?;
            write!(f, "PipelineEnableALTERA")?;
        }
        if *self & Self::LOOP_COALESCE_ALTERA == Self::LOOP_COALESCE_ALTERA {
            bitfmt.fmt(Self::LOOP_COALESCE_ALTERA.0, f)?;
            write!(f, "LoopCoalesceALTERA")?;
        }
        if *self & Self::MAX_INTERLEAVING_ALTERA == Self::MAX_INTERLEAVING_ALTERA {
            bitfmt.fmt(Self::MAX_INTERLEAVING_ALTERA.0, f)?;
            write!(f, "MaxInterleavingALTERA")?;
        }
        if *self & Self::SPECULATED_ITERATIONS_ALTERA == Self::SPECULATED_ITERATIONS_ALTERA {
            bitfmt.fmt(Self::SPECULATED_ITERATIONS_ALTERA.0, f)?;
            write!(f, "SpeculatedIterationsALTERA")?;
        }
        if *self & Self::NO_FUSION_ALTERA == Self::NO_FUSION_ALTERA {
            bitfmt.fmt(Self::NO_FUSION_ALTERA.0, f)?;
            write!(f, "NoFusionALTERA")?;
        }
        if *self & Self::LOOP_COUNT_ALTERA == Self::LOOP_COUNT_ALTERA {
            bitfmt.fmt(Self::LOOP_COUNT_ALTERA.0, f)?;
            write!(f, "LoopCountALTERA")?;
        }
        if *self & Self::MAX_REINVOCATION_DELAY_ALTERA == Self::MAX_REINVOCATION_DELAY_ALTERA {
            bitfmt.fmt(Self::MAX_REINVOCATION_DELAY_ALTERA.0, f)?;
            write!(f, "MaxReinvocationDelayALTERA")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FunctionControl(pub(crate) u32);
impl FunctionControl {
    pub const NONE: Self = Self(0x0000);
    pub const INLINE: Self = Self(0x0001);
    pub const DONT_INLINE: Self = Self(0x0002);
    pub const PURE: Self = Self(0x0004);
    pub const CONST: Self = Self(0x0008);
    pub const OPT_NONE_EXT: Self = Self(0x10000);
}
impl Word for FunctionControl {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for FunctionControl {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for FunctionControl {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for FunctionControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::INLINE == Self::INLINE {
            bitfmt.fmt(Self::INLINE.0, f)?;
            write!(f, "Inline")?;
        }
        if *self & Self::DONT_INLINE == Self::DONT_INLINE {
            bitfmt.fmt(Self::DONT_INLINE.0, f)?;
            write!(f, "DontInline")?;
        }
        if *self & Self::PURE == Self::PURE {
            bitfmt.fmt(Self::PURE.0, f)?;
            write!(f, "Pure")?;
        }
        if *self & Self::CONST == Self::CONST {
            bitfmt.fmt(Self::CONST.0, f)?;
            write!(f, "Const")?;
        }
        if *self & Self::OPT_NONE_EXT == Self::OPT_NONE_EXT {
            bitfmt.fmt(Self::OPT_NONE_EXT.0, f)?;
            write!(f, "OptNoneEXT")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MemorySemantics(pub(crate) u32);
impl MemorySemantics {
    pub const RELAXED: Self = Self(0x0000);
    pub const ACQUIRE: Self = Self(0x0002);
    pub const RELEASE: Self = Self(0x0004);
    pub const ACQUIRE_RELEASE: Self = Self(0x0008);
    pub const SEQUENTIALLY_CONSISTENT: Self = Self(0x0010);
    pub const UNIFORM_MEMORY: Self = Self(0x0040);
    pub const SUBGROUP_MEMORY: Self = Self(0x0080);
    pub const WORKGROUP_MEMORY: Self = Self(0x0100);
    pub const CROSS_WORKGROUP_MEMORY: Self = Self(0x0200);
    pub const ATOMIC_COUNTER_MEMORY: Self = Self(0x0400);
    pub const IMAGE_MEMORY: Self = Self(0x0800);
    pub const OUTPUT_MEMORY: Self = Self(0x1000);
    pub const MAKE_AVAILABLE: Self = Self(0x2000);
    pub const MAKE_VISIBLE: Self = Self(0x4000);
    pub const VOLATILE: Self = Self(0x8000);
}
impl Word for MemorySemantics {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for MemorySemantics {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for MemorySemantics {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for MemorySemantics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::ACQUIRE == Self::ACQUIRE {
            bitfmt.fmt(Self::ACQUIRE.0, f)?;
            write!(f, "Acquire")?;
        }
        if *self & Self::RELEASE == Self::RELEASE {
            bitfmt.fmt(Self::RELEASE.0, f)?;
            write!(f, "Release")?;
        }
        if *self & Self::ACQUIRE_RELEASE == Self::ACQUIRE_RELEASE {
            bitfmt.fmt(Self::ACQUIRE_RELEASE.0, f)?;
            write!(f, "AcquireRelease")?;
        }
        if *self & Self::SEQUENTIALLY_CONSISTENT == Self::SEQUENTIALLY_CONSISTENT {
            bitfmt.fmt(Self::SEQUENTIALLY_CONSISTENT.0, f)?;
            write!(f, "SequentiallyConsistent")?;
        }
        if *self & Self::UNIFORM_MEMORY == Self::UNIFORM_MEMORY {
            bitfmt.fmt(Self::UNIFORM_MEMORY.0, f)?;
            write!(f, "UniformMemory")?;
        }
        if *self & Self::SUBGROUP_MEMORY == Self::SUBGROUP_MEMORY {
            bitfmt.fmt(Self::SUBGROUP_MEMORY.0, f)?;
            write!(f, "SubgroupMemory")?;
        }
        if *self & Self::WORKGROUP_MEMORY == Self::WORKGROUP_MEMORY {
            bitfmt.fmt(Self::WORKGROUP_MEMORY.0, f)?;
            write!(f, "WorkgroupMemory")?;
        }
        if *self & Self::CROSS_WORKGROUP_MEMORY == Self::CROSS_WORKGROUP_MEMORY {
            bitfmt.fmt(Self::CROSS_WORKGROUP_MEMORY.0, f)?;
            write!(f, "CrossWorkgroupMemory")?;
        }
        if *self & Self::ATOMIC_COUNTER_MEMORY == Self::ATOMIC_COUNTER_MEMORY {
            bitfmt.fmt(Self::ATOMIC_COUNTER_MEMORY.0, f)?;
            write!(f, "AtomicCounterMemory")?;
        }
        if *self & Self::IMAGE_MEMORY == Self::IMAGE_MEMORY {
            bitfmt.fmt(Self::IMAGE_MEMORY.0, f)?;
            write!(f, "ImageMemory")?;
        }
        if *self & Self::OUTPUT_MEMORY == Self::OUTPUT_MEMORY {
            bitfmt.fmt(Self::OUTPUT_MEMORY.0, f)?;
            write!(f, "OutputMemory")?;
        }
        if *self & Self::MAKE_AVAILABLE == Self::MAKE_AVAILABLE {
            bitfmt.fmt(Self::MAKE_AVAILABLE.0, f)?;
            write!(f, "MakeAvailable")?;
        }
        if *self & Self::MAKE_VISIBLE == Self::MAKE_VISIBLE {
            bitfmt.fmt(Self::MAKE_VISIBLE.0, f)?;
            write!(f, "MakeVisible")?;
        }
        if *self & Self::VOLATILE == Self::VOLATILE {
            bitfmt.fmt(Self::VOLATILE.0, f)?;
            write!(f, "Volatile")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MemoryAccess(pub(crate) u32);
impl MemoryAccess {
    pub const NONE: Self = Self(0x0000);
    pub const VOLATILE: Self = Self(0x0001);
    pub const ALIGNED: Self = Self(0x0002);
    pub const NONTEMPORAL: Self = Self(0x0004);
    pub const MAKE_POINTER_AVAILABLE: Self = Self(0x0008);
    pub const MAKE_POINTER_VISIBLE: Self = Self(0x0010);
    pub const NON_PRIVATE_POINTER: Self = Self(0x0020);
    pub const ALIAS_SCOPE_INTELMASK: Self = Self(0x10000);
    pub const NO_ALIAS_INTELMASK: Self = Self(0x20000);
}
impl Word for MemoryAccess {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for MemoryAccess {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for MemoryAccess {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for MemoryAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::VOLATILE == Self::VOLATILE {
            bitfmt.fmt(Self::VOLATILE.0, f)?;
            write!(f, "Volatile")?;
        }
        if *self & Self::ALIGNED == Self::ALIGNED {
            bitfmt.fmt(Self::ALIGNED.0, f)?;
            write!(f, "Aligned")?;
        }
        if *self & Self::NONTEMPORAL == Self::NONTEMPORAL {
            bitfmt.fmt(Self::NONTEMPORAL.0, f)?;
            write!(f, "Nontemporal")?;
        }
        if *self & Self::MAKE_POINTER_AVAILABLE == Self::MAKE_POINTER_AVAILABLE {
            bitfmt.fmt(Self::MAKE_POINTER_AVAILABLE.0, f)?;
            write!(f, "MakePointerAvailable")?;
        }
        if *self & Self::MAKE_POINTER_VISIBLE == Self::MAKE_POINTER_VISIBLE {
            bitfmt.fmt(Self::MAKE_POINTER_VISIBLE.0, f)?;
            write!(f, "MakePointerVisible")?;
        }
        if *self & Self::NON_PRIVATE_POINTER == Self::NON_PRIVATE_POINTER {
            bitfmt.fmt(Self::NON_PRIVATE_POINTER.0, f)?;
            write!(f, "NonPrivatePointer")?;
        }
        if *self & Self::ALIAS_SCOPE_INTELMASK == Self::ALIAS_SCOPE_INTELMASK {
            bitfmt.fmt(Self::ALIAS_SCOPE_INTELMASK.0, f)?;
            write!(f, "AliasScopeINTELMask")?;
        }
        if *self & Self::NO_ALIAS_INTELMASK == Self::NO_ALIAS_INTELMASK {
            bitfmt.fmt(Self::NO_ALIAS_INTELMASK.0, f)?;
            write!(f, "NoAliasINTELMask")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KernelProfilingInfo(pub(crate) u32);
impl KernelProfilingInfo {
    pub const NONE: Self = Self(0x0000);
    pub const CMD_EXEC_TIME: Self = Self(0x0001);
}
impl Word for KernelProfilingInfo {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for KernelProfilingInfo {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for KernelProfilingInfo {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for KernelProfilingInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::CMD_EXEC_TIME == Self::CMD_EXEC_TIME {
            bitfmt.fmt(Self::CMD_EXEC_TIME.0, f)?;
            write!(f, "CmdExecTime")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RayFlags(pub(crate) u32);
impl RayFlags {
    pub const NONE_KHR: Self = Self(0x0000);
    pub const OPAQUE_KHR: Self = Self(0x0001);
    pub const NO_OPAQUE_KHR: Self = Self(0x0002);
    pub const TERMINATE_ON_FIRST_HIT_KHR: Self = Self(0x0004);
    pub const SKIP_CLOSEST_HIT_SHADER_KHR: Self = Self(0x0008);
    pub const CULL_BACK_FACING_TRIANGLES_KHR: Self = Self(0x0010);
    pub const CULL_FRONT_FACING_TRIANGLES_KHR: Self = Self(0x0020);
    pub const CULL_OPAQUE_KHR: Self = Self(0x0040);
    pub const CULL_NO_OPAQUE_KHR: Self = Self(0x0080);
    pub const SKIP_TRIANGLES_KHR: Self = Self(0x0100);
    pub const SKIP_AABBS_KHR: Self = Self(0x0200);
    pub const FORCE_OPACITY_MICROMAP2_STATE_EXT: Self = Self(0x0400);
}
impl Word for RayFlags {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for RayFlags {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for RayFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for RayFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::OPAQUE_KHR == Self::OPAQUE_KHR {
            bitfmt.fmt(Self::OPAQUE_KHR.0, f)?;
            write!(f, "OpaqueKHR")?;
        }
        if *self & Self::NO_OPAQUE_KHR == Self::NO_OPAQUE_KHR {
            bitfmt.fmt(Self::NO_OPAQUE_KHR.0, f)?;
            write!(f, "NoOpaqueKHR")?;
        }
        if *self & Self::TERMINATE_ON_FIRST_HIT_KHR == Self::TERMINATE_ON_FIRST_HIT_KHR {
            bitfmt.fmt(Self::TERMINATE_ON_FIRST_HIT_KHR.0, f)?;
            write!(f, "TerminateOnFirstHitKHR")?;
        }
        if *self & Self::SKIP_CLOSEST_HIT_SHADER_KHR == Self::SKIP_CLOSEST_HIT_SHADER_KHR {
            bitfmt.fmt(Self::SKIP_CLOSEST_HIT_SHADER_KHR.0, f)?;
            write!(f, "SkipClosestHitShaderKHR")?;
        }
        if *self & Self::CULL_BACK_FACING_TRIANGLES_KHR == Self::CULL_BACK_FACING_TRIANGLES_KHR {
            bitfmt.fmt(Self::CULL_BACK_FACING_TRIANGLES_KHR.0, f)?;
            write!(f, "CullBackFacingTrianglesKHR")?;
        }
        if *self & Self::CULL_FRONT_FACING_TRIANGLES_KHR == Self::CULL_FRONT_FACING_TRIANGLES_KHR {
            bitfmt.fmt(Self::CULL_FRONT_FACING_TRIANGLES_KHR.0, f)?;
            write!(f, "CullFrontFacingTrianglesKHR")?;
        }
        if *self & Self::CULL_OPAQUE_KHR == Self::CULL_OPAQUE_KHR {
            bitfmt.fmt(Self::CULL_OPAQUE_KHR.0, f)?;
            write!(f, "CullOpaqueKHR")?;
        }
        if *self & Self::CULL_NO_OPAQUE_KHR == Self::CULL_NO_OPAQUE_KHR {
            bitfmt.fmt(Self::CULL_NO_OPAQUE_KHR.0, f)?;
            write!(f, "CullNoOpaqueKHR")?;
        }
        if *self & Self::SKIP_TRIANGLES_KHR == Self::SKIP_TRIANGLES_KHR {
            bitfmt.fmt(Self::SKIP_TRIANGLES_KHR.0, f)?;
            write!(f, "SkipTrianglesKHR")?;
        }
        if *self & Self::SKIP_AABBS_KHR == Self::SKIP_AABBS_KHR {
            bitfmt.fmt(Self::SKIP_AABBS_KHR.0, f)?;
            write!(f, "SkipAABBsKHR")?;
        }
        if *self & Self::FORCE_OPACITY_MICROMAP2_STATE_EXT
            == Self::FORCE_OPACITY_MICROMAP2_STATE_EXT
        {
            bitfmt.fmt(Self::FORCE_OPACITY_MICROMAP2_STATE_EXT.0, f)?;
            write!(f, "ForceOpacityMicromap2StateEXT")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FragmentShadingRate(pub(crate) u32);
impl FragmentShadingRate {
    pub const VERTICAL2_PIXELS: Self = Self(0x0001);
    pub const VERTICAL4_PIXELS: Self = Self(0x0002);
    pub const HORIZONTAL2_PIXELS: Self = Self(0x0004);
    pub const HORIZONTAL4_PIXELS: Self = Self(0x0008);
}
impl Word for FragmentShadingRate {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for FragmentShadingRate {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for FragmentShadingRate {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for FragmentShadingRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::VERTICAL2_PIXELS == Self::VERTICAL2_PIXELS {
            bitfmt.fmt(Self::VERTICAL2_PIXELS.0, f)?;
            write!(f, "Vertical2Pixels")?;
        }
        if *self & Self::VERTICAL4_PIXELS == Self::VERTICAL4_PIXELS {
            bitfmt.fmt(Self::VERTICAL4_PIXELS.0, f)?;
            write!(f, "Vertical4Pixels")?;
        }
        if *self & Self::HORIZONTAL2_PIXELS == Self::HORIZONTAL2_PIXELS {
            bitfmt.fmt(Self::HORIZONTAL2_PIXELS.0, f)?;
            write!(f, "Horizontal2Pixels")?;
        }
        if *self & Self::HORIZONTAL4_PIXELS == Self::HORIZONTAL4_PIXELS {
            bitfmt.fmt(Self::HORIZONTAL4_PIXELS.0, f)?;
            write!(f, "Horizontal4Pixels")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RawAccessChainOperands(pub(crate) u32);
impl RawAccessChainOperands {
    pub const NONE: Self = Self(0x0000);
    pub const ROBUSTNESS_PER_COMPONENT_NV: Self = Self(0x0001);
    pub const ROBUSTNESS_PER_ELEMENT_NV: Self = Self(0x0002);
}
impl Word for RawAccessChainOperands {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for RawAccessChainOperands {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for RawAccessChainOperands {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for RawAccessChainOperands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::ROBUSTNESS_PER_COMPONENT_NV == Self::ROBUSTNESS_PER_COMPONENT_NV {
            bitfmt.fmt(Self::ROBUSTNESS_PER_COMPONENT_NV.0, f)?;
            write!(f, "RobustnessPerComponentNV")?;
        }
        if *self & Self::ROBUSTNESS_PER_ELEMENT_NV == Self::ROBUSTNESS_PER_ELEMENT_NV {
            bitfmt.fmt(Self::ROBUSTNESS_PER_ELEMENT_NV.0, f)?;
            write!(f, "RobustnessPerElementNV")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SourceLanguage(pub(crate) u32);
impl SourceLanguage {
    pub const UNKNOWN: Self = Self(0u32);
    pub const ESSL: Self = Self(1u32);
    pub const GLSL: Self = Self(2u32);
    pub const OPEN_CL_C: Self = Self(3u32);
    pub const OPEN_CL_CPP: Self = Self(4u32);
    pub const HLSL: Self = Self(5u32);
    pub const CPP_FOR_OPEN_CL: Self = Self(6u32);
    pub const SYCL: Self = Self(7u32);
    pub const HERO_C: Self = Self(8u32);
    pub const NZSL: Self = Self(9u32);
    pub const WGSL: Self = Self(10u32);
    pub const SLANG: Self = Self(11u32);
    pub const ZIG: Self = Self(12u32);
    pub const RUST: Self = Self(13u32);
}
impl Word for SourceLanguage {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for SourceLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UNKNOWN => write!(f, "Unknown"),
            Self::ESSL => write!(f, "ESSL"),
            Self::GLSL => write!(f, "GLSL"),
            Self::OPEN_CL_C => write!(f, "OpenCL_C"),
            Self::OPEN_CL_CPP => write!(f, "OpenCL_CPP"),
            Self::HLSL => write!(f, "HLSL"),
            Self::CPP_FOR_OPEN_CL => write!(f, "CPP_for_OpenCL"),
            Self::SYCL => write!(f, "SYCL"),
            Self::HERO_C => write!(f, "HERO_C"),
            Self::NZSL => write!(f, "NZSL"),
            Self::WGSL => write!(f, "WGSL"),
            Self::SLANG => write!(f, "Slang"),
            Self::ZIG => write!(f, "Zig"),
            Self::RUST => write!(f, "Rust"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ExecutionModel(pub(crate) u32);
impl ExecutionModel {
    pub const VERTEX: Self = Self(0u32);
    pub const TESSELLATION_CONTROL: Self = Self(1u32);
    pub const TESSELLATION_EVALUATION: Self = Self(2u32);
    pub const GEOMETRY: Self = Self(3u32);
    pub const FRAGMENT: Self = Self(4u32);
    pub const GLCOMPUTE: Self = Self(5u32);
    pub const KERNEL: Self = Self(6u32);
    pub const TASK_NV: Self = Self(5267u32);
    pub const MESH_NV: Self = Self(5268u32);
    pub const RAY_GENERATION_KHR: Self = Self(5313u32);
    pub const INTERSECTION_KHR: Self = Self(5314u32);
    pub const ANY_HIT_KHR: Self = Self(5315u32);
    pub const CLOSEST_HIT_KHR: Self = Self(5316u32);
    pub const MISS_KHR: Self = Self(5317u32);
    pub const CALLABLE_KHR: Self = Self(5318u32);
    pub const TASK_EXT: Self = Self(5364u32);
    pub const MESH_EXT: Self = Self(5365u32);
}
impl Word for ExecutionModel {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for ExecutionModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::VERTEX => write!(f, "Vertex"),
            Self::TESSELLATION_CONTROL => write!(f, "TessellationControl"),
            Self::TESSELLATION_EVALUATION => write!(f, "TessellationEvaluation"),
            Self::GEOMETRY => write!(f, "Geometry"),
            Self::FRAGMENT => write!(f, "Fragment"),
            Self::GLCOMPUTE => write!(f, "GLCompute"),
            Self::KERNEL => write!(f, "Kernel"),
            Self::TASK_NV => write!(f, "TaskNV"),
            Self::MESH_NV => write!(f, "MeshNV"),
            Self::RAY_GENERATION_KHR => write!(f, "RayGenerationKHR"),
            Self::INTERSECTION_KHR => write!(f, "IntersectionKHR"),
            Self::ANY_HIT_KHR => write!(f, "AnyHitKHR"),
            Self::CLOSEST_HIT_KHR => write!(f, "ClosestHitKHR"),
            Self::MISS_KHR => write!(f, "MissKHR"),
            Self::CALLABLE_KHR => write!(f, "CallableKHR"),
            Self::TASK_EXT => write!(f, "TaskEXT"),
            Self::MESH_EXT => write!(f, "MeshEXT"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AddressingModel(pub(crate) u32);
impl AddressingModel {
    pub const LOGICAL: Self = Self(0u32);
    pub const PHYSICAL32: Self = Self(1u32);
    pub const PHYSICAL64: Self = Self(2u32);
    pub const PHYSICAL_STORAGE_BUFFER64: Self = Self(5348u32);
}
impl Word for AddressingModel {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for AddressingModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::LOGICAL => write!(f, "Logical"),
            Self::PHYSICAL32 => write!(f, "Physical32"),
            Self::PHYSICAL64 => write!(f, "Physical64"),
            Self::PHYSICAL_STORAGE_BUFFER64 => write!(f, "PhysicalStorageBuffer64"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MemoryModel(pub(crate) u32);
impl MemoryModel {
    pub const SIMPLE: Self = Self(0u32);
    pub const GLSL450: Self = Self(1u32);
    pub const OPEN_CL: Self = Self(2u32);
    pub const VULKAN: Self = Self(3u32);
}
impl Word for MemoryModel {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for MemoryModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::SIMPLE => write!(f, "Simple"),
            Self::GLSL450 => write!(f, "GLSL450"),
            Self::OPEN_CL => write!(f, "OpenCL"),
            Self::VULKAN => write!(f, "Vulkan"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum ExecutionMode {
    Invocations {
        number_of_invocations: LiteralInteger,
    },
    SpacingEqual,
    SpacingFractionalEven,
    SpacingFractionalOdd,
    VertexOrderCw,
    VertexOrderCcw,
    PixelCenterInteger,
    OriginUpperLeft,
    OriginLowerLeft,
    EarlyFragmentTests,
    PointMode,
    Xfb,
    DepthReplacing,
    DepthGreater,
    DepthLess,
    DepthUnchanged,
    LocalSize {
        x_size: LiteralInteger,
        y_size: LiteralInteger,
        z_size: LiteralInteger,
    },
    LocalSizeHint {
        x_size: LiteralInteger,
        y_size: LiteralInteger,
        z_size: LiteralInteger,
    },
    InputPoints,
    InputLines,
    InputLinesAdjacency,
    Triangles,
    InputTrianglesAdjacency,
    Quads,
    Isolines,
    OutputVertices {
        vertex_count: LiteralInteger,
    },
    OutputPoints,
    OutputLineStrip,
    OutputTriangleStrip,
    VecTypeHint {
        vector_type: LiteralInteger,
    },
    ContractionOff,
    Initializer,
    Finalizer,
    SubgroupSize {
        subgroup_size: LiteralInteger,
    },
    SubgroupsPerWorkgroup {
        subgroups_per_workgroup: LiteralInteger,
    },
    SubgroupsPerWorkgroupId {
        subgroups_per_workgroup: IdRef,
    },
    LocalSizeId {
        x_size: IdRef,
        y_size: IdRef,
        z_size: IdRef,
    },
    LocalSizeHintId {
        x_size_hint: IdRef,
        y_size_hint: IdRef,
        z_size_hint: IdRef,
    },
    NonCoherentColorAttachmentReadEXT,
    NonCoherentDepthAttachmentReadEXT,
    NonCoherentStencilAttachmentReadEXT,
    SubgroupUniformControlFlowKHR,
    PostDepthCoverage,
    DenormPreserve {
        target_width: LiteralInteger,
    },
    DenormFlushToZero {
        target_width: LiteralInteger,
    },
    SignedZeroInfNanPreserve {
        target_width: LiteralInteger,
    },
    RoundingModeRTE {
        target_width: LiteralInteger,
    },
    RoundingModeRTZ {
        target_width: LiteralInteger,
    },
    NonCoherentTileAttachmentReadQCOM,
    TileShadingRateQCOM {
        x_rate: LiteralInteger,
        y_rate: LiteralInteger,
        z_rate: LiteralInteger,
    },
    EarlyAndLateFragmentTestsAMD,
    StencilRefReplacingEXT,
    CoalescingAMDX,
    IsApiEntryAMDX {
        is_entry: IdRef,
    },
    MaxNodeRecursionAMDX {
        number_of_recursions: IdRef,
    },
    StaticNumWorkgroupsAMDX {
        x_size: IdRef,
        y_size: IdRef,
        z_size: IdRef,
    },
    ShaderIndexAMDX {
        shader_index: IdRef,
    },
    MaxNumWorkgroupsAMDX {
        x_size: IdRef,
        y_size: IdRef,
        z_size: IdRef,
    },
    StencilRefUnchangedFrontAMD,
    StencilRefGreaterFrontAMD,
    StencilRefLessFrontAMD,
    StencilRefUnchangedBackAMD,
    StencilRefGreaterBackAMD,
    StencilRefLessBackAMD,
    QuadDerivativesKHR,
    RequireFullQuadsKHR,
    SharesInputWithAMDX {
        node_name: IdRef,
        shader_index: IdRef,
    },
    ArithmeticPoisonKHR,
    OutputLinesEXT,
    OutputPrimitivesEXT {
        primitive_count: LiteralInteger,
    },
    DerivativeGroupQuadsKHR,
    DerivativeGroupLinearKHR,
    OutputTrianglesEXT,
    PixelInterlockOrderedEXT,
    PixelInterlockUnorderedEXT,
    SampleInterlockOrderedEXT,
    SampleInterlockUnorderedEXT,
    ShadingRateInterlockOrderedEXT,
    ShadingRateInterlockUnorderedEXT,
    Shader64BitIndexingEXT,
    SharedLocalMemorySizeINTEL {
        size: LiteralInteger,
    },
    RoundingModeRTPINTEL {
        target_width: LiteralInteger,
    },
    RoundingModeRTNINTEL {
        target_width: LiteralInteger,
    },
    FloatingPointModeALTINTEL {
        target_width: LiteralInteger,
    },
    FloatingPointModeIEEEINTEL {
        target_width: LiteralInteger,
    },
    MaxWorkgroupSizeINTEL {
        max_x_size: LiteralInteger,
        max_y_size: LiteralInteger,
        max_z_size: LiteralInteger,
    },
    MaxWorkDimINTEL {
        max_dimensions: LiteralInteger,
    },
    NoGlobalOffsetINTEL,
    NumSIMDWorkitemsINTEL {
        vector_width: LiteralInteger,
    },
    SchedulerTargetFmaxMhzINTEL {
        target_fmax: LiteralInteger,
    },
    MaximallyReconvergesKHR,
    FPFastMathDefault {
        target_type: IdRef,
        fast_math_mode: IdRef,
    },
    StreamingInterfaceINTEL {
        stall_free_return: LiteralInteger,
    },
    RegisterMapInterfaceINTEL {
        wait_for_done_write: LiteralInteger,
    },
    NamedBarrierCountINTEL {
        barrier_count: LiteralInteger,
    },
    MaximumRegistersINTEL {
        number_of_registers: LiteralInteger,
    },
    MaximumRegistersIdINTEL {
        number_of_registers: IdRef,
    },
    NamedMaximumRegistersINTEL {
        named_maximum_number_of_registers: NamedMaximumNumberOfRegisters,
    },
}
impl<'a> ExecutionMode {
    #[inline]
    pub fn parse_one(stream: &mut InstructionStream<'a>) -> ParseResult<Self> {
        let variant = stream.read()?;
        match variant {
            0u32 => Ok(Self::Invocations {
                number_of_invocations: LiteralInteger::parse_one(stream)?,
            }),
            1u32 => Ok(Self::SpacingEqual),
            2u32 => Ok(Self::SpacingFractionalEven),
            3u32 => Ok(Self::SpacingFractionalOdd),
            4u32 => Ok(Self::VertexOrderCw),
            5u32 => Ok(Self::VertexOrderCcw),
            6u32 => Ok(Self::PixelCenterInteger),
            7u32 => Ok(Self::OriginUpperLeft),
            8u32 => Ok(Self::OriginLowerLeft),
            9u32 => Ok(Self::EarlyFragmentTests),
            10u32 => Ok(Self::PointMode),
            11u32 => Ok(Self::Xfb),
            12u32 => Ok(Self::DepthReplacing),
            14u32 => Ok(Self::DepthGreater),
            15u32 => Ok(Self::DepthLess),
            16u32 => Ok(Self::DepthUnchanged),
            17u32 => Ok(Self::LocalSize {
                x_size: LiteralInteger::parse_one(stream)?,
                y_size: LiteralInteger::parse_one(stream)?,
                z_size: LiteralInteger::parse_one(stream)?,
            }),
            18u32 => Ok(Self::LocalSizeHint {
                x_size: LiteralInteger::parse_one(stream)?,
                y_size: LiteralInteger::parse_one(stream)?,
                z_size: LiteralInteger::parse_one(stream)?,
            }),
            19u32 => Ok(Self::InputPoints),
            20u32 => Ok(Self::InputLines),
            21u32 => Ok(Self::InputLinesAdjacency),
            22u32 => Ok(Self::Triangles),
            23u32 => Ok(Self::InputTrianglesAdjacency),
            24u32 => Ok(Self::Quads),
            25u32 => Ok(Self::Isolines),
            26u32 => Ok(Self::OutputVertices {
                vertex_count: LiteralInteger::parse_one(stream)?,
            }),
            27u32 => Ok(Self::OutputPoints),
            28u32 => Ok(Self::OutputLineStrip),
            29u32 => Ok(Self::OutputTriangleStrip),
            30u32 => Ok(Self::VecTypeHint {
                vector_type: LiteralInteger::parse_one(stream)?,
            }),
            31u32 => Ok(Self::ContractionOff),
            33u32 => Ok(Self::Initializer),
            34u32 => Ok(Self::Finalizer),
            35u32 => Ok(Self::SubgroupSize {
                subgroup_size: LiteralInteger::parse_one(stream)?,
            }),
            36u32 => Ok(Self::SubgroupsPerWorkgroup {
                subgroups_per_workgroup: LiteralInteger::parse_one(stream)?,
            }),
            37u32 => Ok(Self::SubgroupsPerWorkgroupId {
                subgroups_per_workgroup: IdRef::parse_one(stream)?,
            }),
            38u32 => Ok(Self::LocalSizeId {
                x_size: IdRef::parse_one(stream)?,
                y_size: IdRef::parse_one(stream)?,
                z_size: IdRef::parse_one(stream)?,
            }),
            39u32 => Ok(Self::LocalSizeHintId {
                x_size_hint: IdRef::parse_one(stream)?,
                y_size_hint: IdRef::parse_one(stream)?,
                z_size_hint: IdRef::parse_one(stream)?,
            }),
            4169u32 => Ok(Self::NonCoherentColorAttachmentReadEXT),
            4170u32 => Ok(Self::NonCoherentDepthAttachmentReadEXT),
            4171u32 => Ok(Self::NonCoherentStencilAttachmentReadEXT),
            4421u32 => Ok(Self::SubgroupUniformControlFlowKHR),
            4446u32 => Ok(Self::PostDepthCoverage),
            4459u32 => Ok(Self::DenormPreserve {
                target_width: LiteralInteger::parse_one(stream)?,
            }),
            4460u32 => Ok(Self::DenormFlushToZero {
                target_width: LiteralInteger::parse_one(stream)?,
            }),
            4461u32 => Ok(Self::SignedZeroInfNanPreserve {
                target_width: LiteralInteger::parse_one(stream)?,
            }),
            4462u32 => Ok(Self::RoundingModeRTE {
                target_width: LiteralInteger::parse_one(stream)?,
            }),
            4463u32 => Ok(Self::RoundingModeRTZ {
                target_width: LiteralInteger::parse_one(stream)?,
            }),
            4489u32 => Ok(Self::NonCoherentTileAttachmentReadQCOM),
            4490u32 => Ok(Self::TileShadingRateQCOM {
                x_rate: LiteralInteger::parse_one(stream)?,
                y_rate: LiteralInteger::parse_one(stream)?,
                z_rate: LiteralInteger::parse_one(stream)?,
            }),
            5017u32 => Ok(Self::EarlyAndLateFragmentTestsAMD),
            5027u32 => Ok(Self::StencilRefReplacingEXT),
            5069u32 => Ok(Self::CoalescingAMDX),
            5070u32 => Ok(Self::IsApiEntryAMDX {
                is_entry: IdRef::parse_one(stream)?,
            }),
            5071u32 => Ok(Self::MaxNodeRecursionAMDX {
                number_of_recursions: IdRef::parse_one(stream)?,
            }),
            5072u32 => Ok(Self::StaticNumWorkgroupsAMDX {
                x_size: IdRef::parse_one(stream)?,
                y_size: IdRef::parse_one(stream)?,
                z_size: IdRef::parse_one(stream)?,
            }),
            5073u32 => Ok(Self::ShaderIndexAMDX {
                shader_index: IdRef::parse_one(stream)?,
            }),
            5077u32 => Ok(Self::MaxNumWorkgroupsAMDX {
                x_size: IdRef::parse_one(stream)?,
                y_size: IdRef::parse_one(stream)?,
                z_size: IdRef::parse_one(stream)?,
            }),
            5079u32 => Ok(Self::StencilRefUnchangedFrontAMD),
            5080u32 => Ok(Self::StencilRefGreaterFrontAMD),
            5081u32 => Ok(Self::StencilRefLessFrontAMD),
            5082u32 => Ok(Self::StencilRefUnchangedBackAMD),
            5083u32 => Ok(Self::StencilRefGreaterBackAMD),
            5084u32 => Ok(Self::StencilRefLessBackAMD),
            5088u32 => Ok(Self::QuadDerivativesKHR),
            5089u32 => Ok(Self::RequireFullQuadsKHR),
            5102u32 => Ok(Self::SharesInputWithAMDX {
                node_name: IdRef::parse_one(stream)?,
                shader_index: IdRef::parse_one(stream)?,
            }),
            5157u32 => Ok(Self::ArithmeticPoisonKHR),
            5269u32 => Ok(Self::OutputLinesEXT),
            5270u32 => Ok(Self::OutputPrimitivesEXT {
                primitive_count: LiteralInteger::parse_one(stream)?,
            }),
            5289u32 => Ok(Self::DerivativeGroupQuadsKHR),
            5290u32 => Ok(Self::DerivativeGroupLinearKHR),
            5298u32 => Ok(Self::OutputTrianglesEXT),
            5366u32 => Ok(Self::PixelInterlockOrderedEXT),
            5367u32 => Ok(Self::PixelInterlockUnorderedEXT),
            5368u32 => Ok(Self::SampleInterlockOrderedEXT),
            5369u32 => Ok(Self::SampleInterlockUnorderedEXT),
            5370u32 => Ok(Self::ShadingRateInterlockOrderedEXT),
            5371u32 => Ok(Self::ShadingRateInterlockUnorderedEXT),
            5427u32 => Ok(Self::Shader64BitIndexingEXT),
            5618u32 => Ok(Self::SharedLocalMemorySizeINTEL {
                size: LiteralInteger::parse_one(stream)?,
            }),
            5620u32 => Ok(Self::RoundingModeRTPINTEL {
                target_width: LiteralInteger::parse_one(stream)?,
            }),
            5621u32 => Ok(Self::RoundingModeRTNINTEL {
                target_width: LiteralInteger::parse_one(stream)?,
            }),
            5622u32 => Ok(Self::FloatingPointModeALTINTEL {
                target_width: LiteralInteger::parse_one(stream)?,
            }),
            5623u32 => Ok(Self::FloatingPointModeIEEEINTEL {
                target_width: LiteralInteger::parse_one(stream)?,
            }),
            5893u32 => Ok(Self::MaxWorkgroupSizeINTEL {
                max_x_size: LiteralInteger::parse_one(stream)?,
                max_y_size: LiteralInteger::parse_one(stream)?,
                max_z_size: LiteralInteger::parse_one(stream)?,
            }),
            5894u32 => Ok(Self::MaxWorkDimINTEL {
                max_dimensions: LiteralInteger::parse_one(stream)?,
            }),
            5895u32 => Ok(Self::NoGlobalOffsetINTEL),
            5896u32 => Ok(Self::NumSIMDWorkitemsINTEL {
                vector_width: LiteralInteger::parse_one(stream)?,
            }),
            5903u32 => Ok(Self::SchedulerTargetFmaxMhzINTEL {
                target_fmax: LiteralInteger::parse_one(stream)?,
            }),
            6023u32 => Ok(Self::MaximallyReconvergesKHR),
            6028u32 => Ok(Self::FPFastMathDefault {
                target_type: IdRef::parse_one(stream)?,
                fast_math_mode: IdRef::parse_one(stream)?,
            }),
            6154u32 => Ok(Self::StreamingInterfaceINTEL {
                stall_free_return: LiteralInteger::parse_one(stream)?,
            }),
            6160u32 => Ok(Self::RegisterMapInterfaceINTEL {
                wait_for_done_write: LiteralInteger::parse_one(stream)?,
            }),
            6417u32 => Ok(Self::NamedBarrierCountINTEL {
                barrier_count: LiteralInteger::parse_one(stream)?,
            }),
            6461u32 => Ok(Self::MaximumRegistersINTEL {
                number_of_registers: LiteralInteger::parse_one(stream)?,
            }),
            6462u32 => Ok(Self::MaximumRegistersIdINTEL {
                number_of_registers: IdRef::parse_one(stream)?,
            }),
            6463u32 => Ok(Self::NamedMaximumRegistersINTEL {
                named_maximum_number_of_registers: NamedMaximumNumberOfRegisters::parse_one(
                    stream,
                )?,
            }),
            x => Err(ParseError::UnknownVariant {
                kind: stringify!(ExecutionMode),
                value: x,
            }),
        }
    }
}
impl Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invocations {
                number_of_invocations,
            } => {
                write!(f, "{}", stringify!(Invocations))?;
                write!(f, " {}", number_of_invocations)?;
                Ok(())
            }
            Self::SpacingEqual => write!(f, "{}", stringify!(SpacingEqual)),
            Self::SpacingFractionalEven => write!(f, "{}", stringify!(SpacingFractionalEven)),
            Self::SpacingFractionalOdd => write!(f, "{}", stringify!(SpacingFractionalOdd)),
            Self::VertexOrderCw => write!(f, "{}", stringify!(VertexOrderCw)),
            Self::VertexOrderCcw => write!(f, "{}", stringify!(VertexOrderCcw)),
            Self::PixelCenterInteger => write!(f, "{}", stringify!(PixelCenterInteger)),
            Self::OriginUpperLeft => write!(f, "{}", stringify!(OriginUpperLeft)),
            Self::OriginLowerLeft => write!(f, "{}", stringify!(OriginLowerLeft)),
            Self::EarlyFragmentTests => write!(f, "{}", stringify!(EarlyFragmentTests)),
            Self::PointMode => write!(f, "{}", stringify!(PointMode)),
            Self::Xfb => write!(f, "{}", stringify!(Xfb)),
            Self::DepthReplacing => write!(f, "{}", stringify!(DepthReplacing)),
            Self::DepthGreater => write!(f, "{}", stringify!(DepthGreater)),
            Self::DepthLess => write!(f, "{}", stringify!(DepthLess)),
            Self::DepthUnchanged => write!(f, "{}", stringify!(DepthUnchanged)),
            Self::LocalSize {
                x_size,
                y_size,
                z_size,
            } => {
                write!(f, "{}", stringify!(LocalSize))?;
                write!(f, " {}", x_size)?;
                write!(f, " {}", y_size)?;
                write!(f, " {}", z_size)?;
                Ok(())
            }
            Self::LocalSizeHint {
                x_size,
                y_size,
                z_size,
            } => {
                write!(f, "{}", stringify!(LocalSizeHint))?;
                write!(f, " {}", x_size)?;
                write!(f, " {}", y_size)?;
                write!(f, " {}", z_size)?;
                Ok(())
            }
            Self::InputPoints => write!(f, "{}", stringify!(InputPoints)),
            Self::InputLines => write!(f, "{}", stringify!(InputLines)),
            Self::InputLinesAdjacency => write!(f, "{}", stringify!(InputLinesAdjacency)),
            Self::Triangles => write!(f, "{}", stringify!(Triangles)),
            Self::InputTrianglesAdjacency => write!(f, "{}", stringify!(InputTrianglesAdjacency)),
            Self::Quads => write!(f, "{}", stringify!(Quads)),
            Self::Isolines => write!(f, "{}", stringify!(Isolines)),
            Self::OutputVertices { vertex_count } => {
                write!(f, "{}", stringify!(OutputVertices))?;
                write!(f, " {}", vertex_count)?;
                Ok(())
            }
            Self::OutputPoints => write!(f, "{}", stringify!(OutputPoints)),
            Self::OutputLineStrip => write!(f, "{}", stringify!(OutputLineStrip)),
            Self::OutputTriangleStrip => write!(f, "{}", stringify!(OutputTriangleStrip)),
            Self::VecTypeHint { vector_type } => {
                write!(f, "{}", stringify!(VecTypeHint))?;
                write!(f, " {}", vector_type)?;
                Ok(())
            }
            Self::ContractionOff => write!(f, "{}", stringify!(ContractionOff)),
            Self::Initializer => write!(f, "{}", stringify!(Initializer)),
            Self::Finalizer => write!(f, "{}", stringify!(Finalizer)),
            Self::SubgroupSize { subgroup_size } => {
                write!(f, "{}", stringify!(SubgroupSize))?;
                write!(f, " {}", subgroup_size)?;
                Ok(())
            }
            Self::SubgroupsPerWorkgroup {
                subgroups_per_workgroup,
            } => {
                write!(f, "{}", stringify!(SubgroupsPerWorkgroup))?;
                write!(f, " {}", subgroups_per_workgroup)?;
                Ok(())
            }
            Self::SubgroupsPerWorkgroupId {
                subgroups_per_workgroup,
            } => {
                write!(f, "{}", stringify!(SubgroupsPerWorkgroupId))?;
                write!(f, " {}", subgroups_per_workgroup)?;
                Ok(())
            }
            Self::LocalSizeId {
                x_size,
                y_size,
                z_size,
            } => {
                write!(f, "{}", stringify!(LocalSizeId))?;
                write!(f, " {}", x_size)?;
                write!(f, " {}", y_size)?;
                write!(f, " {}", z_size)?;
                Ok(())
            }
            Self::LocalSizeHintId {
                x_size_hint,
                y_size_hint,
                z_size_hint,
            } => {
                write!(f, "{}", stringify!(LocalSizeHintId))?;
                write!(f, " {}", x_size_hint)?;
                write!(f, " {}", y_size_hint)?;
                write!(f, " {}", z_size_hint)?;
                Ok(())
            }
            Self::NonCoherentColorAttachmentReadEXT => {
                write!(f, "{}", stringify!(NonCoherentColorAttachmentReadEXT))
            }
            Self::NonCoherentDepthAttachmentReadEXT => {
                write!(f, "{}", stringify!(NonCoherentDepthAttachmentReadEXT))
            }
            Self::NonCoherentStencilAttachmentReadEXT => {
                write!(f, "{}", stringify!(NonCoherentStencilAttachmentReadEXT))
            }
            Self::SubgroupUniformControlFlowKHR => {
                write!(f, "{}", stringify!(SubgroupUniformControlFlowKHR))
            }
            Self::PostDepthCoverage => write!(f, "{}", stringify!(PostDepthCoverage)),
            Self::DenormPreserve { target_width } => {
                write!(f, "{}", stringify!(DenormPreserve))?;
                write!(f, " {}", target_width)?;
                Ok(())
            }
            Self::DenormFlushToZero { target_width } => {
                write!(f, "{}", stringify!(DenormFlushToZero))?;
                write!(f, " {}", target_width)?;
                Ok(())
            }
            Self::SignedZeroInfNanPreserve { target_width } => {
                write!(f, "{}", stringify!(SignedZeroInfNanPreserve))?;
                write!(f, " {}", target_width)?;
                Ok(())
            }
            Self::RoundingModeRTE { target_width } => {
                write!(f, "{}", stringify!(RoundingModeRTE))?;
                write!(f, " {}", target_width)?;
                Ok(())
            }
            Self::RoundingModeRTZ { target_width } => {
                write!(f, "{}", stringify!(RoundingModeRTZ))?;
                write!(f, " {}", target_width)?;
                Ok(())
            }
            Self::NonCoherentTileAttachmentReadQCOM => {
                write!(f, "{}", stringify!(NonCoherentTileAttachmentReadQCOM))
            }
            Self::TileShadingRateQCOM {
                x_rate,
                y_rate,
                z_rate,
            } => {
                write!(f, "{}", stringify!(TileShadingRateQCOM))?;
                write!(f, " {}", x_rate)?;
                write!(f, " {}", y_rate)?;
                write!(f, " {}", z_rate)?;
                Ok(())
            }
            Self::EarlyAndLateFragmentTestsAMD => {
                write!(f, "{}", stringify!(EarlyAndLateFragmentTestsAMD))
            }
            Self::StencilRefReplacingEXT => write!(f, "{}", stringify!(StencilRefReplacingEXT)),
            Self::CoalescingAMDX => write!(f, "{}", stringify!(CoalescingAMDX)),
            Self::IsApiEntryAMDX { is_entry } => {
                write!(f, "{}", stringify!(IsApiEntryAMDX))?;
                write!(f, " {}", is_entry)?;
                Ok(())
            }
            Self::MaxNodeRecursionAMDX {
                number_of_recursions,
            } => {
                write!(f, "{}", stringify!(MaxNodeRecursionAMDX))?;
                write!(f, " {}", number_of_recursions)?;
                Ok(())
            }
            Self::StaticNumWorkgroupsAMDX {
                x_size,
                y_size,
                z_size,
            } => {
                write!(f, "{}", stringify!(StaticNumWorkgroupsAMDX))?;
                write!(f, " {}", x_size)?;
                write!(f, " {}", y_size)?;
                write!(f, " {}", z_size)?;
                Ok(())
            }
            Self::ShaderIndexAMDX { shader_index } => {
                write!(f, "{}", stringify!(ShaderIndexAMDX))?;
                write!(f, " {}", shader_index)?;
                Ok(())
            }
            Self::MaxNumWorkgroupsAMDX {
                x_size,
                y_size,
                z_size,
            } => {
                write!(f, "{}", stringify!(MaxNumWorkgroupsAMDX))?;
                write!(f, " {}", x_size)?;
                write!(f, " {}", y_size)?;
                write!(f, " {}", z_size)?;
                Ok(())
            }
            Self::StencilRefUnchangedFrontAMD => {
                write!(f, "{}", stringify!(StencilRefUnchangedFrontAMD))
            }
            Self::StencilRefGreaterFrontAMD => {
                write!(f, "{}", stringify!(StencilRefGreaterFrontAMD))
            }
            Self::StencilRefLessFrontAMD => write!(f, "{}", stringify!(StencilRefLessFrontAMD)),
            Self::StencilRefUnchangedBackAMD => {
                write!(f, "{}", stringify!(StencilRefUnchangedBackAMD))
            }
            Self::StencilRefGreaterBackAMD => write!(f, "{}", stringify!(StencilRefGreaterBackAMD)),
            Self::StencilRefLessBackAMD => write!(f, "{}", stringify!(StencilRefLessBackAMD)),
            Self::QuadDerivativesKHR => write!(f, "{}", stringify!(QuadDerivativesKHR)),
            Self::RequireFullQuadsKHR => write!(f, "{}", stringify!(RequireFullQuadsKHR)),
            Self::SharesInputWithAMDX {
                node_name,
                shader_index,
            } => {
                write!(f, "{}", stringify!(SharesInputWithAMDX))?;
                write!(f, " {}", node_name)?;
                write!(f, " {}", shader_index)?;
                Ok(())
            }
            Self::ArithmeticPoisonKHR => write!(f, "{}", stringify!(ArithmeticPoisonKHR)),
            Self::OutputLinesEXT => write!(f, "{}", stringify!(OutputLinesEXT)),
            Self::OutputPrimitivesEXT { primitive_count } => {
                write!(f, "{}", stringify!(OutputPrimitivesEXT))?;
                write!(f, " {}", primitive_count)?;
                Ok(())
            }
            Self::DerivativeGroupQuadsKHR => write!(f, "{}", stringify!(DerivativeGroupQuadsKHR)),
            Self::DerivativeGroupLinearKHR => write!(f, "{}", stringify!(DerivativeGroupLinearKHR)),
            Self::OutputTrianglesEXT => write!(f, "{}", stringify!(OutputTrianglesEXT)),
            Self::PixelInterlockOrderedEXT => write!(f, "{}", stringify!(PixelInterlockOrderedEXT)),
            Self::PixelInterlockUnorderedEXT => {
                write!(f, "{}", stringify!(PixelInterlockUnorderedEXT))
            }
            Self::SampleInterlockOrderedEXT => {
                write!(f, "{}", stringify!(SampleInterlockOrderedEXT))
            }
            Self::SampleInterlockUnorderedEXT => {
                write!(f, "{}", stringify!(SampleInterlockUnorderedEXT))
            }
            Self::ShadingRateInterlockOrderedEXT => {
                write!(f, "{}", stringify!(ShadingRateInterlockOrderedEXT))
            }
            Self::ShadingRateInterlockUnorderedEXT => {
                write!(f, "{}", stringify!(ShadingRateInterlockUnorderedEXT))
            }
            Self::Shader64BitIndexingEXT => write!(f, "{}", stringify!(Shader64BitIndexingEXT)),
            Self::SharedLocalMemorySizeINTEL { size } => {
                write!(f, "{}", stringify!(SharedLocalMemorySizeINTEL))?;
                write!(f, " {}", size)?;
                Ok(())
            }
            Self::RoundingModeRTPINTEL { target_width } => {
                write!(f, "{}", stringify!(RoundingModeRTPINTEL))?;
                write!(f, " {}", target_width)?;
                Ok(())
            }
            Self::RoundingModeRTNINTEL { target_width } => {
                write!(f, "{}", stringify!(RoundingModeRTNINTEL))?;
                write!(f, " {}", target_width)?;
                Ok(())
            }
            Self::FloatingPointModeALTINTEL { target_width } => {
                write!(f, "{}", stringify!(FloatingPointModeALTINTEL))?;
                write!(f, " {}", target_width)?;
                Ok(())
            }
            Self::FloatingPointModeIEEEINTEL { target_width } => {
                write!(f, "{}", stringify!(FloatingPointModeIEEEINTEL))?;
                write!(f, " {}", target_width)?;
                Ok(())
            }
            Self::MaxWorkgroupSizeINTEL {
                max_x_size,
                max_y_size,
                max_z_size,
            } => {
                write!(f, "{}", stringify!(MaxWorkgroupSizeINTEL))?;
                write!(f, " {}", max_x_size)?;
                write!(f, " {}", max_y_size)?;
                write!(f, " {}", max_z_size)?;
                Ok(())
            }
            Self::MaxWorkDimINTEL { max_dimensions } => {
                write!(f, "{}", stringify!(MaxWorkDimINTEL))?;
                write!(f, " {}", max_dimensions)?;
                Ok(())
            }
            Self::NoGlobalOffsetINTEL => write!(f, "{}", stringify!(NoGlobalOffsetINTEL)),
            Self::NumSIMDWorkitemsINTEL { vector_width } => {
                write!(f, "{}", stringify!(NumSIMDWorkitemsINTEL))?;
                write!(f, " {}", vector_width)?;
                Ok(())
            }
            Self::SchedulerTargetFmaxMhzINTEL { target_fmax } => {
                write!(f, "{}", stringify!(SchedulerTargetFmaxMhzINTEL))?;
                write!(f, " {}", target_fmax)?;
                Ok(())
            }
            Self::MaximallyReconvergesKHR => write!(f, "{}", stringify!(MaximallyReconvergesKHR)),
            Self::FPFastMathDefault {
                target_type,
                fast_math_mode,
            } => {
                write!(f, "{}", stringify!(FPFastMathDefault))?;
                write!(f, " {}", target_type)?;
                write!(f, " {}", fast_math_mode)?;
                Ok(())
            }
            Self::StreamingInterfaceINTEL { stall_free_return } => {
                write!(f, "{}", stringify!(StreamingInterfaceINTEL))?;
                write!(f, " {}", stall_free_return)?;
                Ok(())
            }
            Self::RegisterMapInterfaceINTEL {
                wait_for_done_write,
            } => {
                write!(f, "{}", stringify!(RegisterMapInterfaceINTEL))?;
                write!(f, " {}", wait_for_done_write)?;
                Ok(())
            }
            Self::NamedBarrierCountINTEL { barrier_count } => {
                write!(f, "{}", stringify!(NamedBarrierCountINTEL))?;
                write!(f, " {}", barrier_count)?;
                Ok(())
            }
            Self::MaximumRegistersINTEL {
                number_of_registers,
            } => {
                write!(f, "{}", stringify!(MaximumRegistersINTEL))?;
                write!(f, " {}", number_of_registers)?;
                Ok(())
            }
            Self::MaximumRegistersIdINTEL {
                number_of_registers,
            } => {
                write!(f, "{}", stringify!(MaximumRegistersIdINTEL))?;
                write!(f, " {}", number_of_registers)?;
                Ok(())
            }
            Self::NamedMaximumRegistersINTEL {
                named_maximum_number_of_registers,
            } => {
                write!(f, "{}", stringify!(NamedMaximumRegistersINTEL))?;
                write!(f, " {}", named_maximum_number_of_registers)?;
                Ok(())
            }
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StorageClass(pub(crate) u32);
impl StorageClass {
    pub const UNIFORM_CONSTANT: Self = Self(0u32);
    pub const INPUT: Self = Self(1u32);
    pub const UNIFORM: Self = Self(2u32);
    pub const OUTPUT: Self = Self(3u32);
    pub const WORKGROUP: Self = Self(4u32);
    pub const CROSS_WORKGROUP: Self = Self(5u32);
    pub const PRIVATE: Self = Self(6u32);
    pub const FUNCTION: Self = Self(7u32);
    pub const GENERIC: Self = Self(8u32);
    pub const PUSH_CONSTANT: Self = Self(9u32);
    pub const ATOMIC_COUNTER: Self = Self(10u32);
    pub const IMAGE: Self = Self(11u32);
    pub const STORAGE_BUFFER: Self = Self(12u32);
    pub const TILE_IMAGE_EXT: Self = Self(4172u32);
    pub const TILE_ATTACHMENT_QCOM: Self = Self(4491u32);
    pub const NODE_PAYLOAD_AMDX: Self = Self(5068u32);
    pub const CALLABLE_DATA_KHR: Self = Self(5328u32);
    pub const INCOMING_CALLABLE_DATA_KHR: Self = Self(5329u32);
    pub const RAY_PAYLOAD_KHR: Self = Self(5338u32);
    pub const HIT_ATTRIBUTE_KHR: Self = Self(5339u32);
    pub const INCOMING_RAY_PAYLOAD_KHR: Self = Self(5342u32);
    pub const SHADER_RECORD_BUFFER_KHR: Self = Self(5343u32);
    pub const PHYSICAL_STORAGE_BUFFER: Self = Self(5349u32);
    pub const HIT_OBJECT_ATTRIBUTE_NV: Self = Self(5385u32);
    pub const TASK_PAYLOAD_WORKGROUP_EXT: Self = Self(5402u32);
    pub const HIT_OBJECT_ATTRIBUTE_EXT: Self = Self(5411u32);
    pub const CODE_SECTION_INTEL: Self = Self(5605u32);
    pub const DEVICE_ONLY_ALTERA: Self = Self(5936u32);
    pub const HOST_ONLY_ALTERA: Self = Self(5937u32);
}
impl Word for StorageClass {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for StorageClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UNIFORM_CONSTANT => write!(f, "UniformConstant"),
            Self::INPUT => write!(f, "Input"),
            Self::UNIFORM => write!(f, "Uniform"),
            Self::OUTPUT => write!(f, "Output"),
            Self::WORKGROUP => write!(f, "Workgroup"),
            Self::CROSS_WORKGROUP => write!(f, "CrossWorkgroup"),
            Self::PRIVATE => write!(f, "Private"),
            Self::FUNCTION => write!(f, "Function"),
            Self::GENERIC => write!(f, "Generic"),
            Self::PUSH_CONSTANT => write!(f, "PushConstant"),
            Self::ATOMIC_COUNTER => write!(f, "AtomicCounter"),
            Self::IMAGE => write!(f, "Image"),
            Self::STORAGE_BUFFER => write!(f, "StorageBuffer"),
            Self::TILE_IMAGE_EXT => write!(f, "TileImageEXT"),
            Self::TILE_ATTACHMENT_QCOM => write!(f, "TileAttachmentQCOM"),
            Self::NODE_PAYLOAD_AMDX => write!(f, "NodePayloadAMDX"),
            Self::CALLABLE_DATA_KHR => write!(f, "CallableDataKHR"),
            Self::INCOMING_CALLABLE_DATA_KHR => write!(f, "IncomingCallableDataKHR"),
            Self::RAY_PAYLOAD_KHR => write!(f, "RayPayloadKHR"),
            Self::HIT_ATTRIBUTE_KHR => write!(f, "HitAttributeKHR"),
            Self::INCOMING_RAY_PAYLOAD_KHR => write!(f, "IncomingRayPayloadKHR"),
            Self::SHADER_RECORD_BUFFER_KHR => write!(f, "ShaderRecordBufferKHR"),
            Self::PHYSICAL_STORAGE_BUFFER => write!(f, "PhysicalStorageBuffer"),
            Self::HIT_OBJECT_ATTRIBUTE_NV => write!(f, "HitObjectAttributeNV"),
            Self::TASK_PAYLOAD_WORKGROUP_EXT => write!(f, "TaskPayloadWorkgroupEXT"),
            Self::HIT_OBJECT_ATTRIBUTE_EXT => write!(f, "HitObjectAttributeEXT"),
            Self::CODE_SECTION_INTEL => write!(f, "CodeSectionINTEL"),
            Self::DEVICE_ONLY_ALTERA => write!(f, "DeviceOnlyALTERA"),
            Self::HOST_ONLY_ALTERA => write!(f, "HostOnlyALTERA"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Dim(pub(crate) u32);
impl Dim {
    pub const TYPE_1D: Self = Self(0u32);
    pub const TYPE_2D: Self = Self(1u32);
    pub const TYPE_3D: Self = Self(2u32);
    pub const CUBE: Self = Self(3u32);
    pub const RECT: Self = Self(4u32);
    pub const BUFFER: Self = Self(5u32);
    pub const SUBPASS_DATA: Self = Self(6u32);
    pub const TILE_IMAGE_DATA_EXT: Self = Self(4173u32);
}
impl Word for Dim {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for Dim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::TYPE_1D => write!(f, "1D"),
            Self::TYPE_2D => write!(f, "2D"),
            Self::TYPE_3D => write!(f, "3D"),
            Self::CUBE => write!(f, "Cube"),
            Self::RECT => write!(f, "Rect"),
            Self::BUFFER => write!(f, "Buffer"),
            Self::SUBPASS_DATA => write!(f, "SubpassData"),
            Self::TILE_IMAGE_DATA_EXT => write!(f, "TileImageDataEXT"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SamplerAddressingMode(pub(crate) u32);
impl SamplerAddressingMode {
    pub const NONE: Self = Self(0u32);
    pub const CLAMP_TO_EDGE: Self = Self(1u32);
    pub const CLAMP: Self = Self(2u32);
    pub const REPEAT: Self = Self(3u32);
    pub const REPEAT_MIRRORED: Self = Self(4u32);
}
impl Word for SamplerAddressingMode {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for SamplerAddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NONE => write!(f, "None"),
            Self::CLAMP_TO_EDGE => write!(f, "ClampToEdge"),
            Self::CLAMP => write!(f, "Clamp"),
            Self::REPEAT => write!(f, "Repeat"),
            Self::REPEAT_MIRRORED => write!(f, "RepeatMirrored"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SamplerFilterMode(pub(crate) u32);
impl SamplerFilterMode {
    pub const NEAREST: Self = Self(0u32);
    pub const LINEAR: Self = Self(1u32);
}
impl Word for SamplerFilterMode {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for SamplerFilterMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NEAREST => write!(f, "Nearest"),
            Self::LINEAR => write!(f, "Linear"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageFormat(pub(crate) u32);
impl ImageFormat {
    pub const UNKNOWN: Self = Self(0u32);
    pub const RGBA32F: Self = Self(1u32);
    pub const RGBA16F: Self = Self(2u32);
    pub const R32F: Self = Self(3u32);
    pub const RGBA8: Self = Self(4u32);
    pub const RGBA8_SNORM: Self = Self(5u32);
    pub const RG32F: Self = Self(6u32);
    pub const RG16F: Self = Self(7u32);
    pub const R11F_G11F_B10F: Self = Self(8u32);
    pub const R16F: Self = Self(9u32);
    pub const RGBA16: Self = Self(10u32);
    pub const RGB10_A2: Self = Self(11u32);
    pub const RG16: Self = Self(12u32);
    pub const RG8: Self = Self(13u32);
    pub const R16: Self = Self(14u32);
    pub const R8: Self = Self(15u32);
    pub const RGBA16_SNORM: Self = Self(16u32);
    pub const RG16_SNORM: Self = Self(17u32);
    pub const RG8_SNORM: Self = Self(18u32);
    pub const R16_SNORM: Self = Self(19u32);
    pub const R8_SNORM: Self = Self(20u32);
    pub const RGBA32I: Self = Self(21u32);
    pub const RGBA16I: Self = Self(22u32);
    pub const RGBA8I: Self = Self(23u32);
    pub const R32I: Self = Self(24u32);
    pub const RG32I: Self = Self(25u32);
    pub const RG16I: Self = Self(26u32);
    pub const RG8I: Self = Self(27u32);
    pub const R16I: Self = Self(28u32);
    pub const R8I: Self = Self(29u32);
    pub const RGBA32UI: Self = Self(30u32);
    pub const RGBA16UI: Self = Self(31u32);
    pub const RGBA8UI: Self = Self(32u32);
    pub const R32UI: Self = Self(33u32);
    pub const RGB10A2UI: Self = Self(34u32);
    pub const RG32UI: Self = Self(35u32);
    pub const RG16UI: Self = Self(36u32);
    pub const RG8UI: Self = Self(37u32);
    pub const R16UI: Self = Self(38u32);
    pub const R8UI: Self = Self(39u32);
    pub const R64UI: Self = Self(40u32);
    pub const R64I: Self = Self(41u32);
}
impl Word for ImageFormat {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UNKNOWN => write!(f, "Unknown"),
            Self::RGBA32F => write!(f, "Rgba32f"),
            Self::RGBA16F => write!(f, "Rgba16f"),
            Self::R32F => write!(f, "R32f"),
            Self::RGBA8 => write!(f, "Rgba8"),
            Self::RGBA8_SNORM => write!(f, "Rgba8Snorm"),
            Self::RG32F => write!(f, "Rg32f"),
            Self::RG16F => write!(f, "Rg16f"),
            Self::R11F_G11F_B10F => write!(f, "R11fG11fB10f"),
            Self::R16F => write!(f, "R16f"),
            Self::RGBA16 => write!(f, "Rgba16"),
            Self::RGB10_A2 => write!(f, "Rgb10A2"),
            Self::RG16 => write!(f, "Rg16"),
            Self::RG8 => write!(f, "Rg8"),
            Self::R16 => write!(f, "R16"),
            Self::R8 => write!(f, "R8"),
            Self::RGBA16_SNORM => write!(f, "Rgba16Snorm"),
            Self::RG16_SNORM => write!(f, "Rg16Snorm"),
            Self::RG8_SNORM => write!(f, "Rg8Snorm"),
            Self::R16_SNORM => write!(f, "R16Snorm"),
            Self::R8_SNORM => write!(f, "R8Snorm"),
            Self::RGBA32I => write!(f, "Rgba32i"),
            Self::RGBA16I => write!(f, "Rgba16i"),
            Self::RGBA8I => write!(f, "Rgba8i"),
            Self::R32I => write!(f, "R32i"),
            Self::RG32I => write!(f, "Rg32i"),
            Self::RG16I => write!(f, "Rg16i"),
            Self::RG8I => write!(f, "Rg8i"),
            Self::R16I => write!(f, "R16i"),
            Self::R8I => write!(f, "R8i"),
            Self::RGBA32UI => write!(f, "Rgba32ui"),
            Self::RGBA16UI => write!(f, "Rgba16ui"),
            Self::RGBA8UI => write!(f, "Rgba8ui"),
            Self::R32UI => write!(f, "R32ui"),
            Self::RGB10A2UI => write!(f, "Rgb10a2ui"),
            Self::RG32UI => write!(f, "Rg32ui"),
            Self::RG16UI => write!(f, "Rg16ui"),
            Self::RG8UI => write!(f, "Rg8ui"),
            Self::R16UI => write!(f, "R16ui"),
            Self::R8UI => write!(f, "R8ui"),
            Self::R64UI => write!(f, "R64ui"),
            Self::R64I => write!(f, "R64i"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageChannelOrder(pub(crate) u32);
impl ImageChannelOrder {
    pub const R: Self = Self(0u32);
    pub const A: Self = Self(1u32);
    pub const RG: Self = Self(2u32);
    pub const RA: Self = Self(3u32);
    pub const RGB: Self = Self(4u32);
    pub const RGBA: Self = Self(5u32);
    pub const BGRA: Self = Self(6u32);
    pub const ARGB: Self = Self(7u32);
    pub const INTENSITY: Self = Self(8u32);
    pub const LUMINANCE: Self = Self(9u32);
    pub const RX: Self = Self(10u32);
    pub const RGX: Self = Self(11u32);
    pub const RGBX: Self = Self(12u32);
    pub const DEPTH: Self = Self(13u32);
    pub const DEPTH_STENCIL: Self = Self(14u32);
    pub const SRGB: Self = Self(15u32);
    pub const SRGBX: Self = Self(16u32);
    pub const SRGBA: Self = Self(17u32);
    pub const SBGRA: Self = Self(18u32);
    pub const ABGR: Self = Self(19u32);
}
impl Word for ImageChannelOrder {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for ImageChannelOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::R => write!(f, "R"),
            Self::A => write!(f, "A"),
            Self::RG => write!(f, "RG"),
            Self::RA => write!(f, "RA"),
            Self::RGB => write!(f, "RGB"),
            Self::RGBA => write!(f, "RGBA"),
            Self::BGRA => write!(f, "BGRA"),
            Self::ARGB => write!(f, "ARGB"),
            Self::INTENSITY => write!(f, "Intensity"),
            Self::LUMINANCE => write!(f, "Luminance"),
            Self::RX => write!(f, "Rx"),
            Self::RGX => write!(f, "RGx"),
            Self::RGBX => write!(f, "RGBx"),
            Self::DEPTH => write!(f, "Depth"),
            Self::DEPTH_STENCIL => write!(f, "DepthStencil"),
            Self::SRGB => write!(f, "sRGB"),
            Self::SRGBX => write!(f, "sRGBx"),
            Self::SRGBA => write!(f, "sRGBA"),
            Self::SBGRA => write!(f, "sBGRA"),
            Self::ABGR => write!(f, "ABGR"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageChannelDataType(pub(crate) u32);
impl ImageChannelDataType {
    pub const SNORM_INT8: Self = Self(0u32);
    pub const SNORM_INT16: Self = Self(1u32);
    pub const UNORM_INT8: Self = Self(2u32);
    pub const UNORM_INT16: Self = Self(3u32);
    pub const UNORM_SHORT565: Self = Self(4u32);
    pub const UNORM_SHORT555: Self = Self(5u32);
    pub const UNORM_INT101010: Self = Self(6u32);
    pub const SIGNED_INT8: Self = Self(7u32);
    pub const SIGNED_INT16: Self = Self(8u32);
    pub const SIGNED_INT32: Self = Self(9u32);
    pub const UNSIGNED_INT8: Self = Self(10u32);
    pub const UNSIGNED_INT16: Self = Self(11u32);
    pub const UNSIGNED_INT32: Self = Self(12u32);
    pub const HALF_FLOAT: Self = Self(13u32);
    pub const FLOAT: Self = Self(14u32);
    pub const UNORM_INT24: Self = Self(15u32);
    pub const UNORM_INT101010_2: Self = Self(16u32);
    pub const UNORM_INT10_X6_EXT: Self = Self(17u32);
    pub const UNSIGNED_INT_RAW10_EXT: Self = Self(19u32);
    pub const UNSIGNED_INT_RAW12_EXT: Self = Self(20u32);
    pub const UNORM_INT2_101010_EXT: Self = Self(21u32);
    pub const UNSIGNED_INT10_X6_EXT: Self = Self(22u32);
    pub const UNSIGNED_INT12_X4_EXT: Self = Self(23u32);
    pub const UNSIGNED_INT14_X2_EXT: Self = Self(24u32);
    pub const UNORM_INT12_X4_EXT: Self = Self(25u32);
    pub const UNORM_INT14_X2_EXT: Self = Self(26u32);
}
impl Word for ImageChannelDataType {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for ImageChannelDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::SNORM_INT8 => write!(f, "SnormInt8"),
            Self::SNORM_INT16 => write!(f, "SnormInt16"),
            Self::UNORM_INT8 => write!(f, "UnormInt8"),
            Self::UNORM_INT16 => write!(f, "UnormInt16"),
            Self::UNORM_SHORT565 => write!(f, "UnormShort565"),
            Self::UNORM_SHORT555 => write!(f, "UnormShort555"),
            Self::UNORM_INT101010 => write!(f, "UnormInt101010"),
            Self::SIGNED_INT8 => write!(f, "SignedInt8"),
            Self::SIGNED_INT16 => write!(f, "SignedInt16"),
            Self::SIGNED_INT32 => write!(f, "SignedInt32"),
            Self::UNSIGNED_INT8 => write!(f, "UnsignedInt8"),
            Self::UNSIGNED_INT16 => write!(f, "UnsignedInt16"),
            Self::UNSIGNED_INT32 => write!(f, "UnsignedInt32"),
            Self::HALF_FLOAT => write!(f, "HalfFloat"),
            Self::FLOAT => write!(f, "Float"),
            Self::UNORM_INT24 => write!(f, "UnormInt24"),
            Self::UNORM_INT101010_2 => write!(f, "UnormInt101010_2"),
            Self::UNORM_INT10_X6_EXT => write!(f, "UnormInt10X6EXT"),
            Self::UNSIGNED_INT_RAW10_EXT => write!(f, "UnsignedIntRaw10EXT"),
            Self::UNSIGNED_INT_RAW12_EXT => write!(f, "UnsignedIntRaw12EXT"),
            Self::UNORM_INT2_101010_EXT => write!(f, "UnormInt2_101010EXT"),
            Self::UNSIGNED_INT10_X6_EXT => write!(f, "UnsignedInt10X6EXT"),
            Self::UNSIGNED_INT12_X4_EXT => write!(f, "UnsignedInt12X4EXT"),
            Self::UNSIGNED_INT14_X2_EXT => write!(f, "UnsignedInt14X2EXT"),
            Self::UNORM_INT12_X4_EXT => write!(f, "UnormInt12X4EXT"),
            Self::UNORM_INT14_X2_EXT => write!(f, "UnormInt14X2EXT"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FPRoundingMode(pub(crate) u32);
impl FPRoundingMode {
    pub const RTE: Self = Self(0u32);
    pub const RTZ: Self = Self(1u32);
    pub const RTP: Self = Self(2u32);
    pub const RTN: Self = Self(3u32);
}
impl Word for FPRoundingMode {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for FPRoundingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::RTE => write!(f, "RTE"),
            Self::RTZ => write!(f, "RTZ"),
            Self::RTP => write!(f, "RTP"),
            Self::RTN => write!(f, "RTN"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FPDenormMode(pub(crate) u32);
impl FPDenormMode {
    pub const PRESERVE: Self = Self(0u32);
    pub const FLUSH_TO_ZERO: Self = Self(1u32);
}
impl Word for FPDenormMode {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for FPDenormMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::PRESERVE => write!(f, "Preserve"),
            Self::FLUSH_TO_ZERO => write!(f, "FlushToZero"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct QuantizationModes(pub(crate) u32);
impl QuantizationModes {
    pub const TRN: Self = Self(0u32);
    pub const TRN_ZERO: Self = Self(1u32);
    pub const RND: Self = Self(2u32);
    pub const RND_ZERO: Self = Self(3u32);
    pub const RND_INF: Self = Self(4u32);
    pub const RND_MIN_INF: Self = Self(5u32);
    pub const RND_CONV: Self = Self(6u32);
    pub const RND_CONV_ODD: Self = Self(7u32);
}
impl Word for QuantizationModes {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for QuantizationModes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::TRN => write!(f, "TRN"),
            Self::TRN_ZERO => write!(f, "TRN_ZERO"),
            Self::RND => write!(f, "RND"),
            Self::RND_ZERO => write!(f, "RND_ZERO"),
            Self::RND_INF => write!(f, "RND_INF"),
            Self::RND_MIN_INF => write!(f, "RND_MIN_INF"),
            Self::RND_CONV => write!(f, "RND_CONV"),
            Self::RND_CONV_ODD => write!(f, "RND_CONV_ODD"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FPOperationMode(pub(crate) u32);
impl FPOperationMode {
    pub const IEEE: Self = Self(0u32);
    pub const ALT: Self = Self(1u32);
}
impl Word for FPOperationMode {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for FPOperationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::IEEE => write!(f, "IEEE"),
            Self::ALT => write!(f, "ALT"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OverflowModes(pub(crate) u32);
impl OverflowModes {
    pub const WRAP: Self = Self(0u32);
    pub const SAT: Self = Self(1u32);
    pub const SAT_ZERO: Self = Self(2u32);
    pub const SAT_SYM: Self = Self(3u32);
}
impl Word for OverflowModes {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for OverflowModes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::WRAP => write!(f, "WRAP"),
            Self::SAT => write!(f, "SAT"),
            Self::SAT_ZERO => write!(f, "SAT_ZERO"),
            Self::SAT_SYM => write!(f, "SAT_SYM"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LinkageType(pub(crate) u32);
impl LinkageType {
    pub const EXPORT: Self = Self(0u32);
    pub const IMPORT: Self = Self(1u32);
    pub const LINK_ONCE_ODR: Self = Self(2u32);
}
impl Word for LinkageType {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for LinkageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::EXPORT => write!(f, "Export"),
            Self::IMPORT => write!(f, "Import"),
            Self::LINK_ONCE_ODR => write!(f, "LinkOnceODR"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AccessQualifier(pub(crate) u32);
impl AccessQualifier {
    pub const READ_ONLY: Self = Self(0u32);
    pub const WRITE_ONLY: Self = Self(1u32);
    pub const READ_WRITE: Self = Self(2u32);
}
impl Word for AccessQualifier {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for AccessQualifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::READ_ONLY => write!(f, "ReadOnly"),
            Self::WRITE_ONLY => write!(f, "WriteOnly"),
            Self::READ_WRITE => write!(f, "ReadWrite"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HostAccessQualifier(pub(crate) u32);
impl HostAccessQualifier {
    pub const NONE_INTEL: Self = Self(0u32);
    pub const READ_INTEL: Self = Self(1u32);
    pub const WRITE_INTEL: Self = Self(2u32);
    pub const READ_WRITE_INTEL: Self = Self(3u32);
}
impl Word for HostAccessQualifier {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for HostAccessQualifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NONE_INTEL => write!(f, "NoneINTEL"),
            Self::READ_INTEL => write!(f, "ReadINTEL"),
            Self::WRITE_INTEL => write!(f, "WriteINTEL"),
            Self::READ_WRITE_INTEL => write!(f, "ReadWriteINTEL"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FunctionParameterAttribute(pub(crate) u32);
impl FunctionParameterAttribute {
    pub const ZEXT: Self = Self(0u32);
    pub const SEXT: Self = Self(1u32);
    pub const BY_VAL: Self = Self(2u32);
    pub const SRET: Self = Self(3u32);
    pub const NO_ALIAS: Self = Self(4u32);
    pub const NO_CAPTURE: Self = Self(5u32);
    pub const NO_WRITE: Self = Self(6u32);
    pub const NO_READ_WRITE: Self = Self(7u32);
    pub const RUNTIME_ALIGNED_ALTERA: Self = Self(5940u32);
}
impl Word for FunctionParameterAttribute {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for FunctionParameterAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ZEXT => write!(f, "Zext"),
            Self::SEXT => write!(f, "Sext"),
            Self::BY_VAL => write!(f, "ByVal"),
            Self::SRET => write!(f, "Sret"),
            Self::NO_ALIAS => write!(f, "NoAlias"),
            Self::NO_CAPTURE => write!(f, "NoCapture"),
            Self::NO_WRITE => write!(f, "NoWrite"),
            Self::NO_READ_WRITE => write!(f, "NoReadWrite"),
            Self::RUNTIME_ALIGNED_ALTERA => write!(f, "RuntimeAlignedALTERA"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Decoration<'a> {
    RelaxedPrecision,
    SpecId {
        specialization_constant_id: LiteralInteger,
    },
    Block,
    BufferBlock,
    RowMajor,
    ColMajor,
    ArrayStride {
        array_stride: LiteralInteger,
    },
    MatrixStride {
        matrix_stride: LiteralInteger,
    },
    GLSLShared,
    GLSLPacked,
    CPacked,
    BuiltIn {
        built_in: BuiltIn,
    },
    NoPerspective,
    Flat,
    Patch,
    Centroid,
    Sample,
    Invariant,
    Restrict,
    Aliased,
    Volatile,
    Constant,
    Coherent,
    NonWritable,
    NonReadable,
    Uniform,
    UniformId {
        execution: IdScope,
    },
    SaturatedConversion,
    Stream {
        stream_number: LiteralInteger,
    },
    Location {
        location: LiteralInteger,
    },
    Component {
        component: LiteralInteger,
    },
    Index {
        index: LiteralInteger,
    },
    Binding {
        binding_point: LiteralInteger,
    },
    DescriptorSet {
        descriptor_set: LiteralInteger,
    },
    Offset {
        byte_offset: LiteralInteger,
    },
    XfbBuffer {
        xfb_buffer_number: LiteralInteger,
    },
    XfbStride {
        xfb_stride: LiteralInteger,
    },
    FuncParamAttr {
        function_parameter_attribute: FunctionParameterAttribute,
    },
    FPRoundingMode {
        floating_point_rounding_mode: FPRoundingMode,
    },
    FPFastMathMode {
        fast_math_mode: FPFastMathMode,
    },
    LinkageAttributes {
        name: LiteralString<'a>,
        linkage_type: LinkageType,
    },
    NoContraction,
    InputAttachmentIndex {
        attachment_index: LiteralInteger,
    },
    Alignment {
        alignment: LiteralInteger,
    },
    MaxByteOffset {
        max_byte_offset: LiteralInteger,
    },
    AlignmentId {
        alignment: IdRef,
    },
    MaxByteOffsetId {
        max_byte_offset: IdRef,
    },
    SaturatedToLargestFloat8NormalConversionEXT,
    NoSignedWrap,
    NoUnsignedWrap,
    WeightTextureQCOM,
    BlockMatchTextureQCOM,
    BlockMatchSamplerQCOM,
    ExplicitInterpAMD,
    NodeSharesPayloadLimitsWithAMDX {
        payload_type: IdRef,
    },
    NodeMaxPayloadsAMDX {
        max_number_of_payloads: IdRef,
    },
    TrackFinishWritingAMDX,
    PayloadNodeNameAMDX {
        node_name: IdRef,
    },
    PayloadNodeBaseIndexAMDX {
        base_index: IdRef,
    },
    PayloadNodeSparseArrayAMDX,
    PayloadNodeArraySizeAMDX {
        array_size: IdRef,
    },
    PayloadDispatchIndirectAMDX,
    ArrayStrideIdEXT {
        array_stride: IdRef,
    },
    OffsetIdEXT {
        byte_offset: IdRef,
    },
    UTFEncodedKHR,
    OverrideCoverageNV,
    PassthroughNV,
    ViewportRelativeNV,
    SecondaryViewportRelativeNV {
        offset: LiteralInteger,
    },
    PerPrimitiveEXT,
    PerViewNV,
    PerTaskNV,
    PerVertexKHR,
    NonUniform,
    RestrictPointer,
    AliasedPointer,
    MemberOffsetNV {
        member_offset: LiteralInteger,
    },
    HitObjectShaderRecordBufferNV,
    HitObjectShaderRecordBufferEXT,
    BankNV {
        bank: LiteralInteger,
    },
    BindlessSamplerNV,
    BindlessImageNV,
    BoundSamplerNV,
    BoundImageNV,
    SIMTCallINTEL {
        n: LiteralInteger,
    },
    ReferencedIndirectlyINTEL,
    ClobberINTEL {
        register: LiteralString<'a>,
    },
    SideEffectsINTEL,
    VectorComputeVariableINTEL,
    FuncParamIOKindINTEL {
        kind: LiteralInteger,
    },
    VectorComputeFunctionINTEL,
    StackCallINTEL,
    GlobalVariableOffsetINTEL {
        offset: LiteralInteger,
    },
    CounterBuffer {
        counter_buffer: IdRef,
    },
    UserSemantic {
        semantic: LiteralString<'a>,
    },
    UserTypeGOOGLE {
        user_type: LiteralString<'a>,
    },
    FunctionRoundingModeINTEL {
        target_width: LiteralInteger,
        fp_rounding_mode: FPRoundingMode,
    },
    FunctionDenormModeINTEL {
        target_width: LiteralInteger,
        fp_denorm_mode: FPDenormMode,
    },
    RegisterALTERA,
    MemoryALTERA {
        memory_type: LiteralString<'a>,
    },
    NumbanksALTERA {
        banks: LiteralInteger,
    },
    BankwidthALTERA {
        bank_width: LiteralInteger,
    },
    MaxPrivateCopiesALTERA {
        maximum_copies: LiteralInteger,
    },
    SinglepumpALTERA,
    DoublepumpALTERA,
    MaxReplicatesALTERA {
        maximum_replicates: LiteralInteger,
    },
    SimpleDualPortALTERA,
    MergeALTERA {
        merge_key: LiteralString<'a>,
        merge_type: LiteralString<'a>,
    },
    BankBitsALTERA {
        bank_bits: LiteralInteger,
    },
    ForcePow2DepthALTERA {
        force_key: LiteralInteger,
    },
    StridesizeALTERA {
        stride_size: LiteralInteger,
    },
    WordsizeALTERA {
        word_size: LiteralInteger,
    },
    TrueDualPortALTERA,
    BurstCoalesceALTERA,
    CacheSizeALTERA {
        cache_size_in_bytes: LiteralInteger,
    },
    DontStaticallyCoalesceALTERA,
    PrefetchALTERA {
        prefetcher_size_in_bytes: LiteralInteger,
    },
    StallEnableALTERA,
    FuseLoopsInFunctionALTERA,
    MathOpDSPModeALTERA {
        mode: LiteralInteger,
        propagate: LiteralInteger,
    },
    AliasScopeINTEL {
        aliasing_scopes_list: IdRef,
    },
    NoAliasINTEL {
        aliasing_scopes_list: IdRef,
    },
    InitiationIntervalALTERA {
        cycles: LiteralInteger,
    },
    MaxConcurrencyALTERA {
        invocations: LiteralInteger,
    },
    PipelineEnableALTERA {
        enable: LiteralInteger,
    },
    BufferLocationALTERA {
        buffer_location_id: LiteralInteger,
    },
    IOPipeStorageALTERA {
        io_pipe_id: LiteralInteger,
    },
    FunctionFloatingPointModeINTEL {
        target_width: LiteralInteger,
        fp_operation_mode: FPOperationMode,
    },
    SingleElementVectorINTEL,
    VectorComputeCallableFunctionINTEL,
    MediaBlockIOINTEL,
    StallFreeALTERA,
    FPMaxErrorDecorationINTEL {
        max_error: LiteralFloat,
    },
    LatencyControlLabelALTERA {
        latency_label: LiteralInteger,
    },
    LatencyControlConstraintALTERA {
        relative_to: LiteralInteger,
        control_type: LiteralInteger,
        relative_cycle: LiteralInteger,
    },
    ConduitKernelArgumentALTERA,
    RegisterMapKernelArgumentALTERA,
    MMHostInterfaceAddressWidthALTERA {
        address_width: LiteralInteger,
    },
    MMHostInterfaceDataWidthALTERA {
        data_width: LiteralInteger,
    },
    MMHostInterfaceLatencyALTERA {
        latency: LiteralInteger,
    },
    MMHostInterfaceReadWriteModeALTERA {
        read_write_mode: AccessQualifier,
    },
    MMHostInterfaceMaxBurstALTERA {
        max_burst_count: LiteralInteger,
    },
    MMHostInterfaceWaitRequestALTERA {
        waitrequest: LiteralInteger,
    },
    StableKernelArgumentALTERA,
    HostAccessINTEL {
        access: HostAccessQualifier,
        name: LiteralString<'a>,
    },
    InitModeALTERA {
        trigger: InitializationModeQualifier,
    },
    ImplementInRegisterMapALTERA {
        value: LiteralInteger,
    },
    ConditionalINTEL {
        condition: IdRef,
    },
    CacheControlLoadINTEL {
        cache_level: LiteralInteger,
        cache_control: LoadCacheControl,
    },
    CacheControlStoreINTEL {
        cache_level: LiteralInteger,
        cache_control: StoreCacheControl,
    },
}
impl<'a> Decoration<'a> {
    #[inline]
    pub fn parse_one(stream: &mut InstructionStream<'a>) -> ParseResult<Self> {
        let variant = stream.read()?;
        match variant {
            0u32 => Ok(Self::RelaxedPrecision),
            1u32 => Ok(Self::SpecId {
                specialization_constant_id: LiteralInteger::parse_one(stream)?,
            }),
            2u32 => Ok(Self::Block),
            3u32 => Ok(Self::BufferBlock),
            4u32 => Ok(Self::RowMajor),
            5u32 => Ok(Self::ColMajor),
            6u32 => Ok(Self::ArrayStride {
                array_stride: LiteralInteger::parse_one(stream)?,
            }),
            7u32 => Ok(Self::MatrixStride {
                matrix_stride: LiteralInteger::parse_one(stream)?,
            }),
            8u32 => Ok(Self::GLSLShared),
            9u32 => Ok(Self::GLSLPacked),
            10u32 => Ok(Self::CPacked),
            11u32 => Ok(Self::BuiltIn {
                built_in: BuiltIn::parse_one(stream)?,
            }),
            13u32 => Ok(Self::NoPerspective),
            14u32 => Ok(Self::Flat),
            15u32 => Ok(Self::Patch),
            16u32 => Ok(Self::Centroid),
            17u32 => Ok(Self::Sample),
            18u32 => Ok(Self::Invariant),
            19u32 => Ok(Self::Restrict),
            20u32 => Ok(Self::Aliased),
            21u32 => Ok(Self::Volatile),
            22u32 => Ok(Self::Constant),
            23u32 => Ok(Self::Coherent),
            24u32 => Ok(Self::NonWritable),
            25u32 => Ok(Self::NonReadable),
            26u32 => Ok(Self::Uniform),
            27u32 => Ok(Self::UniformId {
                execution: IdScope::parse_one(stream)?,
            }),
            28u32 => Ok(Self::SaturatedConversion),
            29u32 => Ok(Self::Stream {
                stream_number: LiteralInteger::parse_one(stream)?,
            }),
            30u32 => Ok(Self::Location {
                location: LiteralInteger::parse_one(stream)?,
            }),
            31u32 => Ok(Self::Component {
                component: LiteralInteger::parse_one(stream)?,
            }),
            32u32 => Ok(Self::Index {
                index: LiteralInteger::parse_one(stream)?,
            }),
            33u32 => Ok(Self::Binding {
                binding_point: LiteralInteger::parse_one(stream)?,
            }),
            34u32 => Ok(Self::DescriptorSet {
                descriptor_set: LiteralInteger::parse_one(stream)?,
            }),
            35u32 => Ok(Self::Offset {
                byte_offset: LiteralInteger::parse_one(stream)?,
            }),
            36u32 => Ok(Self::XfbBuffer {
                xfb_buffer_number: LiteralInteger::parse_one(stream)?,
            }),
            37u32 => Ok(Self::XfbStride {
                xfb_stride: LiteralInteger::parse_one(stream)?,
            }),
            38u32 => Ok(Self::FuncParamAttr {
                function_parameter_attribute: FunctionParameterAttribute::parse_one(stream)?,
            }),
            39u32 => Ok(Self::FPRoundingMode {
                floating_point_rounding_mode: FPRoundingMode::parse_one(stream)?,
            }),
            40u32 => Ok(Self::FPFastMathMode {
                fast_math_mode: FPFastMathMode::parse_one(stream)?,
            }),
            41u32 => Ok(Self::LinkageAttributes {
                name: LiteralString::parse_one(stream)?,
                linkage_type: LinkageType::parse_one(stream)?,
            }),
            42u32 => Ok(Self::NoContraction),
            43u32 => Ok(Self::InputAttachmentIndex {
                attachment_index: LiteralInteger::parse_one(stream)?,
            }),
            44u32 => Ok(Self::Alignment {
                alignment: LiteralInteger::parse_one(stream)?,
            }),
            45u32 => Ok(Self::MaxByteOffset {
                max_byte_offset: LiteralInteger::parse_one(stream)?,
            }),
            46u32 => Ok(Self::AlignmentId {
                alignment: IdRef::parse_one(stream)?,
            }),
            47u32 => Ok(Self::MaxByteOffsetId {
                max_byte_offset: IdRef::parse_one(stream)?,
            }),
            4216u32 => Ok(Self::SaturatedToLargestFloat8NormalConversionEXT),
            4469u32 => Ok(Self::NoSignedWrap),
            4470u32 => Ok(Self::NoUnsignedWrap),
            4487u32 => Ok(Self::WeightTextureQCOM),
            4488u32 => Ok(Self::BlockMatchTextureQCOM),
            4499u32 => Ok(Self::BlockMatchSamplerQCOM),
            4999u32 => Ok(Self::ExplicitInterpAMD),
            5019u32 => Ok(Self::NodeSharesPayloadLimitsWithAMDX {
                payload_type: IdRef::parse_one(stream)?,
            }),
            5020u32 => Ok(Self::NodeMaxPayloadsAMDX {
                max_number_of_payloads: IdRef::parse_one(stream)?,
            }),
            5078u32 => Ok(Self::TrackFinishWritingAMDX),
            5091u32 => Ok(Self::PayloadNodeNameAMDX {
                node_name: IdRef::parse_one(stream)?,
            }),
            5098u32 => Ok(Self::PayloadNodeBaseIndexAMDX {
                base_index: IdRef::parse_one(stream)?,
            }),
            5099u32 => Ok(Self::PayloadNodeSparseArrayAMDX),
            5100u32 => Ok(Self::PayloadNodeArraySizeAMDX {
                array_size: IdRef::parse_one(stream)?,
            }),
            5105u32 => Ok(Self::PayloadDispatchIndirectAMDX),
            5124u32 => Ok(Self::ArrayStrideIdEXT {
                array_stride: IdRef::parse_one(stream)?,
            }),
            5125u32 => Ok(Self::OffsetIdEXT {
                byte_offset: IdRef::parse_one(stream)?,
            }),
            5145u32 => Ok(Self::UTFEncodedKHR),
            5248u32 => Ok(Self::OverrideCoverageNV),
            5250u32 => Ok(Self::PassthroughNV),
            5252u32 => Ok(Self::ViewportRelativeNV),
            5256u32 => Ok(Self::SecondaryViewportRelativeNV {
                offset: LiteralInteger::parse_one(stream)?,
            }),
            5271u32 => Ok(Self::PerPrimitiveEXT),
            5272u32 => Ok(Self::PerViewNV),
            5273u32 => Ok(Self::PerTaskNV),
            5285u32 => Ok(Self::PerVertexKHR),
            5300u32 => Ok(Self::NonUniform),
            5355u32 => Ok(Self::RestrictPointer),
            5356u32 => Ok(Self::AliasedPointer),
            5358u32 => Ok(Self::MemberOffsetNV {
                member_offset: LiteralInteger::parse_one(stream)?,
            }),
            5386u32 => Ok(Self::HitObjectShaderRecordBufferNV),
            5389u32 => Ok(Self::HitObjectShaderRecordBufferEXT),
            5397u32 => Ok(Self::BankNV {
                bank: LiteralInteger::parse_one(stream)?,
            }),
            5398u32 => Ok(Self::BindlessSamplerNV),
            5399u32 => Ok(Self::BindlessImageNV),
            5400u32 => Ok(Self::BoundSamplerNV),
            5401u32 => Ok(Self::BoundImageNV),
            5599u32 => Ok(Self::SIMTCallINTEL {
                n: LiteralInteger::parse_one(stream)?,
            }),
            5602u32 => Ok(Self::ReferencedIndirectlyINTEL),
            5607u32 => Ok(Self::ClobberINTEL {
                register: LiteralString::parse_one(stream)?,
            }),
            5608u32 => Ok(Self::SideEffectsINTEL),
            5624u32 => Ok(Self::VectorComputeVariableINTEL),
            5625u32 => Ok(Self::FuncParamIOKindINTEL {
                kind: LiteralInteger::parse_one(stream)?,
            }),
            5626u32 => Ok(Self::VectorComputeFunctionINTEL),
            5627u32 => Ok(Self::StackCallINTEL),
            5628u32 => Ok(Self::GlobalVariableOffsetINTEL {
                offset: LiteralInteger::parse_one(stream)?,
            }),
            5634u32 => Ok(Self::CounterBuffer {
                counter_buffer: IdRef::parse_one(stream)?,
            }),
            5635u32 => Ok(Self::UserSemantic {
                semantic: LiteralString::parse_one(stream)?,
            }),
            5636u32 => Ok(Self::UserTypeGOOGLE {
                user_type: LiteralString::parse_one(stream)?,
            }),
            5822u32 => Ok(Self::FunctionRoundingModeINTEL {
                target_width: LiteralInteger::parse_one(stream)?,
                fp_rounding_mode: FPRoundingMode::parse_one(stream)?,
            }),
            5823u32 => Ok(Self::FunctionDenormModeINTEL {
                target_width: LiteralInteger::parse_one(stream)?,
                fp_denorm_mode: FPDenormMode::parse_one(stream)?,
            }),
            5825u32 => Ok(Self::RegisterALTERA),
            5826u32 => Ok(Self::MemoryALTERA {
                memory_type: LiteralString::parse_one(stream)?,
            }),
            5827u32 => Ok(Self::NumbanksALTERA {
                banks: LiteralInteger::parse_one(stream)?,
            }),
            5828u32 => Ok(Self::BankwidthALTERA {
                bank_width: LiteralInteger::parse_one(stream)?,
            }),
            5829u32 => Ok(Self::MaxPrivateCopiesALTERA {
                maximum_copies: LiteralInteger::parse_one(stream)?,
            }),
            5830u32 => Ok(Self::SinglepumpALTERA),
            5831u32 => Ok(Self::DoublepumpALTERA),
            5832u32 => Ok(Self::MaxReplicatesALTERA {
                maximum_replicates: LiteralInteger::parse_one(stream)?,
            }),
            5833u32 => Ok(Self::SimpleDualPortALTERA),
            5834u32 => Ok(Self::MergeALTERA {
                merge_key: LiteralString::parse_one(stream)?,
                merge_type: LiteralString::parse_one(stream)?,
            }),
            5835u32 => Ok(Self::BankBitsALTERA {
                bank_bits: LiteralInteger::parse_one(stream)?,
            }),
            5836u32 => Ok(Self::ForcePow2DepthALTERA {
                force_key: LiteralInteger::parse_one(stream)?,
            }),
            5883u32 => Ok(Self::StridesizeALTERA {
                stride_size: LiteralInteger::parse_one(stream)?,
            }),
            5884u32 => Ok(Self::WordsizeALTERA {
                word_size: LiteralInteger::parse_one(stream)?,
            }),
            5885u32 => Ok(Self::TrueDualPortALTERA),
            5899u32 => Ok(Self::BurstCoalesceALTERA),
            5900u32 => Ok(Self::CacheSizeALTERA {
                cache_size_in_bytes: LiteralInteger::parse_one(stream)?,
            }),
            5901u32 => Ok(Self::DontStaticallyCoalesceALTERA),
            5902u32 => Ok(Self::PrefetchALTERA {
                prefetcher_size_in_bytes: LiteralInteger::parse_one(stream)?,
            }),
            5905u32 => Ok(Self::StallEnableALTERA),
            5907u32 => Ok(Self::FuseLoopsInFunctionALTERA),
            5909u32 => Ok(Self::MathOpDSPModeALTERA {
                mode: LiteralInteger::parse_one(stream)?,
                propagate: LiteralInteger::parse_one(stream)?,
            }),
            5914u32 => Ok(Self::AliasScopeINTEL {
                aliasing_scopes_list: IdRef::parse_one(stream)?,
            }),
            5915u32 => Ok(Self::NoAliasINTEL {
                aliasing_scopes_list: IdRef::parse_one(stream)?,
            }),
            5917u32 => Ok(Self::InitiationIntervalALTERA {
                cycles: LiteralInteger::parse_one(stream)?,
            }),
            5918u32 => Ok(Self::MaxConcurrencyALTERA {
                invocations: LiteralInteger::parse_one(stream)?,
            }),
            5919u32 => Ok(Self::PipelineEnableALTERA {
                enable: LiteralInteger::parse_one(stream)?,
            }),
            5921u32 => Ok(Self::BufferLocationALTERA {
                buffer_location_id: LiteralInteger::parse_one(stream)?,
            }),
            5944u32 => Ok(Self::IOPipeStorageALTERA {
                io_pipe_id: LiteralInteger::parse_one(stream)?,
            }),
            6080u32 => Ok(Self::FunctionFloatingPointModeINTEL {
                target_width: LiteralInteger::parse_one(stream)?,
                fp_operation_mode: FPOperationMode::parse_one(stream)?,
            }),
            6085u32 => Ok(Self::SingleElementVectorINTEL),
            6087u32 => Ok(Self::VectorComputeCallableFunctionINTEL),
            6140u32 => Ok(Self::MediaBlockIOINTEL),
            6151u32 => Ok(Self::StallFreeALTERA),
            6170u32 => Ok(Self::FPMaxErrorDecorationINTEL {
                max_error: LiteralFloat::parse_one(stream)?,
            }),
            6172u32 => Ok(Self::LatencyControlLabelALTERA {
                latency_label: LiteralInteger::parse_one(stream)?,
            }),
            6173u32 => Ok(Self::LatencyControlConstraintALTERA {
                relative_to: LiteralInteger::parse_one(stream)?,
                control_type: LiteralInteger::parse_one(stream)?,
                relative_cycle: LiteralInteger::parse_one(stream)?,
            }),
            6175u32 => Ok(Self::ConduitKernelArgumentALTERA),
            6176u32 => Ok(Self::RegisterMapKernelArgumentALTERA),
            6177u32 => Ok(Self::MMHostInterfaceAddressWidthALTERA {
                address_width: LiteralInteger::parse_one(stream)?,
            }),
            6178u32 => Ok(Self::MMHostInterfaceDataWidthALTERA {
                data_width: LiteralInteger::parse_one(stream)?,
            }),
            6179u32 => Ok(Self::MMHostInterfaceLatencyALTERA {
                latency: LiteralInteger::parse_one(stream)?,
            }),
            6180u32 => Ok(Self::MMHostInterfaceReadWriteModeALTERA {
                read_write_mode: AccessQualifier::parse_one(stream)?,
            }),
            6181u32 => Ok(Self::MMHostInterfaceMaxBurstALTERA {
                max_burst_count: LiteralInteger::parse_one(stream)?,
            }),
            6182u32 => Ok(Self::MMHostInterfaceWaitRequestALTERA {
                waitrequest: LiteralInteger::parse_one(stream)?,
            }),
            6183u32 => Ok(Self::StableKernelArgumentALTERA),
            6188u32 => Ok(Self::HostAccessINTEL {
                access: HostAccessQualifier::parse_one(stream)?,
                name: LiteralString::parse_one(stream)?,
            }),
            6190u32 => Ok(Self::InitModeALTERA {
                trigger: InitializationModeQualifier::parse_one(stream)?,
            }),
            6191u32 => Ok(Self::ImplementInRegisterMapALTERA {
                value: LiteralInteger::parse_one(stream)?,
            }),
            6247u32 => Ok(Self::ConditionalINTEL {
                condition: IdRef::parse_one(stream)?,
            }),
            6442u32 => Ok(Self::CacheControlLoadINTEL {
                cache_level: LiteralInteger::parse_one(stream)?,
                cache_control: LoadCacheControl::parse_one(stream)?,
            }),
            6443u32 => Ok(Self::CacheControlStoreINTEL {
                cache_level: LiteralInteger::parse_one(stream)?,
                cache_control: StoreCacheControl::parse_one(stream)?,
            }),
            x => Err(ParseError::UnknownVariant {
                kind: stringify!(Decoration),
                value: x,
            }),
        }
    }
}
impl<'a> Display for Decoration<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RelaxedPrecision => write!(f, "{}", stringify!(RelaxedPrecision)),
            Self::SpecId {
                specialization_constant_id,
            } => {
                write!(f, "{}", stringify!(SpecId))?;
                write!(f, " {}", specialization_constant_id)?;
                Ok(())
            }
            Self::Block => write!(f, "{}", stringify!(Block)),
            Self::BufferBlock => write!(f, "{}", stringify!(BufferBlock)),
            Self::RowMajor => write!(f, "{}", stringify!(RowMajor)),
            Self::ColMajor => write!(f, "{}", stringify!(ColMajor)),
            Self::ArrayStride { array_stride } => {
                write!(f, "{}", stringify!(ArrayStride))?;
                write!(f, " {}", array_stride)?;
                Ok(())
            }
            Self::MatrixStride { matrix_stride } => {
                write!(f, "{}", stringify!(MatrixStride))?;
                write!(f, " {}", matrix_stride)?;
                Ok(())
            }
            Self::GLSLShared => write!(f, "{}", stringify!(GLSLShared)),
            Self::GLSLPacked => write!(f, "{}", stringify!(GLSLPacked)),
            Self::CPacked => write!(f, "{}", stringify!(CPacked)),
            Self::BuiltIn { built_in } => {
                write!(f, "{}", stringify!(BuiltIn))?;
                write!(f, " {}", built_in)?;
                Ok(())
            }
            Self::NoPerspective => write!(f, "{}", stringify!(NoPerspective)),
            Self::Flat => write!(f, "{}", stringify!(Flat)),
            Self::Patch => write!(f, "{}", stringify!(Patch)),
            Self::Centroid => write!(f, "{}", stringify!(Centroid)),
            Self::Sample => write!(f, "{}", stringify!(Sample)),
            Self::Invariant => write!(f, "{}", stringify!(Invariant)),
            Self::Restrict => write!(f, "{}", stringify!(Restrict)),
            Self::Aliased => write!(f, "{}", stringify!(Aliased)),
            Self::Volatile => write!(f, "{}", stringify!(Volatile)),
            Self::Constant => write!(f, "{}", stringify!(Constant)),
            Self::Coherent => write!(f, "{}", stringify!(Coherent)),
            Self::NonWritable => write!(f, "{}", stringify!(NonWritable)),
            Self::NonReadable => write!(f, "{}", stringify!(NonReadable)),
            Self::Uniform => write!(f, "{}", stringify!(Uniform)),
            Self::UniformId { execution } => {
                write!(f, "{}", stringify!(UniformId))?;
                write!(f, " {}", execution)?;
                Ok(())
            }
            Self::SaturatedConversion => write!(f, "{}", stringify!(SaturatedConversion)),
            Self::Stream { stream_number } => {
                write!(f, "{}", stringify!(Stream))?;
                write!(f, " {}", stream_number)?;
                Ok(())
            }
            Self::Location { location } => {
                write!(f, "{}", stringify!(Location))?;
                write!(f, " {}", location)?;
                Ok(())
            }
            Self::Component { component } => {
                write!(f, "{}", stringify!(Component))?;
                write!(f, " {}", component)?;
                Ok(())
            }
            Self::Index { index } => {
                write!(f, "{}", stringify!(Index))?;
                write!(f, " {}", index)?;
                Ok(())
            }
            Self::Binding { binding_point } => {
                write!(f, "{}", stringify!(Binding))?;
                write!(f, " {}", binding_point)?;
                Ok(())
            }
            Self::DescriptorSet { descriptor_set } => {
                write!(f, "{}", stringify!(DescriptorSet))?;
                write!(f, " {}", descriptor_set)?;
                Ok(())
            }
            Self::Offset { byte_offset } => {
                write!(f, "{}", stringify!(Offset))?;
                write!(f, " {}", byte_offset)?;
                Ok(())
            }
            Self::XfbBuffer { xfb_buffer_number } => {
                write!(f, "{}", stringify!(XfbBuffer))?;
                write!(f, " {}", xfb_buffer_number)?;
                Ok(())
            }
            Self::XfbStride { xfb_stride } => {
                write!(f, "{}", stringify!(XfbStride))?;
                write!(f, " {}", xfb_stride)?;
                Ok(())
            }
            Self::FuncParamAttr {
                function_parameter_attribute,
            } => {
                write!(f, "{}", stringify!(FuncParamAttr))?;
                write!(f, " {}", function_parameter_attribute)?;
                Ok(())
            }
            Self::FPRoundingMode {
                floating_point_rounding_mode,
            } => {
                write!(f, "{}", stringify!(FPRoundingMode))?;
                write!(f, " {}", floating_point_rounding_mode)?;
                Ok(())
            }
            Self::FPFastMathMode { fast_math_mode } => {
                write!(f, "{}", stringify!(FPFastMathMode))?;
                write!(f, " {}", fast_math_mode)?;
                Ok(())
            }
            Self::LinkageAttributes { name, linkage_type } => {
                write!(f, "{}", stringify!(LinkageAttributes))?;
                write!(f, " {}", name)?;
                write!(f, " {}", linkage_type)?;
                Ok(())
            }
            Self::NoContraction => write!(f, "{}", stringify!(NoContraction)),
            Self::InputAttachmentIndex { attachment_index } => {
                write!(f, "{}", stringify!(InputAttachmentIndex))?;
                write!(f, " {}", attachment_index)?;
                Ok(())
            }
            Self::Alignment { alignment } => {
                write!(f, "{}", stringify!(Alignment))?;
                write!(f, " {}", alignment)?;
                Ok(())
            }
            Self::MaxByteOffset { max_byte_offset } => {
                write!(f, "{}", stringify!(MaxByteOffset))?;
                write!(f, " {}", max_byte_offset)?;
                Ok(())
            }
            Self::AlignmentId { alignment } => {
                write!(f, "{}", stringify!(AlignmentId))?;
                write!(f, " {}", alignment)?;
                Ok(())
            }
            Self::MaxByteOffsetId { max_byte_offset } => {
                write!(f, "{}", stringify!(MaxByteOffsetId))?;
                write!(f, " {}", max_byte_offset)?;
                Ok(())
            }
            Self::SaturatedToLargestFloat8NormalConversionEXT => write!(
                f,
                "{}",
                stringify!(SaturatedToLargestFloat8NormalConversionEXT)
            ),
            Self::NoSignedWrap => write!(f, "{}", stringify!(NoSignedWrap)),
            Self::NoUnsignedWrap => write!(f, "{}", stringify!(NoUnsignedWrap)),
            Self::WeightTextureQCOM => write!(f, "{}", stringify!(WeightTextureQCOM)),
            Self::BlockMatchTextureQCOM => write!(f, "{}", stringify!(BlockMatchTextureQCOM)),
            Self::BlockMatchSamplerQCOM => write!(f, "{}", stringify!(BlockMatchSamplerQCOM)),
            Self::ExplicitInterpAMD => write!(f, "{}", stringify!(ExplicitInterpAMD)),
            Self::NodeSharesPayloadLimitsWithAMDX { payload_type } => {
                write!(f, "{}", stringify!(NodeSharesPayloadLimitsWithAMDX))?;
                write!(f, " {}", payload_type)?;
                Ok(())
            }
            Self::NodeMaxPayloadsAMDX {
                max_number_of_payloads,
            } => {
                write!(f, "{}", stringify!(NodeMaxPayloadsAMDX))?;
                write!(f, " {}", max_number_of_payloads)?;
                Ok(())
            }
            Self::TrackFinishWritingAMDX => write!(f, "{}", stringify!(TrackFinishWritingAMDX)),
            Self::PayloadNodeNameAMDX { node_name } => {
                write!(f, "{}", stringify!(PayloadNodeNameAMDX))?;
                write!(f, " {}", node_name)?;
                Ok(())
            }
            Self::PayloadNodeBaseIndexAMDX { base_index } => {
                write!(f, "{}", stringify!(PayloadNodeBaseIndexAMDX))?;
                write!(f, " {}", base_index)?;
                Ok(())
            }
            Self::PayloadNodeSparseArrayAMDX => {
                write!(f, "{}", stringify!(PayloadNodeSparseArrayAMDX))
            }
            Self::PayloadNodeArraySizeAMDX { array_size } => {
                write!(f, "{}", stringify!(PayloadNodeArraySizeAMDX))?;
                write!(f, " {}", array_size)?;
                Ok(())
            }
            Self::PayloadDispatchIndirectAMDX => {
                write!(f, "{}", stringify!(PayloadDispatchIndirectAMDX))
            }
            Self::ArrayStrideIdEXT { array_stride } => {
                write!(f, "{}", stringify!(ArrayStrideIdEXT))?;
                write!(f, " {}", array_stride)?;
                Ok(())
            }
            Self::OffsetIdEXT { byte_offset } => {
                write!(f, "{}", stringify!(OffsetIdEXT))?;
                write!(f, " {}", byte_offset)?;
                Ok(())
            }
            Self::UTFEncodedKHR => write!(f, "{}", stringify!(UTFEncodedKHR)),
            Self::OverrideCoverageNV => write!(f, "{}", stringify!(OverrideCoverageNV)),
            Self::PassthroughNV => write!(f, "{}", stringify!(PassthroughNV)),
            Self::ViewportRelativeNV => write!(f, "{}", stringify!(ViewportRelativeNV)),
            Self::SecondaryViewportRelativeNV { offset } => {
                write!(f, "{}", stringify!(SecondaryViewportRelativeNV))?;
                write!(f, " {}", offset)?;
                Ok(())
            }
            Self::PerPrimitiveEXT => write!(f, "{}", stringify!(PerPrimitiveEXT)),
            Self::PerViewNV => write!(f, "{}", stringify!(PerViewNV)),
            Self::PerTaskNV => write!(f, "{}", stringify!(PerTaskNV)),
            Self::PerVertexKHR => write!(f, "{}", stringify!(PerVertexKHR)),
            Self::NonUniform => write!(f, "{}", stringify!(NonUniform)),
            Self::RestrictPointer => write!(f, "{}", stringify!(RestrictPointer)),
            Self::AliasedPointer => write!(f, "{}", stringify!(AliasedPointer)),
            Self::MemberOffsetNV { member_offset } => {
                write!(f, "{}", stringify!(MemberOffsetNV))?;
                write!(f, " {}", member_offset)?;
                Ok(())
            }
            Self::HitObjectShaderRecordBufferNV => {
                write!(f, "{}", stringify!(HitObjectShaderRecordBufferNV))
            }
            Self::HitObjectShaderRecordBufferEXT => {
                write!(f, "{}", stringify!(HitObjectShaderRecordBufferEXT))
            }
            Self::BankNV { bank } => {
                write!(f, "{}", stringify!(BankNV))?;
                write!(f, " {}", bank)?;
                Ok(())
            }
            Self::BindlessSamplerNV => write!(f, "{}", stringify!(BindlessSamplerNV)),
            Self::BindlessImageNV => write!(f, "{}", stringify!(BindlessImageNV)),
            Self::BoundSamplerNV => write!(f, "{}", stringify!(BoundSamplerNV)),
            Self::BoundImageNV => write!(f, "{}", stringify!(BoundImageNV)),
            Self::SIMTCallINTEL { n } => {
                write!(f, "{}", stringify!(SIMTCallINTEL))?;
                write!(f, " {}", n)?;
                Ok(())
            }
            Self::ReferencedIndirectlyINTEL => {
                write!(f, "{}", stringify!(ReferencedIndirectlyINTEL))
            }
            Self::ClobberINTEL { register } => {
                write!(f, "{}", stringify!(ClobberINTEL))?;
                write!(f, " {}", register)?;
                Ok(())
            }
            Self::SideEffectsINTEL => write!(f, "{}", stringify!(SideEffectsINTEL)),
            Self::VectorComputeVariableINTEL => {
                write!(f, "{}", stringify!(VectorComputeVariableINTEL))
            }
            Self::FuncParamIOKindINTEL { kind } => {
                write!(f, "{}", stringify!(FuncParamIOKindINTEL))?;
                write!(f, " {}", kind)?;
                Ok(())
            }
            Self::VectorComputeFunctionINTEL => {
                write!(f, "{}", stringify!(VectorComputeFunctionINTEL))
            }
            Self::StackCallINTEL => write!(f, "{}", stringify!(StackCallINTEL)),
            Self::GlobalVariableOffsetINTEL { offset } => {
                write!(f, "{}", stringify!(GlobalVariableOffsetINTEL))?;
                write!(f, " {}", offset)?;
                Ok(())
            }
            Self::CounterBuffer { counter_buffer } => {
                write!(f, "{}", stringify!(CounterBuffer))?;
                write!(f, " {}", counter_buffer)?;
                Ok(())
            }
            Self::UserSemantic { semantic } => {
                write!(f, "{}", stringify!(UserSemantic))?;
                write!(f, " {}", semantic)?;
                Ok(())
            }
            Self::UserTypeGOOGLE { user_type } => {
                write!(f, "{}", stringify!(UserTypeGOOGLE))?;
                write!(f, " {}", user_type)?;
                Ok(())
            }
            Self::FunctionRoundingModeINTEL {
                target_width,
                fp_rounding_mode,
            } => {
                write!(f, "{}", stringify!(FunctionRoundingModeINTEL))?;
                write!(f, " {}", target_width)?;
                write!(f, " {}", fp_rounding_mode)?;
                Ok(())
            }
            Self::FunctionDenormModeINTEL {
                target_width,
                fp_denorm_mode,
            } => {
                write!(f, "{}", stringify!(FunctionDenormModeINTEL))?;
                write!(f, " {}", target_width)?;
                write!(f, " {}", fp_denorm_mode)?;
                Ok(())
            }
            Self::RegisterALTERA => write!(f, "{}", stringify!(RegisterALTERA)),
            Self::MemoryALTERA { memory_type } => {
                write!(f, "{}", stringify!(MemoryALTERA))?;
                write!(f, " {}", memory_type)?;
                Ok(())
            }
            Self::NumbanksALTERA { banks } => {
                write!(f, "{}", stringify!(NumbanksALTERA))?;
                write!(f, " {}", banks)?;
                Ok(())
            }
            Self::BankwidthALTERA { bank_width } => {
                write!(f, "{}", stringify!(BankwidthALTERA))?;
                write!(f, " {}", bank_width)?;
                Ok(())
            }
            Self::MaxPrivateCopiesALTERA { maximum_copies } => {
                write!(f, "{}", stringify!(MaxPrivateCopiesALTERA))?;
                write!(f, " {}", maximum_copies)?;
                Ok(())
            }
            Self::SinglepumpALTERA => write!(f, "{}", stringify!(SinglepumpALTERA)),
            Self::DoublepumpALTERA => write!(f, "{}", stringify!(DoublepumpALTERA)),
            Self::MaxReplicatesALTERA { maximum_replicates } => {
                write!(f, "{}", stringify!(MaxReplicatesALTERA))?;
                write!(f, " {}", maximum_replicates)?;
                Ok(())
            }
            Self::SimpleDualPortALTERA => write!(f, "{}", stringify!(SimpleDualPortALTERA)),
            Self::MergeALTERA {
                merge_key,
                merge_type,
            } => {
                write!(f, "{}", stringify!(MergeALTERA))?;
                write!(f, " {}", merge_key)?;
                write!(f, " {}", merge_type)?;
                Ok(())
            }
            Self::BankBitsALTERA { bank_bits } => {
                write!(f, "{}", stringify!(BankBitsALTERA))?;
                write!(f, " {}", bank_bits)?;
                Ok(())
            }
            Self::ForcePow2DepthALTERA { force_key } => {
                write!(f, "{}", stringify!(ForcePow2DepthALTERA))?;
                write!(f, " {}", force_key)?;
                Ok(())
            }
            Self::StridesizeALTERA { stride_size } => {
                write!(f, "{}", stringify!(StridesizeALTERA))?;
                write!(f, " {}", stride_size)?;
                Ok(())
            }
            Self::WordsizeALTERA { word_size } => {
                write!(f, "{}", stringify!(WordsizeALTERA))?;
                write!(f, " {}", word_size)?;
                Ok(())
            }
            Self::TrueDualPortALTERA => write!(f, "{}", stringify!(TrueDualPortALTERA)),
            Self::BurstCoalesceALTERA => write!(f, "{}", stringify!(BurstCoalesceALTERA)),
            Self::CacheSizeALTERA {
                cache_size_in_bytes,
            } => {
                write!(f, "{}", stringify!(CacheSizeALTERA))?;
                write!(f, " {}", cache_size_in_bytes)?;
                Ok(())
            }
            Self::DontStaticallyCoalesceALTERA => {
                write!(f, "{}", stringify!(DontStaticallyCoalesceALTERA))
            }
            Self::PrefetchALTERA {
                prefetcher_size_in_bytes,
            } => {
                write!(f, "{}", stringify!(PrefetchALTERA))?;
                write!(f, " {}", prefetcher_size_in_bytes)?;
                Ok(())
            }
            Self::StallEnableALTERA => write!(f, "{}", stringify!(StallEnableALTERA)),
            Self::FuseLoopsInFunctionALTERA => {
                write!(f, "{}", stringify!(FuseLoopsInFunctionALTERA))
            }
            Self::MathOpDSPModeALTERA { mode, propagate } => {
                write!(f, "{}", stringify!(MathOpDSPModeALTERA))?;
                write!(f, " {}", mode)?;
                write!(f, " {}", propagate)?;
                Ok(())
            }
            Self::AliasScopeINTEL {
                aliasing_scopes_list,
            } => {
                write!(f, "{}", stringify!(AliasScopeINTEL))?;
                write!(f, " {}", aliasing_scopes_list)?;
                Ok(())
            }
            Self::NoAliasINTEL {
                aliasing_scopes_list,
            } => {
                write!(f, "{}", stringify!(NoAliasINTEL))?;
                write!(f, " {}", aliasing_scopes_list)?;
                Ok(())
            }
            Self::InitiationIntervalALTERA { cycles } => {
                write!(f, "{}", stringify!(InitiationIntervalALTERA))?;
                write!(f, " {}", cycles)?;
                Ok(())
            }
            Self::MaxConcurrencyALTERA { invocations } => {
                write!(f, "{}", stringify!(MaxConcurrencyALTERA))?;
                write!(f, " {}", invocations)?;
                Ok(())
            }
            Self::PipelineEnableALTERA { enable } => {
                write!(f, "{}", stringify!(PipelineEnableALTERA))?;
                write!(f, " {}", enable)?;
                Ok(())
            }
            Self::BufferLocationALTERA { buffer_location_id } => {
                write!(f, "{}", stringify!(BufferLocationALTERA))?;
                write!(f, " {}", buffer_location_id)?;
                Ok(())
            }
            Self::IOPipeStorageALTERA { io_pipe_id } => {
                write!(f, "{}", stringify!(IOPipeStorageALTERA))?;
                write!(f, " {}", io_pipe_id)?;
                Ok(())
            }
            Self::FunctionFloatingPointModeINTEL {
                target_width,
                fp_operation_mode,
            } => {
                write!(f, "{}", stringify!(FunctionFloatingPointModeINTEL))?;
                write!(f, " {}", target_width)?;
                write!(f, " {}", fp_operation_mode)?;
                Ok(())
            }
            Self::SingleElementVectorINTEL => write!(f, "{}", stringify!(SingleElementVectorINTEL)),
            Self::VectorComputeCallableFunctionINTEL => {
                write!(f, "{}", stringify!(VectorComputeCallableFunctionINTEL))
            }
            Self::MediaBlockIOINTEL => write!(f, "{}", stringify!(MediaBlockIOINTEL)),
            Self::StallFreeALTERA => write!(f, "{}", stringify!(StallFreeALTERA)),
            Self::FPMaxErrorDecorationINTEL { max_error } => {
                write!(f, "{}", stringify!(FPMaxErrorDecorationINTEL))?;
                write!(f, " {}", max_error)?;
                Ok(())
            }
            Self::LatencyControlLabelALTERA { latency_label } => {
                write!(f, "{}", stringify!(LatencyControlLabelALTERA))?;
                write!(f, " {}", latency_label)?;
                Ok(())
            }
            Self::LatencyControlConstraintALTERA {
                relative_to,
                control_type,
                relative_cycle,
            } => {
                write!(f, "{}", stringify!(LatencyControlConstraintALTERA))?;
                write!(f, " {}", relative_to)?;
                write!(f, " {}", control_type)?;
                write!(f, " {}", relative_cycle)?;
                Ok(())
            }
            Self::ConduitKernelArgumentALTERA => {
                write!(f, "{}", stringify!(ConduitKernelArgumentALTERA))
            }
            Self::RegisterMapKernelArgumentALTERA => {
                write!(f, "{}", stringify!(RegisterMapKernelArgumentALTERA))
            }
            Self::MMHostInterfaceAddressWidthALTERA { address_width } => {
                write!(f, "{}", stringify!(MMHostInterfaceAddressWidthALTERA))?;
                write!(f, " {}", address_width)?;
                Ok(())
            }
            Self::MMHostInterfaceDataWidthALTERA { data_width } => {
                write!(f, "{}", stringify!(MMHostInterfaceDataWidthALTERA))?;
                write!(f, " {}", data_width)?;
                Ok(())
            }
            Self::MMHostInterfaceLatencyALTERA { latency } => {
                write!(f, "{}", stringify!(MMHostInterfaceLatencyALTERA))?;
                write!(f, " {}", latency)?;
                Ok(())
            }
            Self::MMHostInterfaceReadWriteModeALTERA { read_write_mode } => {
                write!(f, "{}", stringify!(MMHostInterfaceReadWriteModeALTERA))?;
                write!(f, " {}", read_write_mode)?;
                Ok(())
            }
            Self::MMHostInterfaceMaxBurstALTERA { max_burst_count } => {
                write!(f, "{}", stringify!(MMHostInterfaceMaxBurstALTERA))?;
                write!(f, " {}", max_burst_count)?;
                Ok(())
            }
            Self::MMHostInterfaceWaitRequestALTERA { waitrequest } => {
                write!(f, "{}", stringify!(MMHostInterfaceWaitRequestALTERA))?;
                write!(f, " {}", waitrequest)?;
                Ok(())
            }
            Self::StableKernelArgumentALTERA => {
                write!(f, "{}", stringify!(StableKernelArgumentALTERA))
            }
            Self::HostAccessINTEL { access, name } => {
                write!(f, "{}", stringify!(HostAccessINTEL))?;
                write!(f, " {}", access)?;
                write!(f, " {}", name)?;
                Ok(())
            }
            Self::InitModeALTERA { trigger } => {
                write!(f, "{}", stringify!(InitModeALTERA))?;
                write!(f, " {}", trigger)?;
                Ok(())
            }
            Self::ImplementInRegisterMapALTERA { value } => {
                write!(f, "{}", stringify!(ImplementInRegisterMapALTERA))?;
                write!(f, " {}", value)?;
                Ok(())
            }
            Self::ConditionalINTEL { condition } => {
                write!(f, "{}", stringify!(ConditionalINTEL))?;
                write!(f, " {}", condition)?;
                Ok(())
            }
            Self::CacheControlLoadINTEL {
                cache_level,
                cache_control,
            } => {
                write!(f, "{}", stringify!(CacheControlLoadINTEL))?;
                write!(f, " {}", cache_level)?;
                write!(f, " {}", cache_control)?;
                Ok(())
            }
            Self::CacheControlStoreINTEL {
                cache_level,
                cache_control,
            } => {
                write!(f, "{}", stringify!(CacheControlStoreINTEL))?;
                write!(f, " {}", cache_level)?;
                write!(f, " {}", cache_control)?;
                Ok(())
            }
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BuiltIn(pub(crate) u32);
impl BuiltIn {
    pub const POSITION: Self = Self(0u32);
    pub const POINT_SIZE: Self = Self(1u32);
    pub const CLIP_DISTANCE: Self = Self(3u32);
    pub const CULL_DISTANCE: Self = Self(4u32);
    pub const VERTEX_ID: Self = Self(5u32);
    pub const INSTANCE_ID: Self = Self(6u32);
    pub const PRIMITIVE_ID: Self = Self(7u32);
    pub const INVOCATION_ID: Self = Self(8u32);
    pub const LAYER: Self = Self(9u32);
    pub const VIEWPORT_INDEX: Self = Self(10u32);
    pub const TESS_LEVEL_OUTER: Self = Self(11u32);
    pub const TESS_LEVEL_INNER: Self = Self(12u32);
    pub const TESS_COORD: Self = Self(13u32);
    pub const PATCH_VERTICES: Self = Self(14u32);
    pub const FRAG_COORD: Self = Self(15u32);
    pub const POINT_COORD: Self = Self(16u32);
    pub const FRONT_FACING: Self = Self(17u32);
    pub const SAMPLE_ID: Self = Self(18u32);
    pub const SAMPLE_POSITION: Self = Self(19u32);
    pub const SAMPLE_MASK: Self = Self(20u32);
    pub const FRAG_DEPTH: Self = Self(22u32);
    pub const HELPER_INVOCATION: Self = Self(23u32);
    pub const NUM_WORKGROUPS: Self = Self(24u32);
    pub const WORKGROUP_SIZE: Self = Self(25u32);
    pub const WORKGROUP_ID: Self = Self(26u32);
    pub const LOCAL_INVOCATION_ID: Self = Self(27u32);
    pub const GLOBAL_INVOCATION_ID: Self = Self(28u32);
    pub const LOCAL_INVOCATION_INDEX: Self = Self(29u32);
    pub const WORK_DIM: Self = Self(30u32);
    pub const GLOBAL_SIZE: Self = Self(31u32);
    pub const ENQUEUED_WORKGROUP_SIZE: Self = Self(32u32);
    pub const GLOBAL_OFFSET: Self = Self(33u32);
    pub const GLOBAL_LINEAR_ID: Self = Self(34u32);
    pub const SUBGROUP_SIZE: Self = Self(36u32);
    pub const SUBGROUP_MAX_SIZE: Self = Self(37u32);
    pub const NUM_SUBGROUPS: Self = Self(38u32);
    pub const NUM_ENQUEUED_SUBGROUPS: Self = Self(39u32);
    pub const SUBGROUP_ID: Self = Self(40u32);
    pub const SUBGROUP_LOCAL_INVOCATION_ID: Self = Self(41u32);
    pub const VERTEX_INDEX: Self = Self(42u32);
    pub const INSTANCE_INDEX: Self = Self(43u32);
    pub const CORE_IDARM: Self = Self(4160u32);
    pub const CORE_COUNT_ARM: Self = Self(4161u32);
    pub const CORE_MAX_IDARM: Self = Self(4162u32);
    pub const WARP_IDARM: Self = Self(4163u32);
    pub const WARP_MAX_IDARM: Self = Self(4164u32);
    pub const SUBGROUP_EQ_MASK: Self = Self(4416u32);
    pub const SUBGROUP_GE_MASK: Self = Self(4417u32);
    pub const SUBGROUP_GT_MASK: Self = Self(4418u32);
    pub const SUBGROUP_LE_MASK: Self = Self(4419u32);
    pub const SUBGROUP_LT_MASK: Self = Self(4420u32);
    pub const BASE_VERTEX: Self = Self(4424u32);
    pub const BASE_INSTANCE: Self = Self(4425u32);
    pub const DRAW_INDEX: Self = Self(4426u32);
    pub const PRIMITIVE_SHADING_RATE_KHR: Self = Self(4432u32);
    pub const DEVICE_INDEX: Self = Self(4438u32);
    pub const VIEW_INDEX: Self = Self(4440u32);
    pub const SHADING_RATE_KHR: Self = Self(4444u32);
    pub const TILE_OFFSET_QCOM: Self = Self(4492u32);
    pub const TILE_DIMENSION_QCOM: Self = Self(4493u32);
    pub const TILE_APRON_SIZE_QCOM: Self = Self(4494u32);
    pub const BARY_COORD_NO_PERSP_AMD: Self = Self(4992u32);
    pub const BARY_COORD_NO_PERSP_CENTROID_AMD: Self = Self(4993u32);
    pub const BARY_COORD_NO_PERSP_SAMPLE_AMD: Self = Self(4994u32);
    pub const BARY_COORD_SMOOTH_AMD: Self = Self(4995u32);
    pub const BARY_COORD_SMOOTH_CENTROID_AMD: Self = Self(4996u32);
    pub const BARY_COORD_SMOOTH_SAMPLE_AMD: Self = Self(4997u32);
    pub const BARY_COORD_PULL_MODEL_AMD: Self = Self(4998u32);
    pub const FRAG_STENCIL_REF_EXT: Self = Self(5014u32);
    pub const REMAINING_RECURSION_LEVELS_AMDX: Self = Self(5021u32);
    pub const SHADER_INDEX_AMDX: Self = Self(5073u32);
    pub const SAMPLER_HEAP_EXT: Self = Self(5122u32);
    pub const RESOURCE_HEAP_EXT: Self = Self(5123u32);
    pub const VIEWPORT_MASK_NV: Self = Self(5253u32);
    pub const SECONDARY_POSITION_NV: Self = Self(5257u32);
    pub const SECONDARY_VIEWPORT_MASK_NV: Self = Self(5258u32);
    pub const POSITION_PER_VIEW_NV: Self = Self(5261u32);
    pub const VIEWPORT_MASK_PER_VIEW_NV: Self = Self(5262u32);
    pub const FULLY_COVERED_EXT: Self = Self(5264u32);
    pub const TASK_COUNT_NV: Self = Self(5274u32);
    pub const PRIMITIVE_COUNT_NV: Self = Self(5275u32);
    pub const PRIMITIVE_INDICES_NV: Self = Self(5276u32);
    pub const CLIP_DISTANCE_PER_VIEW_NV: Self = Self(5277u32);
    pub const CULL_DISTANCE_PER_VIEW_NV: Self = Self(5278u32);
    pub const LAYER_PER_VIEW_NV: Self = Self(5279u32);
    pub const MESH_VIEW_COUNT_NV: Self = Self(5280u32);
    pub const MESH_VIEW_INDICES_NV: Self = Self(5281u32);
    pub const BARY_COORD_KHR: Self = Self(5286u32);
    pub const BARY_COORD_NO_PERSP_KHR: Self = Self(5287u32);
    pub const FRAG_SIZE_EXT: Self = Self(5292u32);
    pub const FRAG_INVOCATION_COUNT_EXT: Self = Self(5293u32);
    pub const PRIMITIVE_POINT_INDICES_EXT: Self = Self(5294u32);
    pub const PRIMITIVE_LINE_INDICES_EXT: Self = Self(5295u32);
    pub const PRIMITIVE_TRIANGLE_INDICES_EXT: Self = Self(5296u32);
    pub const CULL_PRIMITIVE_EXT: Self = Self(5299u32);
    pub const LAUNCH_ID_KHR: Self = Self(5319u32);
    pub const LAUNCH_SIZE_KHR: Self = Self(5320u32);
    pub const WORLD_RAY_ORIGIN_KHR: Self = Self(5321u32);
    pub const WORLD_RAY_DIRECTION_KHR: Self = Self(5322u32);
    pub const OBJECT_RAY_ORIGIN_KHR: Self = Self(5323u32);
    pub const OBJECT_RAY_DIRECTION_KHR: Self = Self(5324u32);
    pub const RAY_TMIN_KHR: Self = Self(5325u32);
    pub const RAY_TMAX_KHR: Self = Self(5326u32);
    pub const INSTANCE_CUSTOM_INDEX_KHR: Self = Self(5327u32);
    pub const OBJECT_TO_WORLD_KHR: Self = Self(5330u32);
    pub const WORLD_TO_OBJECT_KHR: Self = Self(5331u32);
    pub const HIT_TNV: Self = Self(5332u32);
    pub const HIT_KIND_KHR: Self = Self(5333u32);
    pub const CURRENT_RAY_TIME_NV: Self = Self(5334u32);
    pub const HIT_TRIANGLE_VERTEX_POSITIONS_KHR: Self = Self(5335u32);
    pub const HIT_MICRO_TRIANGLE_VERTEX_POSITIONS_NV: Self = Self(5337u32);
    pub const HIT_MICRO_TRIANGLE_VERTEX_BARYCENTRICS_NV: Self = Self(5344u32);
    pub const INCOMING_RAY_FLAGS_KHR: Self = Self(5351u32);
    pub const RAY_GEOMETRY_INDEX_KHR: Self = Self(5352u32);
    pub const HIT_IS_SPHERE_NV: Self = Self(5359u32);
    pub const HIT_IS_LSSNV: Self = Self(5360u32);
    pub const HIT_SPHERE_POSITION_NV: Self = Self(5361u32);
    pub const WARPS_PER_SMNV: Self = Self(5374u32);
    pub const SMCOUNT_NV: Self = Self(5375u32);
    pub const WARP_IDNV: Self = Self(5376u32);
    pub const SMIDNV: Self = Self(5377u32);
    pub const HIT_LSSPOSITIONS_NV: Self = Self(5396u32);
    pub const HIT_KIND_FRONT_FACING_MICRO_TRIANGLE_NV: Self = Self(5405u32);
    pub const HIT_KIND_BACK_FACING_MICRO_TRIANGLE_NV: Self = Self(5406u32);
    pub const HIT_SPHERE_RADIUS_NV: Self = Self(5420u32);
    pub const HIT_LSSRADII_NV: Self = Self(5421u32);
    pub const CLUSTER_IDNV: Self = Self(5436u32);
    pub const CULL_MASK_KHR: Self = Self(6021u32);
}
impl Word for BuiltIn {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for BuiltIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::POSITION => write!(f, "Position"),
            Self::POINT_SIZE => write!(f, "PointSize"),
            Self::CLIP_DISTANCE => write!(f, "ClipDistance"),
            Self::CULL_DISTANCE => write!(f, "CullDistance"),
            Self::VERTEX_ID => write!(f, "VertexId"),
            Self::INSTANCE_ID => write!(f, "InstanceId"),
            Self::PRIMITIVE_ID => write!(f, "PrimitiveId"),
            Self::INVOCATION_ID => write!(f, "InvocationId"),
            Self::LAYER => write!(f, "Layer"),
            Self::VIEWPORT_INDEX => write!(f, "ViewportIndex"),
            Self::TESS_LEVEL_OUTER => write!(f, "TessLevelOuter"),
            Self::TESS_LEVEL_INNER => write!(f, "TessLevelInner"),
            Self::TESS_COORD => write!(f, "TessCoord"),
            Self::PATCH_VERTICES => write!(f, "PatchVertices"),
            Self::FRAG_COORD => write!(f, "FragCoord"),
            Self::POINT_COORD => write!(f, "PointCoord"),
            Self::FRONT_FACING => write!(f, "FrontFacing"),
            Self::SAMPLE_ID => write!(f, "SampleId"),
            Self::SAMPLE_POSITION => write!(f, "SamplePosition"),
            Self::SAMPLE_MASK => write!(f, "SampleMask"),
            Self::FRAG_DEPTH => write!(f, "FragDepth"),
            Self::HELPER_INVOCATION => write!(f, "HelperInvocation"),
            Self::NUM_WORKGROUPS => write!(f, "NumWorkgroups"),
            Self::WORKGROUP_SIZE => write!(f, "WorkgroupSize"),
            Self::WORKGROUP_ID => write!(f, "WorkgroupId"),
            Self::LOCAL_INVOCATION_ID => write!(f, "LocalInvocationId"),
            Self::GLOBAL_INVOCATION_ID => write!(f, "GlobalInvocationId"),
            Self::LOCAL_INVOCATION_INDEX => write!(f, "LocalInvocationIndex"),
            Self::WORK_DIM => write!(f, "WorkDim"),
            Self::GLOBAL_SIZE => write!(f, "GlobalSize"),
            Self::ENQUEUED_WORKGROUP_SIZE => write!(f, "EnqueuedWorkgroupSize"),
            Self::GLOBAL_OFFSET => write!(f, "GlobalOffset"),
            Self::GLOBAL_LINEAR_ID => write!(f, "GlobalLinearId"),
            Self::SUBGROUP_SIZE => write!(f, "SubgroupSize"),
            Self::SUBGROUP_MAX_SIZE => write!(f, "SubgroupMaxSize"),
            Self::NUM_SUBGROUPS => write!(f, "NumSubgroups"),
            Self::NUM_ENQUEUED_SUBGROUPS => write!(f, "NumEnqueuedSubgroups"),
            Self::SUBGROUP_ID => write!(f, "SubgroupId"),
            Self::SUBGROUP_LOCAL_INVOCATION_ID => write!(f, "SubgroupLocalInvocationId"),
            Self::VERTEX_INDEX => write!(f, "VertexIndex"),
            Self::INSTANCE_INDEX => write!(f, "InstanceIndex"),
            Self::CORE_IDARM => write!(f, "CoreIDARM"),
            Self::CORE_COUNT_ARM => write!(f, "CoreCountARM"),
            Self::CORE_MAX_IDARM => write!(f, "CoreMaxIDARM"),
            Self::WARP_IDARM => write!(f, "WarpIDARM"),
            Self::WARP_MAX_IDARM => write!(f, "WarpMaxIDARM"),
            Self::SUBGROUP_EQ_MASK => write!(f, "SubgroupEqMask"),
            Self::SUBGROUP_GE_MASK => write!(f, "SubgroupGeMask"),
            Self::SUBGROUP_GT_MASK => write!(f, "SubgroupGtMask"),
            Self::SUBGROUP_LE_MASK => write!(f, "SubgroupLeMask"),
            Self::SUBGROUP_LT_MASK => write!(f, "SubgroupLtMask"),
            Self::BASE_VERTEX => write!(f, "BaseVertex"),
            Self::BASE_INSTANCE => write!(f, "BaseInstance"),
            Self::DRAW_INDEX => write!(f, "DrawIndex"),
            Self::PRIMITIVE_SHADING_RATE_KHR => write!(f, "PrimitiveShadingRateKHR"),
            Self::DEVICE_INDEX => write!(f, "DeviceIndex"),
            Self::VIEW_INDEX => write!(f, "ViewIndex"),
            Self::SHADING_RATE_KHR => write!(f, "ShadingRateKHR"),
            Self::TILE_OFFSET_QCOM => write!(f, "TileOffsetQCOM"),
            Self::TILE_DIMENSION_QCOM => write!(f, "TileDimensionQCOM"),
            Self::TILE_APRON_SIZE_QCOM => write!(f, "TileApronSizeQCOM"),
            Self::BARY_COORD_NO_PERSP_AMD => write!(f, "BaryCoordNoPerspAMD"),
            Self::BARY_COORD_NO_PERSP_CENTROID_AMD => write!(f, "BaryCoordNoPerspCentroidAMD"),
            Self::BARY_COORD_NO_PERSP_SAMPLE_AMD => write!(f, "BaryCoordNoPerspSampleAMD"),
            Self::BARY_COORD_SMOOTH_AMD => write!(f, "BaryCoordSmoothAMD"),
            Self::BARY_COORD_SMOOTH_CENTROID_AMD => write!(f, "BaryCoordSmoothCentroidAMD"),
            Self::BARY_COORD_SMOOTH_SAMPLE_AMD => write!(f, "BaryCoordSmoothSampleAMD"),
            Self::BARY_COORD_PULL_MODEL_AMD => write!(f, "BaryCoordPullModelAMD"),
            Self::FRAG_STENCIL_REF_EXT => write!(f, "FragStencilRefEXT"),
            Self::REMAINING_RECURSION_LEVELS_AMDX => write!(f, "RemainingRecursionLevelsAMDX"),
            Self::SHADER_INDEX_AMDX => write!(f, "ShaderIndexAMDX"),
            Self::SAMPLER_HEAP_EXT => write!(f, "SamplerHeapEXT"),
            Self::RESOURCE_HEAP_EXT => write!(f, "ResourceHeapEXT"),
            Self::VIEWPORT_MASK_NV => write!(f, "ViewportMaskNV"),
            Self::SECONDARY_POSITION_NV => write!(f, "SecondaryPositionNV"),
            Self::SECONDARY_VIEWPORT_MASK_NV => write!(f, "SecondaryViewportMaskNV"),
            Self::POSITION_PER_VIEW_NV => write!(f, "PositionPerViewNV"),
            Self::VIEWPORT_MASK_PER_VIEW_NV => write!(f, "ViewportMaskPerViewNV"),
            Self::FULLY_COVERED_EXT => write!(f, "FullyCoveredEXT"),
            Self::TASK_COUNT_NV => write!(f, "TaskCountNV"),
            Self::PRIMITIVE_COUNT_NV => write!(f, "PrimitiveCountNV"),
            Self::PRIMITIVE_INDICES_NV => write!(f, "PrimitiveIndicesNV"),
            Self::CLIP_DISTANCE_PER_VIEW_NV => write!(f, "ClipDistancePerViewNV"),
            Self::CULL_DISTANCE_PER_VIEW_NV => write!(f, "CullDistancePerViewNV"),
            Self::LAYER_PER_VIEW_NV => write!(f, "LayerPerViewNV"),
            Self::MESH_VIEW_COUNT_NV => write!(f, "MeshViewCountNV"),
            Self::MESH_VIEW_INDICES_NV => write!(f, "MeshViewIndicesNV"),
            Self::BARY_COORD_KHR => write!(f, "BaryCoordKHR"),
            Self::BARY_COORD_NO_PERSP_KHR => write!(f, "BaryCoordNoPerspKHR"),
            Self::FRAG_SIZE_EXT => write!(f, "FragSizeEXT"),
            Self::FRAG_INVOCATION_COUNT_EXT => write!(f, "FragInvocationCountEXT"),
            Self::PRIMITIVE_POINT_INDICES_EXT => write!(f, "PrimitivePointIndicesEXT"),
            Self::PRIMITIVE_LINE_INDICES_EXT => write!(f, "PrimitiveLineIndicesEXT"),
            Self::PRIMITIVE_TRIANGLE_INDICES_EXT => write!(f, "PrimitiveTriangleIndicesEXT"),
            Self::CULL_PRIMITIVE_EXT => write!(f, "CullPrimitiveEXT"),
            Self::LAUNCH_ID_KHR => write!(f, "LaunchIdKHR"),
            Self::LAUNCH_SIZE_KHR => write!(f, "LaunchSizeKHR"),
            Self::WORLD_RAY_ORIGIN_KHR => write!(f, "WorldRayOriginKHR"),
            Self::WORLD_RAY_DIRECTION_KHR => write!(f, "WorldRayDirectionKHR"),
            Self::OBJECT_RAY_ORIGIN_KHR => write!(f, "ObjectRayOriginKHR"),
            Self::OBJECT_RAY_DIRECTION_KHR => write!(f, "ObjectRayDirectionKHR"),
            Self::RAY_TMIN_KHR => write!(f, "RayTminKHR"),
            Self::RAY_TMAX_KHR => write!(f, "RayTmaxKHR"),
            Self::INSTANCE_CUSTOM_INDEX_KHR => write!(f, "InstanceCustomIndexKHR"),
            Self::OBJECT_TO_WORLD_KHR => write!(f, "ObjectToWorldKHR"),
            Self::WORLD_TO_OBJECT_KHR => write!(f, "WorldToObjectKHR"),
            Self::HIT_TNV => write!(f, "HitTNV"),
            Self::HIT_KIND_KHR => write!(f, "HitKindKHR"),
            Self::CURRENT_RAY_TIME_NV => write!(f, "CurrentRayTimeNV"),
            Self::HIT_TRIANGLE_VERTEX_POSITIONS_KHR => write!(f, "HitTriangleVertexPositionsKHR"),
            Self::HIT_MICRO_TRIANGLE_VERTEX_POSITIONS_NV => {
                write!(f, "HitMicroTriangleVertexPositionsNV")
            }
            Self::HIT_MICRO_TRIANGLE_VERTEX_BARYCENTRICS_NV => {
                write!(f, "HitMicroTriangleVertexBarycentricsNV")
            }
            Self::INCOMING_RAY_FLAGS_KHR => write!(f, "IncomingRayFlagsKHR"),
            Self::RAY_GEOMETRY_INDEX_KHR => write!(f, "RayGeometryIndexKHR"),
            Self::HIT_IS_SPHERE_NV => write!(f, "HitIsSphereNV"),
            Self::HIT_IS_LSSNV => write!(f, "HitIsLSSNV"),
            Self::HIT_SPHERE_POSITION_NV => write!(f, "HitSpherePositionNV"),
            Self::WARPS_PER_SMNV => write!(f, "WarpsPerSMNV"),
            Self::SMCOUNT_NV => write!(f, "SMCountNV"),
            Self::WARP_IDNV => write!(f, "WarpIDNV"),
            Self::SMIDNV => write!(f, "SMIDNV"),
            Self::HIT_LSSPOSITIONS_NV => write!(f, "HitLSSPositionsNV"),
            Self::HIT_KIND_FRONT_FACING_MICRO_TRIANGLE_NV => {
                write!(f, "HitKindFrontFacingMicroTriangleNV")
            }
            Self::HIT_KIND_BACK_FACING_MICRO_TRIANGLE_NV => {
                write!(f, "HitKindBackFacingMicroTriangleNV")
            }
            Self::HIT_SPHERE_RADIUS_NV => write!(f, "HitSphereRadiusNV"),
            Self::HIT_LSSRADII_NV => write!(f, "HitLSSRadiiNV"),
            Self::CLUSTER_IDNV => write!(f, "ClusterIDNV"),
            Self::CULL_MASK_KHR => write!(f, "CullMaskKHR"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Scope(pub(crate) u32);
impl Scope {
    pub const CROSS_DEVICE: Self = Self(0u32);
    pub const DEVICE: Self = Self(1u32);
    pub const WORKGROUP: Self = Self(2u32);
    pub const SUBGROUP: Self = Self(3u32);
    pub const INVOCATION: Self = Self(4u32);
    pub const QUEUE_FAMILY: Self = Self(5u32);
    pub const SHADER_CALL_KHR: Self = Self(6u32);
}
impl Word for Scope {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::CROSS_DEVICE => write!(f, "CrossDevice"),
            Self::DEVICE => write!(f, "Device"),
            Self::WORKGROUP => write!(f, "Workgroup"),
            Self::SUBGROUP => write!(f, "Subgroup"),
            Self::INVOCATION => write!(f, "Invocation"),
            Self::QUEUE_FAMILY => write!(f, "QueueFamily"),
            Self::SHADER_CALL_KHR => write!(f, "ShaderCallKHR"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GroupOperation(pub(crate) u32);
impl GroupOperation {
    pub const REDUCE: Self = Self(0u32);
    pub const INCLUSIVE_SCAN: Self = Self(1u32);
    pub const EXCLUSIVE_SCAN: Self = Self(2u32);
    pub const CLUSTERED_REDUCE: Self = Self(3u32);
    pub const PARTITIONED_REDUCE_EXT: Self = Self(6u32);
    pub const PARTITIONED_INCLUSIVE_SCAN_EXT: Self = Self(7u32);
    pub const PARTITIONED_EXCLUSIVE_SCAN_EXT: Self = Self(8u32);
}
impl Word for GroupOperation {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for GroupOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::REDUCE => write!(f, "Reduce"),
            Self::INCLUSIVE_SCAN => write!(f, "InclusiveScan"),
            Self::EXCLUSIVE_SCAN => write!(f, "ExclusiveScan"),
            Self::CLUSTERED_REDUCE => write!(f, "ClusteredReduce"),
            Self::PARTITIONED_REDUCE_EXT => write!(f, "PartitionedReduceEXT"),
            Self::PARTITIONED_INCLUSIVE_SCAN_EXT => write!(f, "PartitionedInclusiveScanEXT"),
            Self::PARTITIONED_EXCLUSIVE_SCAN_EXT => write!(f, "PartitionedExclusiveScanEXT"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KernelEnqueueFlags(pub(crate) u32);
impl KernelEnqueueFlags {
    pub const NO_WAIT: Self = Self(0u32);
    pub const WAIT_KERNEL: Self = Self(1u32);
    pub const WAIT_WORK_GROUP: Self = Self(2u32);
}
impl Word for KernelEnqueueFlags {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for KernelEnqueueFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NO_WAIT => write!(f, "NoWait"),
            Self::WAIT_KERNEL => write!(f, "WaitKernel"),
            Self::WAIT_WORK_GROUP => write!(f, "WaitWorkGroup"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Capability(pub(crate) u32);
impl Capability {
    pub const MATRIX: Self = Self(0u32);
    pub const SHADER: Self = Self(1u32);
    pub const GEOMETRY: Self = Self(2u32);
    pub const TESSELLATION: Self = Self(3u32);
    pub const ADDRESSES: Self = Self(4u32);
    pub const LINKAGE: Self = Self(5u32);
    pub const KERNEL: Self = Self(6u32);
    pub const VECTOR16: Self = Self(7u32);
    pub const FLOAT16_BUFFER: Self = Self(8u32);
    pub const FLOAT16: Self = Self(9u32);
    pub const FLOAT64: Self = Self(10u32);
    pub const INT64: Self = Self(11u32);
    pub const INT64_ATOMICS: Self = Self(12u32);
    pub const IMAGE_BASIC: Self = Self(13u32);
    pub const IMAGE_READ_WRITE: Self = Self(14u32);
    pub const IMAGE_MIPMAP: Self = Self(15u32);
    pub const PIPES: Self = Self(17u32);
    pub const GROUPS: Self = Self(18u32);
    pub const DEVICE_ENQUEUE: Self = Self(19u32);
    pub const LITERAL_SAMPLER: Self = Self(20u32);
    pub const ATOMIC_STORAGE: Self = Self(21u32);
    pub const INT16: Self = Self(22u32);
    pub const TESSELLATION_POINT_SIZE: Self = Self(23u32);
    pub const GEOMETRY_POINT_SIZE: Self = Self(24u32);
    pub const IMAGE_GATHER_EXTENDED: Self = Self(25u32);
    pub const STORAGE_IMAGE_MULTISAMPLE: Self = Self(27u32);
    pub const UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING: Self = Self(28u32);
    pub const SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING: Self = Self(29u32);
    pub const STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING: Self = Self(30u32);
    pub const STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING: Self = Self(31u32);
    pub const CLIP_DISTANCE: Self = Self(32u32);
    pub const CULL_DISTANCE: Self = Self(33u32);
    pub const IMAGE_CUBE_ARRAY: Self = Self(34u32);
    pub const SAMPLE_RATE_SHADING: Self = Self(35u32);
    pub const IMAGE_RECT: Self = Self(36u32);
    pub const SAMPLED_RECT: Self = Self(37u32);
    pub const GENERIC_POINTER: Self = Self(38u32);
    pub const INT8: Self = Self(39u32);
    pub const INPUT_ATTACHMENT: Self = Self(40u32);
    pub const SPARSE_RESIDENCY: Self = Self(41u32);
    pub const MIN_LOD: Self = Self(42u32);
    pub const SAMPLED1_D: Self = Self(43u32);
    pub const IMAGE1_D: Self = Self(44u32);
    pub const SAMPLED_CUBE_ARRAY: Self = Self(45u32);
    pub const SAMPLED_BUFFER: Self = Self(46u32);
    pub const IMAGE_BUFFER: Self = Self(47u32);
    pub const IMAGE_MSARRAY: Self = Self(48u32);
    pub const STORAGE_IMAGE_EXTENDED_FORMATS: Self = Self(49u32);
    pub const IMAGE_QUERY: Self = Self(50u32);
    pub const DERIVATIVE_CONTROL: Self = Self(51u32);
    pub const INTERPOLATION_FUNCTION: Self = Self(52u32);
    pub const TRANSFORM_FEEDBACK: Self = Self(53u32);
    pub const GEOMETRY_STREAMS: Self = Self(54u32);
    pub const STORAGE_IMAGE_READ_WITHOUT_FORMAT: Self = Self(55u32);
    pub const STORAGE_IMAGE_WRITE_WITHOUT_FORMAT: Self = Self(56u32);
    pub const MULTI_VIEWPORT: Self = Self(57u32);
    pub const SUBGROUP_DISPATCH: Self = Self(58u32);
    pub const NAMED_BARRIER: Self = Self(59u32);
    pub const PIPE_STORAGE: Self = Self(60u32);
    pub const GROUP_NON_UNIFORM: Self = Self(61u32);
    pub const GROUP_NON_UNIFORM_VOTE: Self = Self(62u32);
    pub const GROUP_NON_UNIFORM_ARITHMETIC: Self = Self(63u32);
    pub const GROUP_NON_UNIFORM_BALLOT: Self = Self(64u32);
    pub const GROUP_NON_UNIFORM_SHUFFLE: Self = Self(65u32);
    pub const GROUP_NON_UNIFORM_SHUFFLE_RELATIVE: Self = Self(66u32);
    pub const GROUP_NON_UNIFORM_CLUSTERED: Self = Self(67u32);
    pub const GROUP_NON_UNIFORM_QUAD: Self = Self(68u32);
    pub const SHADER_LAYER: Self = Self(69u32);
    pub const SHADER_VIEWPORT_INDEX: Self = Self(70u32);
    pub const UNIFORM_DECORATION: Self = Self(71u32);
    pub const CORE_BUILTINS_ARM: Self = Self(4165u32);
    pub const TILE_IMAGE_COLOR_READ_ACCESS_EXT: Self = Self(4166u32);
    pub const TILE_IMAGE_DEPTH_READ_ACCESS_EXT: Self = Self(4167u32);
    pub const TILE_IMAGE_STENCIL_READ_ACCESS_EXT: Self = Self(4168u32);
    pub const TENSORS_ARM: Self = Self(4174u32);
    pub const STORAGE_TENSOR_ARRAY_DYNAMIC_INDEXING_ARM: Self = Self(4175u32);
    pub const STORAGE_TENSOR_ARRAY_NON_UNIFORM_INDEXING_ARM: Self = Self(4176u32);
    pub const GRAPH_ARM: Self = Self(4191u32);
    pub const COOPERATIVE_MATRIX_LAYOUTS_ARM: Self = Self(4201u32);
    pub const FLOAT8_EXT: Self = Self(4212u32);
    pub const FLOAT8_COOPERATIVE_MATRIX_EXT: Self = Self(4213u32);
    pub const FRAGMENT_SHADING_RATE_KHR: Self = Self(4422u32);
    pub const SUBGROUP_BALLOT_KHR: Self = Self(4423u32);
    pub const DRAW_PARAMETERS: Self = Self(4427u32);
    pub const WORKGROUP_MEMORY_EXPLICIT_LAYOUT_KHR: Self = Self(4428u32);
    pub const WORKGROUP_MEMORY_EXPLICIT_LAYOUT8_BIT_ACCESS_KHR: Self = Self(4429u32);
    pub const WORKGROUP_MEMORY_EXPLICIT_LAYOUT16_BIT_ACCESS_KHR: Self = Self(4430u32);
    pub const SUBGROUP_VOTE_KHR: Self = Self(4431u32);
    pub const STORAGE_BUFFER16_BIT_ACCESS: Self = Self(4433u32);
    pub const UNIFORM_AND_STORAGE_BUFFER16_BIT_ACCESS: Self = Self(4434u32);
    pub const STORAGE_PUSH_CONSTANT16: Self = Self(4435u32);
    pub const STORAGE_INPUT_OUTPUT16: Self = Self(4436u32);
    pub const DEVICE_GROUP: Self = Self(4437u32);
    pub const MULTI_VIEW: Self = Self(4439u32);
    pub const VARIABLE_POINTERS_STORAGE_BUFFER: Self = Self(4441u32);
    pub const VARIABLE_POINTERS: Self = Self(4442u32);
    pub const ATOMIC_STORAGE_OPS: Self = Self(4445u32);
    pub const SAMPLE_MASK_POST_DEPTH_COVERAGE: Self = Self(4447u32);
    pub const STORAGE_BUFFER8_BIT_ACCESS: Self = Self(4448u32);
    pub const UNIFORM_AND_STORAGE_BUFFER8_BIT_ACCESS: Self = Self(4449u32);
    pub const STORAGE_PUSH_CONSTANT8: Self = Self(4450u32);
    pub const DENORM_PRESERVE: Self = Self(4464u32);
    pub const DENORM_FLUSH_TO_ZERO: Self = Self(4465u32);
    pub const SIGNED_ZERO_INF_NAN_PRESERVE: Self = Self(4466u32);
    pub const ROUNDING_MODE_RTE: Self = Self(4467u32);
    pub const ROUNDING_MODE_RTZ: Self = Self(4468u32);
    pub const RAY_QUERY_PROVISIONAL_KHR: Self = Self(4471u32);
    pub const RAY_QUERY_KHR: Self = Self(4472u32);
    pub const UNTYPED_POINTERS_KHR: Self = Self(4473u32);
    pub const RAY_TRAVERSAL_PRIMITIVE_CULLING_KHR: Self = Self(4478u32);
    pub const RAY_TRACING_KHR: Self = Self(4479u32);
    pub const TEXTURE_SAMPLE_WEIGHTED_QCOM: Self = Self(4484u32);
    pub const TEXTURE_BOX_FILTER_QCOM: Self = Self(4485u32);
    pub const TEXTURE_BLOCK_MATCH_QCOM: Self = Self(4486u32);
    pub const TILE_SHADING_QCOM: Self = Self(4495u32);
    pub const COOPERATIVE_MATRIX_CONVERSION_QCOM: Self = Self(4496u32);
    pub const TEXTURE_BLOCK_MATCH2_QCOM: Self = Self(4498u32);
    pub const FLOAT16_IMAGE_AMD: Self = Self(5008u32);
    pub const IMAGE_GATHER_BIAS_LOD_AMD: Self = Self(5009u32);
    pub const FRAGMENT_MASK_AMD: Self = Self(5010u32);
    pub const STENCIL_EXPORT_EXT: Self = Self(5013u32);
    pub const IMAGE_READ_WRITE_LOD_AMD: Self = Self(5015u32);
    pub const INT64_IMAGE_EXT: Self = Self(5016u32);
    pub const SHADER_CLOCK_KHR: Self = Self(5055u32);
    pub const SHADER_ENQUEUE_AMDX: Self = Self(5067u32);
    pub const QUAD_CONTROL_KHR: Self = Self(5087u32);
    pub const INT4_TYPE_INTEL: Self = Self(5112u32);
    pub const INT4_COOPERATIVE_MATRIX_INTEL: Self = Self(5114u32);
    pub const BFLOAT16_TYPE_KHR: Self = Self(5116u32);
    pub const BFLOAT16_DOT_PRODUCT_KHR: Self = Self(5117u32);
    pub const BFLOAT16_COOPERATIVE_MATRIX_KHR: Self = Self(5118u32);
    pub const ABORT_KHR: Self = Self(5120u32);
    pub const DESCRIPTOR_HEAP_EXT: Self = Self(5128u32);
    pub const CONSTANT_DATA_KHR: Self = Self(5146u32);
    pub const POISON_FREEZE_KHR: Self = Self(5156u32);
    pub const SAMPLE_MASK_OVERRIDE_COVERAGE_NV: Self = Self(5249u32);
    pub const GEOMETRY_SHADER_PASSTHROUGH_NV: Self = Self(5251u32);
    pub const SHADER_VIEWPORT_INDEX_LAYER_EXT: Self = Self(5254u32);
    pub const SHADER_VIEWPORT_MASK_NV: Self = Self(5255u32);
    pub const SHADER_STEREO_VIEW_NV: Self = Self(5259u32);
    pub const PER_VIEW_ATTRIBUTES_NV: Self = Self(5260u32);
    pub const FRAGMENT_FULLY_COVERED_EXT: Self = Self(5265u32);
    pub const MESH_SHADING_NV: Self = Self(5266u32);
    pub const IMAGE_FOOTPRINT_NV: Self = Self(5282u32);
    pub const MESH_SHADING_EXT: Self = Self(5283u32);
    pub const FRAGMENT_BARYCENTRIC_KHR: Self = Self(5284u32);
    pub const COMPUTE_DERIVATIVE_GROUP_QUADS_KHR: Self = Self(5288u32);
    pub const FRAGMENT_DENSITY_EXT: Self = Self(5291u32);
    pub const GROUP_NON_UNIFORM_PARTITIONED_EXT: Self = Self(5297u32);
    pub const SHADER_NON_UNIFORM: Self = Self(5301u32);
    pub const RUNTIME_DESCRIPTOR_ARRAY: Self = Self(5302u32);
    pub const INPUT_ATTACHMENT_ARRAY_DYNAMIC_INDEXING: Self = Self(5303u32);
    pub const UNIFORM_TEXEL_BUFFER_ARRAY_DYNAMIC_INDEXING: Self = Self(5304u32);
    pub const STORAGE_TEXEL_BUFFER_ARRAY_DYNAMIC_INDEXING: Self = Self(5305u32);
    pub const UNIFORM_BUFFER_ARRAY_NON_UNIFORM_INDEXING: Self = Self(5306u32);
    pub const SAMPLED_IMAGE_ARRAY_NON_UNIFORM_INDEXING: Self = Self(5307u32);
    pub const STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING: Self = Self(5308u32);
    pub const STORAGE_IMAGE_ARRAY_NON_UNIFORM_INDEXING: Self = Self(5309u32);
    pub const INPUT_ATTACHMENT_ARRAY_NON_UNIFORM_INDEXING: Self = Self(5310u32);
    pub const UNIFORM_TEXEL_BUFFER_ARRAY_NON_UNIFORM_INDEXING: Self = Self(5311u32);
    pub const STORAGE_TEXEL_BUFFER_ARRAY_NON_UNIFORM_INDEXING: Self = Self(5312u32);
    pub const RAY_TRACING_POSITION_FETCH_KHR: Self = Self(5336u32);
    pub const RAY_TRACING_NV: Self = Self(5340u32);
    pub const RAY_TRACING_MOTION_BLUR_NV: Self = Self(5341u32);
    pub const VULKAN_MEMORY_MODEL: Self = Self(5345u32);
    pub const VULKAN_MEMORY_MODEL_DEVICE_SCOPE: Self = Self(5346u32);
    pub const PHYSICAL_STORAGE_BUFFER_ADDRESSES: Self = Self(5347u32);
    pub const COMPUTE_DERIVATIVE_GROUP_LINEAR_KHR: Self = Self(5350u32);
    pub const RAY_TRACING_PROVISIONAL_KHR: Self = Self(5353u32);
    pub const COOPERATIVE_MATRIX_NV: Self = Self(5357u32);
    pub const FRAGMENT_SHADER_SAMPLE_INTERLOCK_EXT: Self = Self(5363u32);
    pub const FRAGMENT_SHADER_SHADING_RATE_INTERLOCK_EXT: Self = Self(5372u32);
    pub const SHADER_SMBUILTINS_NV: Self = Self(5373u32);
    pub const FRAGMENT_SHADER_PIXEL_INTERLOCK_EXT: Self = Self(5378u32);
    pub const DEMOTE_TO_HELPER_INVOCATION: Self = Self(5379u32);
    pub const DISPLACEMENT_MICROMAP_NV: Self = Self(5380u32);
    pub const RAY_TRACING_OPACITY_MICROMAP_EXT: Self = Self(5381u32);
    pub const SHADER_INVOCATION_REORDER_NV: Self = Self(5383u32);
    pub const SHADER_INVOCATION_REORDER_EXT: Self = Self(5388u32);
    pub const BINDLESS_TEXTURE_NV: Self = Self(5390u32);
    pub const RAY_QUERY_POSITION_FETCH_KHR: Self = Self(5391u32);
    pub const COOPERATIVE_VECTOR_NV: Self = Self(5394u32);
    pub const ATOMIC_FLOAT16_VECTOR_NV: Self = Self(5404u32);
    pub const RAY_TRACING_DISPLACEMENT_MICROMAP_NV: Self = Self(5409u32);
    pub const RAW_ACCESS_CHAINS_NV: Self = Self(5414u32);
    pub const RAY_TRACING_SPHERES_GEOMETRY_NV: Self = Self(5418u32);
    pub const RAY_TRACING_LINEAR_SWEPT_SPHERES_GEOMETRY_NV: Self = Self(5419u32);
    pub const PUSH_CONSTANT_BANKS_NV: Self = Self(5423u32);
    pub const LONG_VECTOR_EXT: Self = Self(5425u32);
    pub const SHADER64_BIT_INDEXING_EXT: Self = Self(5426u32);
    pub const COOPERATIVE_MATRIX_REDUCTIONS_NV: Self = Self(5430u32);
    pub const COOPERATIVE_MATRIX_CONVERSIONS_NV: Self = Self(5431u32);
    pub const COOPERATIVE_MATRIX_PER_ELEMENT_OPERATIONS_NV: Self = Self(5432u32);
    pub const COOPERATIVE_MATRIX_TENSOR_ADDRESSING_NV: Self = Self(5433u32);
    pub const COOPERATIVE_MATRIX_BLOCK_LOADS_NV: Self = Self(5434u32);
    pub const COOPERATIVE_VECTOR_TRAINING_NV: Self = Self(5435u32);
    pub const RAY_TRACING_CLUSTER_ACCELERATION_STRUCTURE_NV: Self = Self(5437u32);
    pub const TENSOR_ADDRESSING_NV: Self = Self(5439u32);
    pub const SUBGROUP_SHUFFLE_INTEL: Self = Self(5568u32);
    pub const SUBGROUP_BUFFER_BLOCK_IOINTEL: Self = Self(5569u32);
    pub const SUBGROUP_IMAGE_BLOCK_IOINTEL: Self = Self(5570u32);
    pub const SUBGROUP_IMAGE_MEDIA_BLOCK_IOINTEL: Self = Self(5579u32);
    pub const ROUND_TO_INFINITY_INTEL: Self = Self(5582u32);
    pub const FLOATING_POINT_MODE_INTEL: Self = Self(5583u32);
    pub const INTEGER_FUNCTIONS2_INTEL: Self = Self(5584u32);
    pub const FUNCTION_POINTERS_INTEL: Self = Self(5603u32);
    pub const INDIRECT_REFERENCES_INTEL: Self = Self(5604u32);
    pub const ASM_INTEL: Self = Self(5606u32);
    pub const ATOMIC_FLOAT32_MIN_MAX_EXT: Self = Self(5612u32);
    pub const ATOMIC_FLOAT64_MIN_MAX_EXT: Self = Self(5613u32);
    pub const ATOMIC_FLOAT16_MIN_MAX_EXT: Self = Self(5616u32);
    pub const VECTOR_COMPUTE_INTEL: Self = Self(5617u32);
    pub const VECTOR_ANY_INTEL: Self = Self(5619u32);
    pub const EXPECT_ASSUME_KHR: Self = Self(5629u32);
    pub const SUBGROUP_AVC_MOTION_ESTIMATION_INTEL: Self = Self(5696u32);
    pub const SUBGROUP_AVC_MOTION_ESTIMATION_INTRA_INTEL: Self = Self(5697u32);
    pub const SUBGROUP_AVC_MOTION_ESTIMATION_CHROMA_INTEL: Self = Self(5698u32);
    pub const VARIABLE_LENGTH_ARRAY_INTEL: Self = Self(5817u32);
    pub const FUNCTION_FLOAT_CONTROL_INTEL: Self = Self(5821u32);
    pub const FPGAMEMORY_ATTRIBUTES_ALTERA: Self = Self(5824u32);
    pub const FPFAST_MATH_MODE_INTEL: Self = Self(5837u32);
    pub const ARBITRARY_PRECISION_INTEGERS_ALTERA: Self = Self(5844u32);
    pub const ARBITRARY_PRECISION_FLOATING_POINT_ALTERA: Self = Self(5845u32);
    pub const UNSTRUCTURED_LOOP_CONTROLS_INTEL: Self = Self(5886u32);
    pub const FPGALOOP_CONTROLS_ALTERA: Self = Self(5888u32);
    pub const KERNEL_ATTRIBUTES_INTEL: Self = Self(5892u32);
    pub const FPGAKERNEL_ATTRIBUTES_INTEL: Self = Self(5897u32);
    pub const FPGAMEMORY_ACCESSES_ALTERA: Self = Self(5898u32);
    pub const FPGACLUSTER_ATTRIBUTES_ALTERA: Self = Self(5904u32);
    pub const LOOP_FUSE_ALTERA: Self = Self(5906u32);
    pub const FPGADSPCONTROL_ALTERA: Self = Self(5908u32);
    pub const MEMORY_ACCESS_ALIASING_INTEL: Self = Self(5910u32);
    pub const FPGAINVOCATION_PIPELINING_ATTRIBUTES_ALTERA: Self = Self(5916u32);
    pub const FPGABUFFER_LOCATION_ALTERA: Self = Self(5920u32);
    pub const ARBITRARY_PRECISION_FIXED_POINT_ALTERA: Self = Self(5922u32);
    pub const USMSTORAGE_CLASSES_ALTERA: Self = Self(5935u32);
    pub const RUNTIME_ALIGNED_ATTRIBUTE_ALTERA: Self = Self(5939u32);
    pub const IOPIPES_ALTERA: Self = Self(5943u32);
    pub const BLOCKING_PIPES_ALTERA: Self = Self(5945u32);
    pub const FPGAREG_ALTERA: Self = Self(5948u32);
    pub const DOT_PRODUCT_INPUT_ALL: Self = Self(6016u32);
    pub const DOT_PRODUCT_INPUT4X8_BIT: Self = Self(6017u32);
    pub const DOT_PRODUCT_INPUT4X8_BIT_PACKED: Self = Self(6018u32);
    pub const DOT_PRODUCT: Self = Self(6019u32);
    pub const RAY_CULL_MASK_KHR: Self = Self(6020u32);
    pub const COOPERATIVE_MATRIX_KHR: Self = Self(6022u32);
    pub const REPLICATED_COMPOSITES_EXT: Self = Self(6024u32);
    pub const BIT_INSTRUCTIONS: Self = Self(6025u32);
    pub const GROUP_NON_UNIFORM_ROTATE_KHR: Self = Self(6026u32);
    pub const FLOAT_CONTROLS2: Self = Self(6029u32);
    pub const FMAKHR: Self = Self(6030u32);
    pub const ATOMIC_FLOAT32_ADD_EXT: Self = Self(6033u32);
    pub const ATOMIC_FLOAT64_ADD_EXT: Self = Self(6034u32);
    pub const LONG_COMPOSITES_INTEL: Self = Self(6089u32);
    pub const OPT_NONE_EXT: Self = Self(6094u32);
    pub const ATOMIC_FLOAT16_ADD_EXT: Self = Self(6095u32);
    pub const DEBUG_INFO_MODULE_INTEL: Self = Self(6114u32);
    pub const BFLOAT16_CONVERSION_INTEL: Self = Self(6115u32);
    pub const SPLIT_BARRIER_INTEL: Self = Self(6141u32);
    pub const ARITHMETIC_FENCE_EXT: Self = Self(6144u32);
    pub const FPGACLUSTER_ATTRIBUTES_V2_ALTERA: Self = Self(6150u32);
    pub const FPGAKERNEL_ATTRIBUTESV2_INTEL: Self = Self(6161u32);
    pub const TASK_SEQUENCE_ALTERA: Self = Self(6162u32);
    pub const FPMAX_ERROR_INTEL: Self = Self(6169u32);
    pub const FPGALATENCY_CONTROL_ALTERA: Self = Self(6171u32);
    pub const FPGAARGUMENT_INTERFACES_ALTERA: Self = Self(6174u32);
    pub const GLOBAL_VARIABLE_HOST_ACCESS_INTEL: Self = Self(6187u32);
    pub const GLOBAL_VARIABLE_FPGADECORATIONS_ALTERA: Self = Self(6189u32);
    pub const SUBGROUP_BUFFER_PREFETCH_INTEL: Self = Self(6220u32);
    pub const SUBGROUP2_DBLOCK_IOINTEL: Self = Self(6228u32);
    pub const SUBGROUP2_DBLOCK_TRANSFORM_INTEL: Self = Self(6229u32);
    pub const SUBGROUP2_DBLOCK_TRANSPOSE_INTEL: Self = Self(6230u32);
    pub const SUBGROUP_MATRIX_MULTIPLY_ACCUMULATE_INTEL: Self = Self(6236u32);
    pub const TERNARY_BITWISE_FUNCTION_INTEL: Self = Self(6241u32);
    pub const UNTYPED_VARIABLE_LENGTH_ARRAY_INTEL: Self = Self(6243u32);
    pub const SPEC_CONDITIONAL_INTEL: Self = Self(6245u32);
    pub const FUNCTION_VARIANTS_INTEL: Self = Self(6246u32);
    pub const GROUP_UNIFORM_ARITHMETIC_KHR: Self = Self(6400u32);
    pub const TENSOR_FLOAT32_ROUNDING_INTEL: Self = Self(6425u32);
    pub const MASKED_GATHER_SCATTER_INTEL: Self = Self(6427u32);
    pub const CACHE_CONTROLS_INTEL: Self = Self(6441u32);
    pub const REGISTER_LIMITS_INTEL: Self = Self(6460u32);
    pub const BINDLESS_IMAGES_INTEL: Self = Self(6528u32);
    pub const DOT_PRODUCT_FLOAT16_ACC_FLOAT32_VALVE: Self = Self(6912u32);
    pub const DOT_PRODUCT_FLOAT16_ACC_FLOAT16_VALVE: Self = Self(6913u32);
    pub const DOT_PRODUCT_BFLOAT16_ACC_VALVE: Self = Self(6914u32);
    pub const DOT_PRODUCT_FLOAT8_ACC_FLOAT32_VALVE: Self = Self(6915u32);
}
impl Word for Capability {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::MATRIX => write!(f, "Matrix"),
            Self::SHADER => write!(f, "Shader"),
            Self::GEOMETRY => write!(f, "Geometry"),
            Self::TESSELLATION => write!(f, "Tessellation"),
            Self::ADDRESSES => write!(f, "Addresses"),
            Self::LINKAGE => write!(f, "Linkage"),
            Self::KERNEL => write!(f, "Kernel"),
            Self::VECTOR16 => write!(f, "Vector16"),
            Self::FLOAT16_BUFFER => write!(f, "Float16Buffer"),
            Self::FLOAT16 => write!(f, "Float16"),
            Self::FLOAT64 => write!(f, "Float64"),
            Self::INT64 => write!(f, "Int64"),
            Self::INT64_ATOMICS => write!(f, "Int64Atomics"),
            Self::IMAGE_BASIC => write!(f, "ImageBasic"),
            Self::IMAGE_READ_WRITE => write!(f, "ImageReadWrite"),
            Self::IMAGE_MIPMAP => write!(f, "ImageMipmap"),
            Self::PIPES => write!(f, "Pipes"),
            Self::GROUPS => write!(f, "Groups"),
            Self::DEVICE_ENQUEUE => write!(f, "DeviceEnqueue"),
            Self::LITERAL_SAMPLER => write!(f, "LiteralSampler"),
            Self::ATOMIC_STORAGE => write!(f, "AtomicStorage"),
            Self::INT16 => write!(f, "Int16"),
            Self::TESSELLATION_POINT_SIZE => write!(f, "TessellationPointSize"),
            Self::GEOMETRY_POINT_SIZE => write!(f, "GeometryPointSize"),
            Self::IMAGE_GATHER_EXTENDED => write!(f, "ImageGatherExtended"),
            Self::STORAGE_IMAGE_MULTISAMPLE => write!(f, "StorageImageMultisample"),
            Self::UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING => {
                write!(f, "UniformBufferArrayDynamicIndexing")
            }
            Self::SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING => {
                write!(f, "SampledImageArrayDynamicIndexing")
            }
            Self::STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING => {
                write!(f, "StorageBufferArrayDynamicIndexing")
            }
            Self::STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING => {
                write!(f, "StorageImageArrayDynamicIndexing")
            }
            Self::CLIP_DISTANCE => write!(f, "ClipDistance"),
            Self::CULL_DISTANCE => write!(f, "CullDistance"),
            Self::IMAGE_CUBE_ARRAY => write!(f, "ImageCubeArray"),
            Self::SAMPLE_RATE_SHADING => write!(f, "SampleRateShading"),
            Self::IMAGE_RECT => write!(f, "ImageRect"),
            Self::SAMPLED_RECT => write!(f, "SampledRect"),
            Self::GENERIC_POINTER => write!(f, "GenericPointer"),
            Self::INT8 => write!(f, "Int8"),
            Self::INPUT_ATTACHMENT => write!(f, "InputAttachment"),
            Self::SPARSE_RESIDENCY => write!(f, "SparseResidency"),
            Self::MIN_LOD => write!(f, "MinLod"),
            Self::SAMPLED1_D => write!(f, "Sampled1D"),
            Self::IMAGE1_D => write!(f, "Image1D"),
            Self::SAMPLED_CUBE_ARRAY => write!(f, "SampledCubeArray"),
            Self::SAMPLED_BUFFER => write!(f, "SampledBuffer"),
            Self::IMAGE_BUFFER => write!(f, "ImageBuffer"),
            Self::IMAGE_MSARRAY => write!(f, "ImageMSArray"),
            Self::STORAGE_IMAGE_EXTENDED_FORMATS => write!(f, "StorageImageExtendedFormats"),
            Self::IMAGE_QUERY => write!(f, "ImageQuery"),
            Self::DERIVATIVE_CONTROL => write!(f, "DerivativeControl"),
            Self::INTERPOLATION_FUNCTION => write!(f, "InterpolationFunction"),
            Self::TRANSFORM_FEEDBACK => write!(f, "TransformFeedback"),
            Self::GEOMETRY_STREAMS => write!(f, "GeometryStreams"),
            Self::STORAGE_IMAGE_READ_WITHOUT_FORMAT => write!(f, "StorageImageReadWithoutFormat"),
            Self::STORAGE_IMAGE_WRITE_WITHOUT_FORMAT => write!(f, "StorageImageWriteWithoutFormat"),
            Self::MULTI_VIEWPORT => write!(f, "MultiViewport"),
            Self::SUBGROUP_DISPATCH => write!(f, "SubgroupDispatch"),
            Self::NAMED_BARRIER => write!(f, "NamedBarrier"),
            Self::PIPE_STORAGE => write!(f, "PipeStorage"),
            Self::GROUP_NON_UNIFORM => write!(f, "GroupNonUniform"),
            Self::GROUP_NON_UNIFORM_VOTE => write!(f, "GroupNonUniformVote"),
            Self::GROUP_NON_UNIFORM_ARITHMETIC => write!(f, "GroupNonUniformArithmetic"),
            Self::GROUP_NON_UNIFORM_BALLOT => write!(f, "GroupNonUniformBallot"),
            Self::GROUP_NON_UNIFORM_SHUFFLE => write!(f, "GroupNonUniformShuffle"),
            Self::GROUP_NON_UNIFORM_SHUFFLE_RELATIVE => write!(f, "GroupNonUniformShuffleRelative"),
            Self::GROUP_NON_UNIFORM_CLUSTERED => write!(f, "GroupNonUniformClustered"),
            Self::GROUP_NON_UNIFORM_QUAD => write!(f, "GroupNonUniformQuad"),
            Self::SHADER_LAYER => write!(f, "ShaderLayer"),
            Self::SHADER_VIEWPORT_INDEX => write!(f, "ShaderViewportIndex"),
            Self::UNIFORM_DECORATION => write!(f, "UniformDecoration"),
            Self::CORE_BUILTINS_ARM => write!(f, "CoreBuiltinsARM"),
            Self::TILE_IMAGE_COLOR_READ_ACCESS_EXT => write!(f, "TileImageColorReadAccessEXT"),
            Self::TILE_IMAGE_DEPTH_READ_ACCESS_EXT => write!(f, "TileImageDepthReadAccessEXT"),
            Self::TILE_IMAGE_STENCIL_READ_ACCESS_EXT => write!(f, "TileImageStencilReadAccessEXT"),
            Self::TENSORS_ARM => write!(f, "TensorsARM"),
            Self::STORAGE_TENSOR_ARRAY_DYNAMIC_INDEXING_ARM => {
                write!(f, "StorageTensorArrayDynamicIndexingARM")
            }
            Self::STORAGE_TENSOR_ARRAY_NON_UNIFORM_INDEXING_ARM => {
                write!(f, "StorageTensorArrayNonUniformIndexingARM")
            }
            Self::GRAPH_ARM => write!(f, "GraphARM"),
            Self::COOPERATIVE_MATRIX_LAYOUTS_ARM => write!(f, "CooperativeMatrixLayoutsARM"),
            Self::FLOAT8_EXT => write!(f, "Float8EXT"),
            Self::FLOAT8_COOPERATIVE_MATRIX_EXT => write!(f, "Float8CooperativeMatrixEXT"),
            Self::FRAGMENT_SHADING_RATE_KHR => write!(f, "FragmentShadingRateKHR"),
            Self::SUBGROUP_BALLOT_KHR => write!(f, "SubgroupBallotKHR"),
            Self::DRAW_PARAMETERS => write!(f, "DrawParameters"),
            Self::WORKGROUP_MEMORY_EXPLICIT_LAYOUT_KHR => {
                write!(f, "WorkgroupMemoryExplicitLayoutKHR")
            }
            Self::WORKGROUP_MEMORY_EXPLICIT_LAYOUT8_BIT_ACCESS_KHR => {
                write!(f, "WorkgroupMemoryExplicitLayout8BitAccessKHR")
            }
            Self::WORKGROUP_MEMORY_EXPLICIT_LAYOUT16_BIT_ACCESS_KHR => {
                write!(f, "WorkgroupMemoryExplicitLayout16BitAccessKHR")
            }
            Self::SUBGROUP_VOTE_KHR => write!(f, "SubgroupVoteKHR"),
            Self::STORAGE_BUFFER16_BIT_ACCESS => write!(f, "StorageBuffer16BitAccess"),
            Self::UNIFORM_AND_STORAGE_BUFFER16_BIT_ACCESS => {
                write!(f, "UniformAndStorageBuffer16BitAccess")
            }
            Self::STORAGE_PUSH_CONSTANT16 => write!(f, "StoragePushConstant16"),
            Self::STORAGE_INPUT_OUTPUT16 => write!(f, "StorageInputOutput16"),
            Self::DEVICE_GROUP => write!(f, "DeviceGroup"),
            Self::MULTI_VIEW => write!(f, "MultiView"),
            Self::VARIABLE_POINTERS_STORAGE_BUFFER => write!(f, "VariablePointersStorageBuffer"),
            Self::VARIABLE_POINTERS => write!(f, "VariablePointers"),
            Self::ATOMIC_STORAGE_OPS => write!(f, "AtomicStorageOps"),
            Self::SAMPLE_MASK_POST_DEPTH_COVERAGE => write!(f, "SampleMaskPostDepthCoverage"),
            Self::STORAGE_BUFFER8_BIT_ACCESS => write!(f, "StorageBuffer8BitAccess"),
            Self::UNIFORM_AND_STORAGE_BUFFER8_BIT_ACCESS => {
                write!(f, "UniformAndStorageBuffer8BitAccess")
            }
            Self::STORAGE_PUSH_CONSTANT8 => write!(f, "StoragePushConstant8"),
            Self::DENORM_PRESERVE => write!(f, "DenormPreserve"),
            Self::DENORM_FLUSH_TO_ZERO => write!(f, "DenormFlushToZero"),
            Self::SIGNED_ZERO_INF_NAN_PRESERVE => write!(f, "SignedZeroInfNanPreserve"),
            Self::ROUNDING_MODE_RTE => write!(f, "RoundingModeRTE"),
            Self::ROUNDING_MODE_RTZ => write!(f, "RoundingModeRTZ"),
            Self::RAY_QUERY_PROVISIONAL_KHR => write!(f, "RayQueryProvisionalKHR"),
            Self::RAY_QUERY_KHR => write!(f, "RayQueryKHR"),
            Self::UNTYPED_POINTERS_KHR => write!(f, "UntypedPointersKHR"),
            Self::RAY_TRAVERSAL_PRIMITIVE_CULLING_KHR => {
                write!(f, "RayTraversalPrimitiveCullingKHR")
            }
            Self::RAY_TRACING_KHR => write!(f, "RayTracingKHR"),
            Self::TEXTURE_SAMPLE_WEIGHTED_QCOM => write!(f, "TextureSampleWeightedQCOM"),
            Self::TEXTURE_BOX_FILTER_QCOM => write!(f, "TextureBoxFilterQCOM"),
            Self::TEXTURE_BLOCK_MATCH_QCOM => write!(f, "TextureBlockMatchQCOM"),
            Self::TILE_SHADING_QCOM => write!(f, "TileShadingQCOM"),
            Self::COOPERATIVE_MATRIX_CONVERSION_QCOM => {
                write!(f, "CooperativeMatrixConversionQCOM")
            }
            Self::TEXTURE_BLOCK_MATCH2_QCOM => write!(f, "TextureBlockMatch2QCOM"),
            Self::FLOAT16_IMAGE_AMD => write!(f, "Float16ImageAMD"),
            Self::IMAGE_GATHER_BIAS_LOD_AMD => write!(f, "ImageGatherBiasLodAMD"),
            Self::FRAGMENT_MASK_AMD => write!(f, "FragmentMaskAMD"),
            Self::STENCIL_EXPORT_EXT => write!(f, "StencilExportEXT"),
            Self::IMAGE_READ_WRITE_LOD_AMD => write!(f, "ImageReadWriteLodAMD"),
            Self::INT64_IMAGE_EXT => write!(f, "Int64ImageEXT"),
            Self::SHADER_CLOCK_KHR => write!(f, "ShaderClockKHR"),
            Self::SHADER_ENQUEUE_AMDX => write!(f, "ShaderEnqueueAMDX"),
            Self::QUAD_CONTROL_KHR => write!(f, "QuadControlKHR"),
            Self::INT4_TYPE_INTEL => write!(f, "Int4TypeINTEL"),
            Self::INT4_COOPERATIVE_MATRIX_INTEL => write!(f, "Int4CooperativeMatrixINTEL"),
            Self::BFLOAT16_TYPE_KHR => write!(f, "BFloat16TypeKHR"),
            Self::BFLOAT16_DOT_PRODUCT_KHR => write!(f, "BFloat16DotProductKHR"),
            Self::BFLOAT16_COOPERATIVE_MATRIX_KHR => write!(f, "BFloat16CooperativeMatrixKHR"),
            Self::ABORT_KHR => write!(f, "AbortKHR"),
            Self::DESCRIPTOR_HEAP_EXT => write!(f, "DescriptorHeapEXT"),
            Self::CONSTANT_DATA_KHR => write!(f, "ConstantDataKHR"),
            Self::POISON_FREEZE_KHR => write!(f, "PoisonFreezeKHR"),
            Self::SAMPLE_MASK_OVERRIDE_COVERAGE_NV => write!(f, "SampleMaskOverrideCoverageNV"),
            Self::GEOMETRY_SHADER_PASSTHROUGH_NV => write!(f, "GeometryShaderPassthroughNV"),
            Self::SHADER_VIEWPORT_INDEX_LAYER_EXT => write!(f, "ShaderViewportIndexLayerEXT"),
            Self::SHADER_VIEWPORT_MASK_NV => write!(f, "ShaderViewportMaskNV"),
            Self::SHADER_STEREO_VIEW_NV => write!(f, "ShaderStereoViewNV"),
            Self::PER_VIEW_ATTRIBUTES_NV => write!(f, "PerViewAttributesNV"),
            Self::FRAGMENT_FULLY_COVERED_EXT => write!(f, "FragmentFullyCoveredEXT"),
            Self::MESH_SHADING_NV => write!(f, "MeshShadingNV"),
            Self::IMAGE_FOOTPRINT_NV => write!(f, "ImageFootprintNV"),
            Self::MESH_SHADING_EXT => write!(f, "MeshShadingEXT"),
            Self::FRAGMENT_BARYCENTRIC_KHR => write!(f, "FragmentBarycentricKHR"),
            Self::COMPUTE_DERIVATIVE_GROUP_QUADS_KHR => write!(f, "ComputeDerivativeGroupQuadsKHR"),
            Self::FRAGMENT_DENSITY_EXT => write!(f, "FragmentDensityEXT"),
            Self::GROUP_NON_UNIFORM_PARTITIONED_EXT => write!(f, "GroupNonUniformPartitionedEXT"),
            Self::SHADER_NON_UNIFORM => write!(f, "ShaderNonUniform"),
            Self::RUNTIME_DESCRIPTOR_ARRAY => write!(f, "RuntimeDescriptorArray"),
            Self::INPUT_ATTACHMENT_ARRAY_DYNAMIC_INDEXING => {
                write!(f, "InputAttachmentArrayDynamicIndexing")
            }
            Self::UNIFORM_TEXEL_BUFFER_ARRAY_DYNAMIC_INDEXING => {
                write!(f, "UniformTexelBufferArrayDynamicIndexing")
            }
            Self::STORAGE_TEXEL_BUFFER_ARRAY_DYNAMIC_INDEXING => {
                write!(f, "StorageTexelBufferArrayDynamicIndexing")
            }
            Self::UNIFORM_BUFFER_ARRAY_NON_UNIFORM_INDEXING => {
                write!(f, "UniformBufferArrayNonUniformIndexing")
            }
            Self::SAMPLED_IMAGE_ARRAY_NON_UNIFORM_INDEXING => {
                write!(f, "SampledImageArrayNonUniformIndexing")
            }
            Self::STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING => {
                write!(f, "StorageBufferArrayNonUniformIndexing")
            }
            Self::STORAGE_IMAGE_ARRAY_NON_UNIFORM_INDEXING => {
                write!(f, "StorageImageArrayNonUniformIndexing")
            }
            Self::INPUT_ATTACHMENT_ARRAY_NON_UNIFORM_INDEXING => {
                write!(f, "InputAttachmentArrayNonUniformIndexing")
            }
            Self::UNIFORM_TEXEL_BUFFER_ARRAY_NON_UNIFORM_INDEXING => {
                write!(f, "UniformTexelBufferArrayNonUniformIndexing")
            }
            Self::STORAGE_TEXEL_BUFFER_ARRAY_NON_UNIFORM_INDEXING => {
                write!(f, "StorageTexelBufferArrayNonUniformIndexing")
            }
            Self::RAY_TRACING_POSITION_FETCH_KHR => write!(f, "RayTracingPositionFetchKHR"),
            Self::RAY_TRACING_NV => write!(f, "RayTracingNV"),
            Self::RAY_TRACING_MOTION_BLUR_NV => write!(f, "RayTracingMotionBlurNV"),
            Self::VULKAN_MEMORY_MODEL => write!(f, "VulkanMemoryModel"),
            Self::VULKAN_MEMORY_MODEL_DEVICE_SCOPE => write!(f, "VulkanMemoryModelDeviceScope"),
            Self::PHYSICAL_STORAGE_BUFFER_ADDRESSES => write!(f, "PhysicalStorageBufferAddresses"),
            Self::COMPUTE_DERIVATIVE_GROUP_LINEAR_KHR => {
                write!(f, "ComputeDerivativeGroupLinearKHR")
            }
            Self::RAY_TRACING_PROVISIONAL_KHR => write!(f, "RayTracingProvisionalKHR"),
            Self::COOPERATIVE_MATRIX_NV => write!(f, "CooperativeMatrixNV"),
            Self::FRAGMENT_SHADER_SAMPLE_INTERLOCK_EXT => {
                write!(f, "FragmentShaderSampleInterlockEXT")
            }
            Self::FRAGMENT_SHADER_SHADING_RATE_INTERLOCK_EXT => {
                write!(f, "FragmentShaderShadingRateInterlockEXT")
            }
            Self::SHADER_SMBUILTINS_NV => write!(f, "ShaderSMBuiltinsNV"),
            Self::FRAGMENT_SHADER_PIXEL_INTERLOCK_EXT => {
                write!(f, "FragmentShaderPixelInterlockEXT")
            }
            Self::DEMOTE_TO_HELPER_INVOCATION => write!(f, "DemoteToHelperInvocation"),
            Self::DISPLACEMENT_MICROMAP_NV => write!(f, "DisplacementMicromapNV"),
            Self::RAY_TRACING_OPACITY_MICROMAP_EXT => write!(f, "RayTracingOpacityMicromapEXT"),
            Self::SHADER_INVOCATION_REORDER_NV => write!(f, "ShaderInvocationReorderNV"),
            Self::SHADER_INVOCATION_REORDER_EXT => write!(f, "ShaderInvocationReorderEXT"),
            Self::BINDLESS_TEXTURE_NV => write!(f, "BindlessTextureNV"),
            Self::RAY_QUERY_POSITION_FETCH_KHR => write!(f, "RayQueryPositionFetchKHR"),
            Self::COOPERATIVE_VECTOR_NV => write!(f, "CooperativeVectorNV"),
            Self::ATOMIC_FLOAT16_VECTOR_NV => write!(f, "AtomicFloat16VectorNV"),
            Self::RAY_TRACING_DISPLACEMENT_MICROMAP_NV => {
                write!(f, "RayTracingDisplacementMicromapNV")
            }
            Self::RAW_ACCESS_CHAINS_NV => write!(f, "RawAccessChainsNV"),
            Self::RAY_TRACING_SPHERES_GEOMETRY_NV => write!(f, "RayTracingSpheresGeometryNV"),
            Self::RAY_TRACING_LINEAR_SWEPT_SPHERES_GEOMETRY_NV => {
                write!(f, "RayTracingLinearSweptSpheresGeometryNV")
            }
            Self::PUSH_CONSTANT_BANKS_NV => write!(f, "PushConstantBanksNV"),
            Self::LONG_VECTOR_EXT => write!(f, "LongVectorEXT"),
            Self::SHADER64_BIT_INDEXING_EXT => write!(f, "Shader64BitIndexingEXT"),
            Self::COOPERATIVE_MATRIX_REDUCTIONS_NV => write!(f, "CooperativeMatrixReductionsNV"),
            Self::COOPERATIVE_MATRIX_CONVERSIONS_NV => write!(f, "CooperativeMatrixConversionsNV"),
            Self::COOPERATIVE_MATRIX_PER_ELEMENT_OPERATIONS_NV => {
                write!(f, "CooperativeMatrixPerElementOperationsNV")
            }
            Self::COOPERATIVE_MATRIX_TENSOR_ADDRESSING_NV => {
                write!(f, "CooperativeMatrixTensorAddressingNV")
            }
            Self::COOPERATIVE_MATRIX_BLOCK_LOADS_NV => write!(f, "CooperativeMatrixBlockLoadsNV"),
            Self::COOPERATIVE_VECTOR_TRAINING_NV => write!(f, "CooperativeVectorTrainingNV"),
            Self::RAY_TRACING_CLUSTER_ACCELERATION_STRUCTURE_NV => {
                write!(f, "RayTracingClusterAccelerationStructureNV")
            }
            Self::TENSOR_ADDRESSING_NV => write!(f, "TensorAddressingNV"),
            Self::SUBGROUP_SHUFFLE_INTEL => write!(f, "SubgroupShuffleINTEL"),
            Self::SUBGROUP_BUFFER_BLOCK_IOINTEL => write!(f, "SubgroupBufferBlockIOINTEL"),
            Self::SUBGROUP_IMAGE_BLOCK_IOINTEL => write!(f, "SubgroupImageBlockIOINTEL"),
            Self::SUBGROUP_IMAGE_MEDIA_BLOCK_IOINTEL => write!(f, "SubgroupImageMediaBlockIOINTEL"),
            Self::ROUND_TO_INFINITY_INTEL => write!(f, "RoundToInfinityINTEL"),
            Self::FLOATING_POINT_MODE_INTEL => write!(f, "FloatingPointModeINTEL"),
            Self::INTEGER_FUNCTIONS2_INTEL => write!(f, "IntegerFunctions2INTEL"),
            Self::FUNCTION_POINTERS_INTEL => write!(f, "FunctionPointersINTEL"),
            Self::INDIRECT_REFERENCES_INTEL => write!(f, "IndirectReferencesINTEL"),
            Self::ASM_INTEL => write!(f, "AsmINTEL"),
            Self::ATOMIC_FLOAT32_MIN_MAX_EXT => write!(f, "AtomicFloat32MinMaxEXT"),
            Self::ATOMIC_FLOAT64_MIN_MAX_EXT => write!(f, "AtomicFloat64MinMaxEXT"),
            Self::ATOMIC_FLOAT16_MIN_MAX_EXT => write!(f, "AtomicFloat16MinMaxEXT"),
            Self::VECTOR_COMPUTE_INTEL => write!(f, "VectorComputeINTEL"),
            Self::VECTOR_ANY_INTEL => write!(f, "VectorAnyINTEL"),
            Self::EXPECT_ASSUME_KHR => write!(f, "ExpectAssumeKHR"),
            Self::SUBGROUP_AVC_MOTION_ESTIMATION_INTEL => {
                write!(f, "SubgroupAvcMotionEstimationINTEL")
            }
            Self::SUBGROUP_AVC_MOTION_ESTIMATION_INTRA_INTEL => {
                write!(f, "SubgroupAvcMotionEstimationIntraINTEL")
            }
            Self::SUBGROUP_AVC_MOTION_ESTIMATION_CHROMA_INTEL => {
                write!(f, "SubgroupAvcMotionEstimationChromaINTEL")
            }
            Self::VARIABLE_LENGTH_ARRAY_INTEL => write!(f, "VariableLengthArrayINTEL"),
            Self::FUNCTION_FLOAT_CONTROL_INTEL => write!(f, "FunctionFloatControlINTEL"),
            Self::FPGAMEMORY_ATTRIBUTES_ALTERA => write!(f, "FPGAMemoryAttributesALTERA"),
            Self::FPFAST_MATH_MODE_INTEL => write!(f, "FPFastMathModeINTEL"),
            Self::ARBITRARY_PRECISION_INTEGERS_ALTERA => {
                write!(f, "ArbitraryPrecisionIntegersALTERA")
            }
            Self::ARBITRARY_PRECISION_FLOATING_POINT_ALTERA => {
                write!(f, "ArbitraryPrecisionFloatingPointALTERA")
            }
            Self::UNSTRUCTURED_LOOP_CONTROLS_INTEL => write!(f, "UnstructuredLoopControlsINTEL"),
            Self::FPGALOOP_CONTROLS_ALTERA => write!(f, "FPGALoopControlsALTERA"),
            Self::KERNEL_ATTRIBUTES_INTEL => write!(f, "KernelAttributesINTEL"),
            Self::FPGAKERNEL_ATTRIBUTES_INTEL => write!(f, "FPGAKernelAttributesINTEL"),
            Self::FPGAMEMORY_ACCESSES_ALTERA => write!(f, "FPGAMemoryAccessesALTERA"),
            Self::FPGACLUSTER_ATTRIBUTES_ALTERA => write!(f, "FPGAClusterAttributesALTERA"),
            Self::LOOP_FUSE_ALTERA => write!(f, "LoopFuseALTERA"),
            Self::FPGADSPCONTROL_ALTERA => write!(f, "FPGADSPControlALTERA"),
            Self::MEMORY_ACCESS_ALIASING_INTEL => write!(f, "MemoryAccessAliasingINTEL"),
            Self::FPGAINVOCATION_PIPELINING_ATTRIBUTES_ALTERA => {
                write!(f, "FPGAInvocationPipeliningAttributesALTERA")
            }
            Self::FPGABUFFER_LOCATION_ALTERA => write!(f, "FPGABufferLocationALTERA"),
            Self::ARBITRARY_PRECISION_FIXED_POINT_ALTERA => {
                write!(f, "ArbitraryPrecisionFixedPointALTERA")
            }
            Self::USMSTORAGE_CLASSES_ALTERA => write!(f, "USMStorageClassesALTERA"),
            Self::RUNTIME_ALIGNED_ATTRIBUTE_ALTERA => write!(f, "RuntimeAlignedAttributeALTERA"),
            Self::IOPIPES_ALTERA => write!(f, "IOPipesALTERA"),
            Self::BLOCKING_PIPES_ALTERA => write!(f, "BlockingPipesALTERA"),
            Self::FPGAREG_ALTERA => write!(f, "FPGARegALTERA"),
            Self::DOT_PRODUCT_INPUT_ALL => write!(f, "DotProductInputAll"),
            Self::DOT_PRODUCT_INPUT4X8_BIT => write!(f, "DotProductInput4x8Bit"),
            Self::DOT_PRODUCT_INPUT4X8_BIT_PACKED => write!(f, "DotProductInput4x8BitPacked"),
            Self::DOT_PRODUCT => write!(f, "DotProduct"),
            Self::RAY_CULL_MASK_KHR => write!(f, "RayCullMaskKHR"),
            Self::COOPERATIVE_MATRIX_KHR => write!(f, "CooperativeMatrixKHR"),
            Self::REPLICATED_COMPOSITES_EXT => write!(f, "ReplicatedCompositesEXT"),
            Self::BIT_INSTRUCTIONS => write!(f, "BitInstructions"),
            Self::GROUP_NON_UNIFORM_ROTATE_KHR => write!(f, "GroupNonUniformRotateKHR"),
            Self::FLOAT_CONTROLS2 => write!(f, "FloatControls2"),
            Self::FMAKHR => write!(f, "FMAKHR"),
            Self::ATOMIC_FLOAT32_ADD_EXT => write!(f, "AtomicFloat32AddEXT"),
            Self::ATOMIC_FLOAT64_ADD_EXT => write!(f, "AtomicFloat64AddEXT"),
            Self::LONG_COMPOSITES_INTEL => write!(f, "LongCompositesINTEL"),
            Self::OPT_NONE_EXT => write!(f, "OptNoneEXT"),
            Self::ATOMIC_FLOAT16_ADD_EXT => write!(f, "AtomicFloat16AddEXT"),
            Self::DEBUG_INFO_MODULE_INTEL => write!(f, "DebugInfoModuleINTEL"),
            Self::BFLOAT16_CONVERSION_INTEL => write!(f, "BFloat16ConversionINTEL"),
            Self::SPLIT_BARRIER_INTEL => write!(f, "SplitBarrierINTEL"),
            Self::ARITHMETIC_FENCE_EXT => write!(f, "ArithmeticFenceEXT"),
            Self::FPGACLUSTER_ATTRIBUTES_V2_ALTERA => write!(f, "FPGAClusterAttributesV2ALTERA"),
            Self::FPGAKERNEL_ATTRIBUTESV2_INTEL => write!(f, "FPGAKernelAttributesv2INTEL"),
            Self::TASK_SEQUENCE_ALTERA => write!(f, "TaskSequenceALTERA"),
            Self::FPMAX_ERROR_INTEL => write!(f, "FPMaxErrorINTEL"),
            Self::FPGALATENCY_CONTROL_ALTERA => write!(f, "FPGALatencyControlALTERA"),
            Self::FPGAARGUMENT_INTERFACES_ALTERA => write!(f, "FPGAArgumentInterfacesALTERA"),
            Self::GLOBAL_VARIABLE_HOST_ACCESS_INTEL => write!(f, "GlobalVariableHostAccessINTEL"),
            Self::GLOBAL_VARIABLE_FPGADECORATIONS_ALTERA => {
                write!(f, "GlobalVariableFPGADecorationsALTERA")
            }
            Self::SUBGROUP_BUFFER_PREFETCH_INTEL => write!(f, "SubgroupBufferPrefetchINTEL"),
            Self::SUBGROUP2_DBLOCK_IOINTEL => write!(f, "Subgroup2DBlockIOINTEL"),
            Self::SUBGROUP2_DBLOCK_TRANSFORM_INTEL => write!(f, "Subgroup2DBlockTransformINTEL"),
            Self::SUBGROUP2_DBLOCK_TRANSPOSE_INTEL => write!(f, "Subgroup2DBlockTransposeINTEL"),
            Self::SUBGROUP_MATRIX_MULTIPLY_ACCUMULATE_INTEL => {
                write!(f, "SubgroupMatrixMultiplyAccumulateINTEL")
            }
            Self::TERNARY_BITWISE_FUNCTION_INTEL => write!(f, "TernaryBitwiseFunctionINTEL"),
            Self::UNTYPED_VARIABLE_LENGTH_ARRAY_INTEL => {
                write!(f, "UntypedVariableLengthArrayINTEL")
            }
            Self::SPEC_CONDITIONAL_INTEL => write!(f, "SpecConditionalINTEL"),
            Self::FUNCTION_VARIANTS_INTEL => write!(f, "FunctionVariantsINTEL"),
            Self::GROUP_UNIFORM_ARITHMETIC_KHR => write!(f, "GroupUniformArithmeticKHR"),
            Self::TENSOR_FLOAT32_ROUNDING_INTEL => write!(f, "TensorFloat32RoundingINTEL"),
            Self::MASKED_GATHER_SCATTER_INTEL => write!(f, "MaskedGatherScatterINTEL"),
            Self::CACHE_CONTROLS_INTEL => write!(f, "CacheControlsINTEL"),
            Self::REGISTER_LIMITS_INTEL => write!(f, "RegisterLimitsINTEL"),
            Self::BINDLESS_IMAGES_INTEL => write!(f, "BindlessImagesINTEL"),
            Self::DOT_PRODUCT_FLOAT16_ACC_FLOAT32_VALVE => {
                write!(f, "DotProductFloat16AccFloat32VALVE")
            }
            Self::DOT_PRODUCT_FLOAT16_ACC_FLOAT16_VALVE => {
                write!(f, "DotProductFloat16AccFloat16VALVE")
            }
            Self::DOT_PRODUCT_BFLOAT16_ACC_VALVE => write!(f, "DotProductBFloat16AccVALVE"),
            Self::DOT_PRODUCT_FLOAT8_ACC_FLOAT32_VALVE => {
                write!(f, "DotProductFloat8AccFloat32VALVE")
            }
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RayQueryIntersection(pub(crate) u32);
impl RayQueryIntersection {
    pub const RAY_QUERY_CANDIDATE_INTERSECTION_KHR: Self = Self(0u32);
    pub const RAY_QUERY_COMMITTED_INTERSECTION_KHR: Self = Self(1u32);
}
impl Word for RayQueryIntersection {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for RayQueryIntersection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::RAY_QUERY_CANDIDATE_INTERSECTION_KHR => {
                write!(f, "RayQueryCandidateIntersectionKHR")
            }
            Self::RAY_QUERY_COMMITTED_INTERSECTION_KHR => {
                write!(f, "RayQueryCommittedIntersectionKHR")
            }
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RayQueryCommittedIntersectionType(pub(crate) u32);
impl RayQueryCommittedIntersectionType {
    pub const RAY_QUERY_COMMITTED_INTERSECTION_NONE_KHR: Self = Self(0u32);
    pub const RAY_QUERY_COMMITTED_INTERSECTION_TRIANGLE_KHR: Self = Self(1u32);
    pub const RAY_QUERY_COMMITTED_INTERSECTION_GENERATED_KHR: Self = Self(2u32);
}
impl Word for RayQueryCommittedIntersectionType {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for RayQueryCommittedIntersectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::RAY_QUERY_COMMITTED_INTERSECTION_NONE_KHR => {
                write!(f, "RayQueryCommittedIntersectionNoneKHR")
            }
            Self::RAY_QUERY_COMMITTED_INTERSECTION_TRIANGLE_KHR => {
                write!(f, "RayQueryCommittedIntersectionTriangleKHR")
            }
            Self::RAY_QUERY_COMMITTED_INTERSECTION_GENERATED_KHR => {
                write!(f, "RayQueryCommittedIntersectionGeneratedKHR")
            }
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RayQueryCandidateIntersectionType(pub(crate) u32);
impl RayQueryCandidateIntersectionType {
    pub const RAY_QUERY_CANDIDATE_INTERSECTION_TRIANGLE_KHR: Self = Self(0u32);
    pub const RAY_QUERY_CANDIDATE_INTERSECTION_AABBKHR: Self = Self(1u32);
}
impl Word for RayQueryCandidateIntersectionType {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for RayQueryCandidateIntersectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::RAY_QUERY_CANDIDATE_INTERSECTION_TRIANGLE_KHR => {
                write!(f, "RayQueryCandidateIntersectionTriangleKHR")
            }
            Self::RAY_QUERY_CANDIDATE_INTERSECTION_AABBKHR => {
                write!(f, "RayQueryCandidateIntersectionAABBKHR")
            }
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PackedVectorFormat(pub(crate) u32);
impl PackedVectorFormat {
    pub const PACKED_VECTOR_FORMAT4X8_BIT: Self = Self(0u32);
}
impl Word for PackedVectorFormat {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for PackedVectorFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::PACKED_VECTOR_FORMAT4X8_BIT => write!(f, "PackedVectorFormat4x8Bit"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CooperativeMatrixOperands(pub(crate) u32);
impl CooperativeMatrixOperands {
    pub const NONE_KHR: Self = Self(0x0000);
    pub const MATRIX_ASIGNED_COMPONENTS_KHR: Self = Self(0x0001);
    pub const MATRIX_BSIGNED_COMPONENTS_KHR: Self = Self(0x0002);
    pub const MATRIX_CSIGNED_COMPONENTS_KHR: Self = Self(0x0004);
    pub const MATRIX_RESULT_SIGNED_COMPONENTS_KHR: Self = Self(0x0008);
    pub const SATURATING_ACCUMULATION_KHR: Self = Self(0x0010);
}
impl Word for CooperativeMatrixOperands {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for CooperativeMatrixOperands {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for CooperativeMatrixOperands {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for CooperativeMatrixOperands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::MATRIX_ASIGNED_COMPONENTS_KHR == Self::MATRIX_ASIGNED_COMPONENTS_KHR {
            bitfmt.fmt(Self::MATRIX_ASIGNED_COMPONENTS_KHR.0, f)?;
            write!(f, "MatrixASignedComponentsKHR")?;
        }
        if *self & Self::MATRIX_BSIGNED_COMPONENTS_KHR == Self::MATRIX_BSIGNED_COMPONENTS_KHR {
            bitfmt.fmt(Self::MATRIX_BSIGNED_COMPONENTS_KHR.0, f)?;
            write!(f, "MatrixBSignedComponentsKHR")?;
        }
        if *self & Self::MATRIX_CSIGNED_COMPONENTS_KHR == Self::MATRIX_CSIGNED_COMPONENTS_KHR {
            bitfmt.fmt(Self::MATRIX_CSIGNED_COMPONENTS_KHR.0, f)?;
            write!(f, "MatrixCSignedComponentsKHR")?;
        }
        if *self & Self::MATRIX_RESULT_SIGNED_COMPONENTS_KHR
            == Self::MATRIX_RESULT_SIGNED_COMPONENTS_KHR
        {
            bitfmt.fmt(Self::MATRIX_RESULT_SIGNED_COMPONENTS_KHR.0, f)?;
            write!(f, "MatrixResultSignedComponentsKHR")?;
        }
        if *self & Self::SATURATING_ACCUMULATION_KHR == Self::SATURATING_ACCUMULATION_KHR {
            bitfmt.fmt(Self::SATURATING_ACCUMULATION_KHR.0, f)?;
            write!(f, "SaturatingAccumulationKHR")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CooperativeMatrixLayout(pub(crate) u32);
impl CooperativeMatrixLayout {
    pub const ROW_MAJOR_KHR: Self = Self(0u32);
    pub const COLUMN_MAJOR_KHR: Self = Self(1u32);
    pub const ROW_BLOCKED_INTERLEAVED_ARM: Self = Self(4202u32);
    pub const COLUMN_BLOCKED_INTERLEAVED_ARM: Self = Self(4203u32);
}
impl Word for CooperativeMatrixLayout {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for CooperativeMatrixLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ROW_MAJOR_KHR => write!(f, "RowMajorKHR"),
            Self::COLUMN_MAJOR_KHR => write!(f, "ColumnMajorKHR"),
            Self::ROW_BLOCKED_INTERLEAVED_ARM => write!(f, "RowBlockedInterleavedARM"),
            Self::COLUMN_BLOCKED_INTERLEAVED_ARM => write!(f, "ColumnBlockedInterleavedARM"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CooperativeMatrixUse(pub(crate) u32);
impl CooperativeMatrixUse {
    pub const MATRIX_AKHR: Self = Self(0u32);
    pub const MATRIX_BKHR: Self = Self(1u32);
    pub const MATRIX_ACCUMULATOR_KHR: Self = Self(2u32);
}
impl Word for CooperativeMatrixUse {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for CooperativeMatrixUse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::MATRIX_AKHR => write!(f, "MatrixAKHR"),
            Self::MATRIX_BKHR => write!(f, "MatrixBKHR"),
            Self::MATRIX_ACCUMULATOR_KHR => write!(f, "MatrixAccumulatorKHR"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CooperativeMatrixReduce(pub(crate) u32);
impl CooperativeMatrixReduce {
    pub const ROW: Self = Self(0x0001);
    pub const COLUMN: Self = Self(0x0002);
    pub const TYPE_2X2: Self = Self(0x0004);
}
impl Word for CooperativeMatrixReduce {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for CooperativeMatrixReduce {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for CooperativeMatrixReduce {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for CooperativeMatrixReduce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::ROW == Self::ROW {
            bitfmt.fmt(Self::ROW.0, f)?;
            write!(f, "Row")?;
        }
        if *self & Self::COLUMN == Self::COLUMN {
            bitfmt.fmt(Self::COLUMN.0, f)?;
            write!(f, "Column")?;
        }
        if *self & Self::TYPE_2X2 == Self::TYPE_2X2 {
            bitfmt.fmt(Self::TYPE_2X2.0, f)?;
            write!(f, "2x2")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TensorClampMode(pub(crate) u32);
impl TensorClampMode {
    pub const UNDEFINED: Self = Self(0u32);
    pub const CONSTANT: Self = Self(1u32);
    pub const CLAMP_TO_EDGE: Self = Self(2u32);
    pub const REPEAT: Self = Self(3u32);
    pub const REPEAT_MIRRORED: Self = Self(4u32);
}
impl Word for TensorClampMode {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for TensorClampMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UNDEFINED => write!(f, "Undefined"),
            Self::CONSTANT => write!(f, "Constant"),
            Self::CLAMP_TO_EDGE => write!(f, "ClampToEdge"),
            Self::REPEAT => write!(f, "Repeat"),
            Self::REPEAT_MIRRORED => write!(f, "RepeatMirrored"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TensorAddressingOperands(pub(crate) u32);
impl TensorAddressingOperands {
    pub const NONE: Self = Self(0x0000);
    pub const TENSOR_VIEW: Self = Self(0x0001);
    pub const DECODE_FUNC: Self = Self(0x0002);
}
impl Word for TensorAddressingOperands {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for TensorAddressingOperands {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for TensorAddressingOperands {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for TensorAddressingOperands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::TENSOR_VIEW == Self::TENSOR_VIEW {
            bitfmt.fmt(Self::TENSOR_VIEW.0, f)?;
            write!(f, "TensorView")?;
        }
        if *self & Self::DECODE_FUNC == Self::DECODE_FUNC {
            bitfmt.fmt(Self::DECODE_FUNC.0, f)?;
            write!(f, "DecodeFunc")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InitializationModeQualifier(pub(crate) u32);
impl InitializationModeQualifier {
    pub const INIT_ON_DEVICE_REPROGRAM_ALTERA: Self = Self(0u32);
    pub const INIT_ON_DEVICE_RESET_ALTERA: Self = Self(1u32);
}
impl Word for InitializationModeQualifier {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for InitializationModeQualifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::INIT_ON_DEVICE_REPROGRAM_ALTERA => write!(f, "InitOnDeviceReprogramALTERA"),
            Self::INIT_ON_DEVICE_RESET_ALTERA => write!(f, "InitOnDeviceResetALTERA"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LoadCacheControl(pub(crate) u32);
impl LoadCacheControl {
    pub const UNCACHED_INTEL: Self = Self(0u32);
    pub const CACHED_INTEL: Self = Self(1u32);
    pub const STREAMING_INTEL: Self = Self(2u32);
    pub const INVALIDATE_AFTER_READ_INTEL: Self = Self(3u32);
    pub const CONST_CACHED_INTEL: Self = Self(4u32);
}
impl Word for LoadCacheControl {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for LoadCacheControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UNCACHED_INTEL => write!(f, "UncachedINTEL"),
            Self::CACHED_INTEL => write!(f, "CachedINTEL"),
            Self::STREAMING_INTEL => write!(f, "StreamingINTEL"),
            Self::INVALIDATE_AFTER_READ_INTEL => write!(f, "InvalidateAfterReadINTEL"),
            Self::CONST_CACHED_INTEL => write!(f, "ConstCachedINTEL"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StoreCacheControl(pub(crate) u32);
impl StoreCacheControl {
    pub const UNCACHED_INTEL: Self = Self(0u32);
    pub const WRITE_THROUGH_INTEL: Self = Self(1u32);
    pub const WRITE_BACK_INTEL: Self = Self(2u32);
    pub const STREAMING_INTEL: Self = Self(3u32);
}
impl Word for StoreCacheControl {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for StoreCacheControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UNCACHED_INTEL => write!(f, "UncachedINTEL"),
            Self::WRITE_THROUGH_INTEL => write!(f, "WriteThroughINTEL"),
            Self::WRITE_BACK_INTEL => write!(f, "WriteBackINTEL"),
            Self::STREAMING_INTEL => write!(f, "StreamingINTEL"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NamedMaximumNumberOfRegisters(pub(crate) u32);
impl NamedMaximumNumberOfRegisters {
    pub const AUTO_INTEL: Self = Self(0u32);
}
impl Word for NamedMaximumNumberOfRegisters {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for NamedMaximumNumberOfRegisters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::AUTO_INTEL => write!(f, "AutoINTEL"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MatrixMultiplyAccumulateOperands(pub(crate) u32);
impl MatrixMultiplyAccumulateOperands {
    pub const NONE: Self = Self(0x0);
    pub const MATRIX_ASIGNED_COMPONENTS_INTEL: Self = Self(0x1);
    pub const MATRIX_BSIGNED_COMPONENTS_INTEL: Self = Self(0x2);
    pub const MATRIX_CBFLOAT16_INTEL: Self = Self(0x4);
    pub const MATRIX_RESULT_BFLOAT16_INTEL: Self = Self(0x8);
    pub const MATRIX_APACKED_INT8_INTEL: Self = Self(0x10);
    pub const MATRIX_BPACKED_INT8_INTEL: Self = Self(0x20);
    pub const MATRIX_APACKED_INT4_INTEL: Self = Self(0x40);
    pub const MATRIX_BPACKED_INT4_INTEL: Self = Self(0x80);
    pub const MATRIX_ATF32_INTEL: Self = Self(0x100);
    pub const MATRIX_BTF32_INTEL: Self = Self(0x200);
    pub const MATRIX_APACKED_FLOAT16_INTEL: Self = Self(0x400);
    pub const MATRIX_BPACKED_FLOAT16_INTEL: Self = Self(0x800);
    pub const MATRIX_APACKED_BFLOAT16_INTEL: Self = Self(0x1000);
    pub const MATRIX_BPACKED_BFLOAT16_INTEL: Self = Self(0x2000);
}
impl Word for MatrixMultiplyAccumulateOperands {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for MatrixMultiplyAccumulateOperands {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for MatrixMultiplyAccumulateOperands {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for MatrixMultiplyAccumulateOperands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::MATRIX_ASIGNED_COMPONENTS_INTEL == Self::MATRIX_ASIGNED_COMPONENTS_INTEL {
            bitfmt.fmt(Self::MATRIX_ASIGNED_COMPONENTS_INTEL.0, f)?;
            write!(f, "MatrixASignedComponentsINTEL")?;
        }
        if *self & Self::MATRIX_BSIGNED_COMPONENTS_INTEL == Self::MATRIX_BSIGNED_COMPONENTS_INTEL {
            bitfmt.fmt(Self::MATRIX_BSIGNED_COMPONENTS_INTEL.0, f)?;
            write!(f, "MatrixBSignedComponentsINTEL")?;
        }
        if *self & Self::MATRIX_CBFLOAT16_INTEL == Self::MATRIX_CBFLOAT16_INTEL {
            bitfmt.fmt(Self::MATRIX_CBFLOAT16_INTEL.0, f)?;
            write!(f, "MatrixCBFloat16INTEL")?;
        }
        if *self & Self::MATRIX_RESULT_BFLOAT16_INTEL == Self::MATRIX_RESULT_BFLOAT16_INTEL {
            bitfmt.fmt(Self::MATRIX_RESULT_BFLOAT16_INTEL.0, f)?;
            write!(f, "MatrixResultBFloat16INTEL")?;
        }
        if *self & Self::MATRIX_APACKED_INT8_INTEL == Self::MATRIX_APACKED_INT8_INTEL {
            bitfmt.fmt(Self::MATRIX_APACKED_INT8_INTEL.0, f)?;
            write!(f, "MatrixAPackedInt8INTEL")?;
        }
        if *self & Self::MATRIX_BPACKED_INT8_INTEL == Self::MATRIX_BPACKED_INT8_INTEL {
            bitfmt.fmt(Self::MATRIX_BPACKED_INT8_INTEL.0, f)?;
            write!(f, "MatrixBPackedInt8INTEL")?;
        }
        if *self & Self::MATRIX_APACKED_INT4_INTEL == Self::MATRIX_APACKED_INT4_INTEL {
            bitfmt.fmt(Self::MATRIX_APACKED_INT4_INTEL.0, f)?;
            write!(f, "MatrixAPackedInt4INTEL")?;
        }
        if *self & Self::MATRIX_BPACKED_INT4_INTEL == Self::MATRIX_BPACKED_INT4_INTEL {
            bitfmt.fmt(Self::MATRIX_BPACKED_INT4_INTEL.0, f)?;
            write!(f, "MatrixBPackedInt4INTEL")?;
        }
        if *self & Self::MATRIX_ATF32_INTEL == Self::MATRIX_ATF32_INTEL {
            bitfmt.fmt(Self::MATRIX_ATF32_INTEL.0, f)?;
            write!(f, "MatrixATF32INTEL")?;
        }
        if *self & Self::MATRIX_BTF32_INTEL == Self::MATRIX_BTF32_INTEL {
            bitfmt.fmt(Self::MATRIX_BTF32_INTEL.0, f)?;
            write!(f, "MatrixBTF32INTEL")?;
        }
        if *self & Self::MATRIX_APACKED_FLOAT16_INTEL == Self::MATRIX_APACKED_FLOAT16_INTEL {
            bitfmt.fmt(Self::MATRIX_APACKED_FLOAT16_INTEL.0, f)?;
            write!(f, "MatrixAPackedFloat16INTEL")?;
        }
        if *self & Self::MATRIX_BPACKED_FLOAT16_INTEL == Self::MATRIX_BPACKED_FLOAT16_INTEL {
            bitfmt.fmt(Self::MATRIX_BPACKED_FLOAT16_INTEL.0, f)?;
            write!(f, "MatrixBPackedFloat16INTEL")?;
        }
        if *self & Self::MATRIX_APACKED_BFLOAT16_INTEL == Self::MATRIX_APACKED_BFLOAT16_INTEL {
            bitfmt.fmt(Self::MATRIX_APACKED_BFLOAT16_INTEL.0, f)?;
            write!(f, "MatrixAPackedBFloat16INTEL")?;
        }
        if *self & Self::MATRIX_BPACKED_BFLOAT16_INTEL == Self::MATRIX_BPACKED_BFLOAT16_INTEL {
            bitfmt.fmt(Self::MATRIX_BPACKED_BFLOAT16_INTEL.0, f)?;
            write!(f, "MatrixBPackedBFloat16INTEL")?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FPEncoding(pub(crate) u32);
impl FPEncoding {
    pub const BFLOAT16_KHR: Self = Self(0u32);
    pub const FLOAT8_E4_M3_EXT: Self = Self(4214u32);
    pub const FLOAT8_E5_M2_EXT: Self = Self(4215u32);
}
impl Word for FPEncoding {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for FPEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::BFLOAT16_KHR => write!(f, "BFloat16KHR"),
            Self::FLOAT8_E4_M3_EXT => write!(f, "Float8E4M3EXT"),
            Self::FLOAT8_E5_M2_EXT => write!(f, "Float8E5M2EXT"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CooperativeVectorMatrixLayout(pub(crate) u32);
impl CooperativeVectorMatrixLayout {
    pub const ROW_MAJOR_NV: Self = Self(0u32);
    pub const COLUMN_MAJOR_NV: Self = Self(1u32);
    pub const INFERENCING_OPTIMAL_NV: Self = Self(2u32);
    pub const TRAINING_OPTIMAL_NV: Self = Self(3u32);
}
impl Word for CooperativeVectorMatrixLayout {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for CooperativeVectorMatrixLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ROW_MAJOR_NV => write!(f, "RowMajorNV"),
            Self::COLUMN_MAJOR_NV => write!(f, "ColumnMajorNV"),
            Self::INFERENCING_OPTIMAL_NV => write!(f, "InferencingOptimalNV"),
            Self::TRAINING_OPTIMAL_NV => write!(f, "TrainingOptimalNV"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ComponentType(pub(crate) u32);
impl ComponentType {
    pub const FLOAT16_NV: Self = Self(0u32);
    pub const FLOAT32_NV: Self = Self(1u32);
    pub const FLOAT64_NV: Self = Self(2u32);
    pub const SIGNED_INT8_NV: Self = Self(3u32);
    pub const SIGNED_INT16_NV: Self = Self(4u32);
    pub const SIGNED_INT32_NV: Self = Self(5u32);
    pub const SIGNED_INT64_NV: Self = Self(6u32);
    pub const UNSIGNED_INT8_NV: Self = Self(7u32);
    pub const UNSIGNED_INT16_NV: Self = Self(8u32);
    pub const UNSIGNED_INT32_NV: Self = Self(9u32);
    pub const UNSIGNED_INT64_NV: Self = Self(10u32);
    pub const SIGNED_INT8_PACKED_NV: Self = Self(1000491000u32);
    pub const UNSIGNED_INT8_PACKED_NV: Self = Self(1000491001u32);
    pub const FLOAT_E4_M3_NV: Self = Self(1000491002u32);
    pub const FLOAT_E5_M2_NV: Self = Self(1000491003u32);
}
impl Word for ComponentType {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::FLOAT16_NV => write!(f, "Float16NV"),
            Self::FLOAT32_NV => write!(f, "Float32NV"),
            Self::FLOAT64_NV => write!(f, "Float64NV"),
            Self::SIGNED_INT8_NV => write!(f, "SignedInt8NV"),
            Self::SIGNED_INT16_NV => write!(f, "SignedInt16NV"),
            Self::SIGNED_INT32_NV => write!(f, "SignedInt32NV"),
            Self::SIGNED_INT64_NV => write!(f, "SignedInt64NV"),
            Self::UNSIGNED_INT8_NV => write!(f, "UnsignedInt8NV"),
            Self::UNSIGNED_INT16_NV => write!(f, "UnsignedInt16NV"),
            Self::UNSIGNED_INT32_NV => write!(f, "UnsignedInt32NV"),
            Self::UNSIGNED_INT64_NV => write!(f, "UnsignedInt64NV"),
            Self::SIGNED_INT8_PACKED_NV => write!(f, "SignedInt8PackedNV"),
            Self::UNSIGNED_INT8_PACKED_NV => write!(f, "UnsignedInt8PackedNV"),
            Self::FLOAT_E4_M3_NV => write!(f, "FloatE4M3NV"),
            Self::FLOAT_E5_M2_NV => write!(f, "FloatE5M2NV"),
            x => write!(f, "{x}"),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TensorOperands(pub(crate) u32);
impl TensorOperands {
    pub const NONE_ARM: Self = Self(0x0000);
    pub const NONTEMPORAL_ARM: Self = Self(0x0001);
    pub const OUT_OF_BOUNDS_VALUE_ARM: Self = Self(0x0002);
    pub const MAKE_ELEMENT_AVAILABLE_ARM: Self = Self(0x0004);
    pub const MAKE_ELEMENT_VISIBLE_ARM: Self = Self(0x0008);
    pub const NON_PRIVATE_ELEMENT_ARM: Self = Self(0x0010);
}
impl Word for TensorOperands {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl BitAnd for TensorOperands {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for TensorOperands {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl Display for TensorOperands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "None");
        }
        let mut bitfmt = BitFmt(0);
        if *self & Self::NONTEMPORAL_ARM == Self::NONTEMPORAL_ARM {
            bitfmt.fmt(Self::NONTEMPORAL_ARM.0, f)?;
            write!(f, "NontemporalARM")?;
        }
        if *self & Self::OUT_OF_BOUNDS_VALUE_ARM == Self::OUT_OF_BOUNDS_VALUE_ARM {
            bitfmt.fmt(Self::OUT_OF_BOUNDS_VALUE_ARM.0, f)?;
            write!(f, "OutOfBoundsValueARM")?;
        }
        if *self & Self::MAKE_ELEMENT_AVAILABLE_ARM == Self::MAKE_ELEMENT_AVAILABLE_ARM {
            bitfmt.fmt(Self::MAKE_ELEMENT_AVAILABLE_ARM.0, f)?;
            write!(f, "MakeElementAvailableARM")?;
        }
        if *self & Self::MAKE_ELEMENT_VISIBLE_ARM == Self::MAKE_ELEMENT_VISIBLE_ARM {
            bitfmt.fmt(Self::MAKE_ELEMENT_VISIBLE_ARM.0, f)?;
            write!(f, "MakeElementVisibleARM")?;
        }
        if *self & Self::NON_PRIVATE_ELEMENT_ARM == Self::NON_PRIVATE_ELEMENT_ARM {
            bitfmt.fmt(Self::NON_PRIVATE_ELEMENT_ARM.0, f)?;
            write!(f, "NonPrivateElementARM")?;
        }
        Ok(())
    }
}
