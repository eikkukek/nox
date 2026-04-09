//! SPIR-V common types
use core::{
    fmt::{self, Display, Debug},
    slice,
};
use crate::{
    core::*,
    stream::*,
    module::*,
};
use super::*;
/// Reference to an <id> representing the result's type of the enclosing instruction.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IdResultType(pub(crate) u32);
impl Display for IdResultType {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}
impl Word for IdResultType {

    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
/// Reference to an <id> representing the result's type of the enclosing instruction.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IdResult(pub(crate) u32);
impl Display for IdResult {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}
impl Word for IdResult {

    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
/// Reference to an <id> representing a 32-bit integer that is a mask from the MemorySemantics
/// operand kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IdMemorySemantics(pub(crate) u32);
impl Display for IdMemorySemantics {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}
impl Word for IdMemorySemantics {

    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
/// Reference to an <id> representing a 32-bit integer that is a mask from the Scope operand kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IdScope(pub(crate) u32);
impl Display for IdScope {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}
impl Word for IdScope {

    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
/// Reference to an <id>.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IdRef(pub(crate) u32);
impl Display for IdRef {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}
impl Word for IdRef {

    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word)
    }
}
impl From<IdResultType> for IdRef {

    #[inline]
    fn from(value: IdResultType) -> Self {
        Self(value.0)
    }
}
impl From<IdResult> for IdRef {

    #[inline]
    fn from(value: IdResult) -> Self {
        Self(value.0)
    }
}
/// An integer consuming one or more words.
pub type LiteralInteger = u32;
/// A null-terminated stream of characters consuming an integral number of words.
pub type LiteralString<'a> = CompilerStr<'a>;
/// A float consuming one word.
pub type LiteralFloat = f32;
impl Word for LiteralFloat {

    #[inline]
    fn from_word(word: u32) -> Self {
        Self::from_bits(word)
    }
}
/// A literal number whose size and format are determined by a previous operand in the enclosing
/// instruction.
pub type LiteralContextDependentNumber = Literal;
impl<'a> LiteralContextDependentNumber {

    pub fn parse_one(
        module: &Module<'a>,
        stream: &mut InstructionStream<'a>,
        ctx: &mut ParseContext,
    ) -> ParseResult<Self> {
        let Some(result_type) = ctx.result_type else {
            return Err(ParseError::ExpectedContextResultType)
        };
        let mut ty_stream = module
            .get_result(result_type.into())
            .ok_or(ParseError::InvalidIdResult(result_type.into()))?;
        match ty_stream.code() {
            Code::TYPE_INT => {
                let int = InstTypeInt {
                    id_result: IdResult::parse_one(&mut ty_stream)?,
                    width: ty_stream.read()?,
                    signedness: ty_stream.read()?,
                };
                Ok(if int.signedness == 1 {
                    match int.width {
                        8 => Literal::I8(u8::cast_signed(
                            stream.read()? as u8
                        )),
                        16 => Literal::I16(u16::cast_signed(
                            stream.read()? as u16,
                        )),
                        32 => Literal::I32(u32::cast_signed(
                            stream.read()?,
                        )),
                        64 => Literal::I64(u64::cast_signed(
                            stream.read()? as u64 |
                            ((stream.read()? as u64) << 32)
                        )),
                        x => return Err(ParseError
                            ::InvalidIntLiteralWidth(x)
                        )
                    }
                } else {
                    match int.width {
                        8 => Literal::U8(stream.read()? as u8),
                        16 => Literal::U16(stream.read()? as u16),
                        32 => Literal::U32(stream.read()?),
                        64 => Literal::U64(
                            stream.read()? as u64 |
                            ((stream.read()? as u64) << 32)
                        ),
                        x => return Err(ParseError
                            ::InvalidIntLiteralWidth(x)
                        )
                    }
                })
            },
            Code::TYPE_FLOAT => {
                let float = InstTypeFloat {
                    id_result: IdResult::parse_one(&mut ty_stream)?,
                    width: ty_stream.read()?,
                    floating_point_encoding: FPEncoding::parse_optional(&mut ty_stream)?,
                };
                Ok(match float.width {
                    16 => Literal::F16(stream.read()? as u16),
                    32 => Literal::F32(f32::from_bits(stream.read()?)),
                    64 => Literal::F64(f64::from_bits(
                        stream.read()? as u64 |
                        ((stream.read()? as u64) << 32)
                    )),
                    x => return Err(ParseError
                        ::InvalidFloatLiteralWidth(x)
                    )
                })
            },
            x => Err(ParseError::InvalidType {
                expected: "float or int type",
                found: x,
            })
        }
    }
}
/// A 32-bit unsigned integer indicating which instruction to use and determining the layout
/// of following operands (for [`InstExtInst`]).
pub type LiteralExtInstInteger = u32;
/// An opcode indicating the operation to be performed and determining the layout of following
/// operands (for [`InstSpecConstantOp`]).
#[derive(Clone, Copy, Debug)]
pub struct LiteralSpecConstantOpInteger<'a> {
    pub code: Code,
    pub operands: &'a [IdRef],
}
impl<'a> LiteralSpecConstantOpInteger<'a> {

    pub fn parse_one(
        stream: &mut InstructionStream<'a>,
    ) -> ParseResult<Self>
    {
        Ok(Self {
            code: Code::from_word(stream.read()?),
            operands: IdRef::parse_eos(stream)?,
        })
    }
}
impl<'a> Display for LiteralSpecConstantOpInteger<'a> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)?;
        for operand in self.operands {
            write!(f, " {operand}")?;
        }
        Ok(())
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Pair<T, U>(T, U)
    where
        T: Word + Debug + Display,
        U: Word + Debug + Display;
impl<T, U> Display for Pair<T, U>
    where
        T: Word + Debug + Display,
        U: Word + Debug + Display,
{

    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}
impl<'a, T, U> Pair<T, U>
    where
        T: Word + Debug + Display,
        U: Word + Debug + Display,
{

    #[inline]
    pub fn parse_one(
        stream: &mut InstructionStream<'a>,
    ) -> ParseResult<Self> {
        Ok(Self(T::from_word(stream.read()?), U::from_word(stream.read()?)))
    }

    #[inline]
    pub fn parse_eos(
        stream: &mut InstructionStream<'a>,
    ) -> ParseResult<&'a [Self]> {
        let words = stream.read_words(None)?;
        if !words.len().is_multiple_of(2) {
            return Err(ParseError::InvalidPairWordCount(words.len() as u32))
        }
        unsafe {
            Ok(slice::from_raw_parts(
                words.as_ptr().cast(),
                words.len() / 2
            ))
        }
    }
}
pub type PairLiteralIntegerIdRef = Pair<LiteralInteger, IdRef>;
pub type PairIdRefLiteralInteger = Pair<IdRef, LiteralInteger>;
pub type PairIdRefIdRef = Pair<IdRef, IdRef>;
#[derive(Default, Clone, Copy)]
pub struct InstInfo {
    pub name: &'static str,
    pub has_id_result_type: bool,
    pub has_id_result: bool,
}
