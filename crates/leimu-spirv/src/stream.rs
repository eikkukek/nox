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
    version: u32,
    bound: u32,
    pos: usize,
}

impl<'a> Stream<'a> {

    /// Creates a new stream from SPIR-V words.
    #[inline(always)]
    pub fn new(spirv: &'a [u32]) -> ParseResult<Self> {
        if spirv.len() < 5 { return Err(ParseError::EndOfStream) }
        let magic_number = spirv[0];
        if magic_number != 0x07230203 {
            return Err(ParseError::InvalidMagicNumber(magic_number))
        }
        Ok(Self {
            spirv: &spirv[5..],
            version: spirv[1],
            bound: spirv[3],
            pos: 0,
        })
    }

    /// Resets the stream's position to zero.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// The version of the SPIR-V passed to this stream.
    #[inline(always)]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// The bound, where all ids are guaranteed to satisfy 0 < id < bound.
    #[inline(always)]
    pub fn bound(&self) -> u32 {
        self.bound
    }
}

/// Represents a single SPIR-V instruction stream.
#[derive(Clone, Copy)]
pub struct InstructionStream<'a> {
    op: op::Code,
    words: &'a [u32],
    pos: usize,
}

impl<'a> InstructionStream<'a> {

    /// Returns the instruction's [`Code`][1].
    ///
    /// [1]: op::Code
    #[inline]
    pub fn code(&self) -> op::Code {
        self.op
    }

    /// Returns whether the end of the stream has been reached.
    #[inline]
    pub fn is_eos(&self) -> bool {
        self.pos == self.words.len()
    }

    /// Resets the stream's position to one.
    #[inline]
    pub fn reset(&mut self) {
        self.pos = 1;
    }

    /// Advances the streams position without reading.
    #[inline]
    pub fn advance(&mut self, count: u32) -> ParseResult<()> {
        let end = self.pos + count as usize;
        if end > self.words.len() { Err(ParseError::EndOfStream) }
        else {
            self.pos = end;
            Ok(())
        }
    }
    
    /// Reads a word from the stream.
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

    /// Reads multiple words from the stream.
    ///
    /// Pass [`None`] for `count` to read until the end of the stream.
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

    /// Reads a [`string`][1].
    ///
    /// [1]: CompilerStr
    #[inline]
    pub fn read_string(&mut self) -> ParseResult<CompilerStr<'a>> {
        let pos = self.pos;
        if pos == self.words.len() { Err(ParseError::EndOfStream) }
        else {
            let str = CompilerStr::new(&self.words[pos..]);
            let div = str.len() / 4;
            let mul = 4 * div;
            self.pos = pos + div + if str.len() - mul != 0 { 1 } else { 0 };
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
