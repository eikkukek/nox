use core::num::NonZeroU32;

#[derive(Clone, Copy, Debug)]
pub struct VertexRange {
    pub start: u32,
    pub end: NonZeroU32,
}

impl VertexRange {

    #[inline(always)]
    pub fn new(range: core::ops::Range<usize>) -> Option<Self> {
        (range.start != range.end).then_some(Self {
            start: range.start as u32,
            end: NonZeroU32::new(range.end as u32)?,
        })
    }

    #[inline(always)]
    pub fn start(self) -> usize {
        self.start as usize
    }

    #[inline(always)]
    pub fn end(self) -> usize {
        self.end.get() as usize
    }

    #[inline(always)]
    pub fn range(self) -> core::ops::Range<usize> {
        self.start as usize..self.end.get() as usize
    }
}
