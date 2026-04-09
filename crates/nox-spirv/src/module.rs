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

/// Indicates that the SPIR-V code passed to a [`module`][1] is invalid.
///
/// [1]: Module
#[derive(Debug)]
pub enum ParseError {
    EndOfStream,
    InvalidType {
         expected: &'static str,
         found: op::Code,
    },
    UnknownOpCode(op::Code),
    WrongOpCode {
        expected: &'static str,
        found: op::Code,
    },
    InvalidIdResult(op::IdRef),
    ExpectedTypeDeclaration(op::IdRef),
    ExpectedContextResultType,
    InvalidIntLiteralWidth(u32),
    InvalidFloatLiteralWidth(u32),
    UnknownVariant {
        kind: &'static str,
        value: u32,
    },
    InvalidPairWordCount(u32),
}

#[derive(Clone)]
pub struct ResultStorage<'a> {
    data: Vec<Option<InstructionStream<'a>>>,
}

impl Default for ResultStorage<'_> {

    #[inline]
    fn default() -> Self {
        Self {
            data: vec![],
        }
    }
}

impl<'a> ResultStorage<'a>
{

    #[inline]
    pub fn insert(&mut self,
        id: op::IdResult,
        mut stream: InstructionStream<'a>
    )
    {
        stream.reset();
        let idx = id.0 as usize;
        if idx >= self.data.len() {
            self.data.resize_with(idx + 1, || None);
        }
        self.data[idx] = Some(stream);
    }

    #[inline]
    pub fn get(&self, id: op::IdRef) -> Option<InstructionStream<'a>> {
        self.data.get(id.0 as usize)
            .and_then(|&value| value)
    }

    #[inline]
    pub fn contains(&self, id: op::IdRef) -> bool {
        self.data.get(id.0 as usize)
            .and_then(|value| value.as_ref())
            .is_some()
    }
}

#[derive(Clone)]
pub struct Module<'a> {
    stream: Stream<'a>,
    results: ResultStorage<'a>,
}

pub type ParseResult<T> = Result<T, ParseError>;

impl<'a> Module<'a> {

    #[inline]
    pub fn new(spirv: &'a [u32]) -> Self {
        Self {
            stream: Stream::new(spirv),
            results: Default::default(),
        }
    }

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
            );
        }
        Ok(true)
    }

    #[inline]
    pub fn parse_full(&mut self) -> ParseResult<()> {
        while self.parse_next()? {}
        Ok(())
    }

    #[inline]
    pub fn all_instructions(&self) -> impl Iterator<Item = InstructionStream<'a>> {
        let mut stream = self.stream;
        stream.reset();
        stream
    }

    #[inline]
    pub fn remaining_instructions(&self) -> impl Iterator<Item = InstructionStream<'a>> {
        self.stream
    }

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

    #[inline]
    pub fn get_result(&self, id: op::IdRef) -> Option<InstructionStream<'a>> {
        self.results.get(id)
    }
}

#[derive(Default, Clone, Copy)]
pub struct ParseContext {
    pub result_type: Option<op::IdResultType>,
}

pub trait WordExt<'a>: Word {

    #[inline]
    fn parse_one(
        stream: &mut InstructionStream<'a>,
    ) -> ParseResult<Self> {
        Ok(Self::from_word(stream.read()?))
    }

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

    #[inline]
    pub fn parse_one(
        stream: &mut InstructionStream<'a>,
    ) -> ParseResult<Self> {
        stream.read_string()
    }

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
            Self::InvalidType { expected, found } => write!(f,
                "invalid type; expected {expected}, found {found}",
            ),
            Self::UnknownOpCode(code) => write!(f,
                "unknown op code {code}"
            ),
            Self::WrongOpCode { expected, found } => write!(f,
                "wrong op code; expected {expected}, found {found}",
            ),
            Self::InvalidIdResult(id) => write!(f,
                "invalid id result {id}",
            ),
            Self::ExpectedTypeDeclaration(id) => write!(f,
                "expected type declaration, found id {id}",
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
