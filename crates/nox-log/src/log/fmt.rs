use nox_mem::vec_types::{GlobalVec, Vector};

use super::*;

#[derive(Default)]
pub struct LogFmt {
    segments: GlobalVec<SegmentSpec>,
}

impl LogFmt {

    fn clear(&mut self) {
        self.segments.clear();
    }
}

impl<'a> IntoIterator for &'a LogFmt {

    type Item = &'a SegmentSpec;
    type IntoIter = core::slice::Iter<'a, SegmentSpec>;

    fn into_iter(self) -> Self::IntoIter {
        self.segments.into_iter()
    }
}

#[derive(Default, Clone)]
pub struct LogSpec {
    pub color_spec: Option<ColorSpec>,
}

impl LogSpec {

    #[inline(always)]
    pub fn with_color_spec(mut self, mut f: impl FnMut(&mut ColorSpec)) -> Self {
        let color_spec = self.color_spec.insert(ColorSpec::new());
        f(color_spec);
        self
    }
}

pub enum SegmentSpec {
    Message(LogSpec),
    Text(CompactString, LogSpec),
}

pub struct LogFmtBuilder<'a> {
    fmt: &'a mut LogFmt,
}

impl<'a> LogFmtBuilder<'a> {

    #[inline(always)]
    pub fn new(fmt: &'a mut LogFmt) -> Self {
        fmt.clear();
        Self {
            fmt,
        }
    }

    #[inline(always)]
    pub fn message(&mut self, mut f: impl FnMut(LogSpec) -> LogSpec) -> &mut Self {
        self.fmt.segments.push(SegmentSpec::Message(f(Default::default())));
        self
    }

    #[inline(always)]
    pub fn text(&mut self, text: &str, mut f: impl FnMut(LogSpec) -> LogSpec) -> &mut Self {
        self.fmt.segments.push(SegmentSpec::Text(CompactString::new(text), f(Default::default())));
        self
    }
}
