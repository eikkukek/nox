//! SPIR-V streams.

use crate::{
    core::*,
    ParseResult, ParseError,
    op,
};

/// An iterator over [`instructions`][1] in SPIR-V code.
///
/// [1]: InstructionStream
#[derive(Clone, Copy)]
pub struct Stream<'a> {
    spirv: &'a [u32],
    pos: usize,
}

impl<'a> Stream<'a> {

    #[inline(always)]
    pub fn new(spirv: &'a [u32]) -> Self {
        Self {
            spirv: &spirv[5..],
            pos: 0,
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.pos = 0;
    }
}

/// Represents a single SPIR-V instruction.
#[derive(Clone, Copy)]
pub struct InstructionStream<'a> {
    op: op::Code,
    words: &'a [u32],
    pos: usize,
}

impl<'a> InstructionStream<'a> {

    #[inline]
    pub fn code(&self) -> op::Code {
        self.op
    }

    #[inline]
    pub fn is_eos(&self) -> bool {
        self.pos == self.words.len()
    }

    #[inline]
    pub fn reset(&mut self) {
        self.pos = 1;
    }

    #[inline]
    pub fn read(&mut self) -> ParseResult<u32> {
        let len = self.words.len();
        let pos = self.pos;
        if pos == len { Err(ParseError::EndOfStream) }
        else {
            self.pos += 1;
            Ok(self.words[pos])
        }
    }

    #[inline]
    pub fn read_words(&mut self, count: Option<u32>) -> ParseResult<&'a [u32]> {
        let len = self.words.len();
        let pos = self.pos;
        if pos == len { Ok(Default::default()) }
        else {
            let end = count
                .map(|count| pos + count as usize)
                .unwrap_or(len);
            if end > len { Err(ParseError::EndOfStream) }
            else {
                self.pos = end;
                Ok(&self.words[pos..end])
            }
        }
    }

    #[inline]
    pub fn read_string(&mut self) -> ParseResult<CompilerStr<'a>> {
        let pos = self.pos;
        if pos == self.words.len() { Err(ParseError::EndOfStream) }
        else {
            let str = CompilerStr::new(&self.words[pos..]);
            self.pos = pos + str.len().div_ceil(8);
            Ok(str)
        }
    }
}

impl<'a> Iterator for Stream<'a> {

    type Item = InstructionStream<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let len = self.spirv.len();
        let pos = self.pos;
        if pos == len { None }
        else {
            let word = self.spirv[pos];
            let (count, op_code) = (word >> 16, word & 0xFFFF);
            let end = pos + count as usize;
            if end > len { None }
            else {
                let instruction = InstructionStream {
                    op: op::Code(op_code as u16),
                    words: &self.spirv[pos..end],
                    pos: 1,
                };
                self.pos = end;
                Some(instruction)
            }
        }
    }
}
