#![warn(missing_docs)]

use core::{
    fmt::{Display, self},
    error::Error,
    slice,
};

use crate::{
    stream::*,
    op,
    core::*,
};

/// SPIR_V version 1.0
pub const VERSION_1_0: u32 = 0x00010000;
/// SPIR_V version 1.1
pub const VERSION_1_1: u32 = 0x00010100;
/// SPIR_V version 1.2
pub const VERSION_1_2: u32 = 0x00010200;
/// SPIR_V version 1.3
pub const VERSION_1_3: u32 = 0x00010300;
/// SPIR_V version 1.4
pub const VERSION_1_4: u32 = 0x00010400;
/// SPIR_V version 1.5
pub const VERSION_1_5: u32 = 0x00010500;
/// SPIR_V version 1.6
pub const VERSION_1_6: u32 = 0x00010600;

/// Indicates that the SPIR-V code passed to a [`module`][1] is invalid.
///
/// [1]: Module
#[derive(Debug)]
pub enum ParseError {
    /// Indicates that the end of the stream was reached prematurely.
    EndOfStream,
    /// Indicates a wrong magic number for the module.
    InvalidMagicNumber(u32),
    /// Indicates an invalid type.
    InvalidType {
        /// The expected type.
        expected: &'static str,
        /// The found [`Code`][1].
        ///
        /// [1]: op::Code
        found: op::Code,
    },
    /// Indicates an invalid result id.
    InvalidIdResult(op::IdRef),
    /// Indicates that a [`context`][1] result type was expected, but [`None`] was found.
    ///
    /// [1]: ParseContext
    ExpectedContextResultType,
    /// Indicates that an invalid integer literal width was found.
    InvalidIntLiteralWidth(u32),
    /// Indicates that an invalid floating point literal width was found.
    InvalidFloatLiteralWidth(u32),
    /// Indicates that an id in SPIR-V was out of the set bounds.
    IdOutOfBounds {
        /// The set bound.
        bound: u32,
        /// The id.
        id: op::IdResult,
    },
    /// Indicates that an unknown variant was found.
    UnknownVariant {
        /// The enum type.
        kind: &'static str,
        /// The found value.
        value: u32,
    },
    /// Indicates that an invalid pair word count was found (not a multiple of 2).
    InvalidPairWordCount(u32),
}

#[derive(Clone)]
struct ResultStorage<'a> {
    data: Vec<Option<InstructionStream<'a>>>,
}

impl<'a> ResultStorage<'a>
{

    #[inline]
    fn new(bound: u32) -> Self {
        Self {
            data: vec![None; bound as usize],
        }
    }

    #[inline]
    fn insert(&mut self,
        id: op::IdResult,
        mut stream: InstructionStream<'a>
    ) -> ParseResult<()>
    {
        stream.reset();
        let idx = id.0 as usize;
        if idx >= self.data.len() {
            return Err(ParseError::IdOutOfBounds {
                bound: self.data.len() as u32,
                id
            })
        }
        self.data[idx] = Some(stream);
        Ok(())
    }

    #[inline]
    fn get(&self, id: op::IdRef) -> Option<InstructionStream<'a>> {
        self.data.get(id.0 as usize)
            .and_then(|&value| value)
    }
}

/// Represents a SPIR-V module.
#[derive(Clone)]
pub struct Module<'a> {
    stream: Stream<'a>,
    results: ResultStorage<'a>,
}

/// The [`Result`] from a parsing operation,
pub type ParseResult<T> = Result<T, ParseError>;

impl<'a> Module<'a> {

    /// Creates a new module from SPIR-V words.
    #[inline]
    pub fn new(spirv: &'a [u32]) -> ParseResult<Self> {
        let stream = Stream::new(spirv)?;
        Ok(Self {
            results: ResultStorage::new(stream.bound()),
            stream,
        })
    }

    /// The SPIR-V version of the module.
    #[inline]
    pub fn version(&self) -> u32 {
        self.stream.version()
    }

    /// Tries to parse the next instruction.
    #[inline]
    pub fn parse_next(&mut self) -> ParseResult<bool> {
        let Some(mut instruction) = self.stream
            .next()
        else {
            return Ok(false)
        };
        let inst_info = op::INST_INFOS[instruction.code().0 as usize];
        if inst_info.has_id_result {
            if inst_info.has_id_result_type {
                let _ = instruction.read()?;
            }
            let id = op::IdResult(instruction.read()?);
            self.results.insert(
                id,
                instruction,
            )?;
        }
        Ok(true)
    }

    /// Tries to parse until the end of the inner [`Stream`].
    #[inline]
    pub fn parse_full(&mut self) -> ParseResult<()> {
        while self.parse_next()? {}
        Ok(())
    }

    /// Returns an iterator over all [`instruction streams`][1].
    ///
    /// [1]: InstructionStream
    #[inline]
    pub fn all_instructions(&self) -> impl Iterator<Item = InstructionStream<'a>> {
        let mut stream = self.stream;
        stream.reset();
        stream
    }
    
    /// Returns an iterator over all remaining unparsed [`instruction streams`][1].
    ///
    /// [1]: InstructionStream
    #[inline]
    pub fn remaining_instructions(&self) -> impl Iterator<Item = InstructionStream<'a>> {
        self.stream
    }

    /// Returns an iterator over all cached [`instruction streams`][1] with a result.
    ///
    /// [1]: InstructionStream
    #[inline]
    pub fn results(&self) -> impl Iterator<Item = (op::IdRef, InstructionStream<'a>)> {
        self.results.data
            .iter()
            .enumerate()
            .filter_map(|(i, result)| {
                result
                    .map(|result| (op::IdRef::from_word(i as u32), result))
            })
    }

    /// Gets a result with `id`, returning [`None`] if the result is not present.
    #[inline]
    pub fn get_result(&self, id: op::IdRef) -> Option<InstructionStream<'a>> {
        self.results.get(id)
    }
}

/// A parse context used when parsing a [`Literal`].
#[derive(Default, Clone, Copy)]
pub struct ParseContext {
    /// The optional [`result type`][1].
    ///
    /// [1]: op::IdResultType
    pub result_type: Option<op::IdResultType>,
}

/// An extension auto-trait for [`Word`].
pub trait WordExt<'a>: Word {

    /// Parses one word.
    #[inline]
    fn parse_one(
        stream: &mut InstructionStream<'a>,
    ) -> ParseResult<Self> {
        Ok(Self::from_word(stream.read()?))
    }

    /// Tries to optionally parse one word.
    #[inline]
    fn parse_optional(
        stream: &mut InstructionStream<'a>,
    ) -> ParseResult<Option<Self>> {
        if stream.is_eos() {
            Ok(None)
        } else {
            Ok(Some(Self::from_word(stream.read()?)))
        }
    }

    /// Parses words until the end of the stream.
    #[inline]
    fn parse_eos(
        stream: &mut InstructionStream<'a>,
    ) -> ParseResult<&'a [Self]> {
        let words = stream.read_words(None)?;
        Ok(unsafe {
            slice::from_raw_parts(
                words.as_ptr().cast(),
                words.len(),
            )
        })
    }
}

impl<'a, T: Word> WordExt<'a> for T {}

impl<'a> CompilerStr<'a> {

    /// Parses a string.
    #[inline]
    pub fn parse_one(
        stream: &mut InstructionStream<'a>,
    ) -> ParseResult<Self> {
        stream.read_string()
    }

    /// Tries to optionally parse a string.
    #[inline]
    pub fn parse_optional(
        stream: &mut InstructionStream<'a>,
    ) -> ParseResult<Option<Self>> {
        if stream.is_eos() {
            Ok(None)
        } else {
            Ok(Some(stream.read_string()?))
        }
    }
}

impl Display for ParseError {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndOfStream => write!(f, "end of stream"),
            Self::InvalidMagicNumber(num) => write!(f, "invalid magic number {num}, expected 0x07230203"),
            Self::InvalidType { expected, found } => write!(f,
                "invalid type; expected {expected}, found {found}",
            ),
            Self::InvalidIdResult(id) => write!(f,
                "invalid id result {id}",
            ),
            Self::ExpectedContextResultType => write!(f,
                "expected context result type, found None",
            ),
            Self::InvalidIntLiteralWidth(width) => write!(f,
                "invalid int literal width {width}",
            ),
            Self::InvalidFloatLiteralWidth(width) => write!(f,
                "invalid float literal width {width}",
            ),
            Self::IdOutOfBounds { bound, id } => write!(f,
                "Id {id} is out of the set bound {bound}",
            ),
            Self::UnknownVariant { kind, value } => write!(f,
                "unknown {kind} variant {value}",
            ),
            Self::InvalidPairWordCount(count) => write!(f,
                "invalid pair word count {count}: count is not a multiple of 2"
            ),
        }
    }
}

impl Error for ParseError {}
