use super::file_pos::FilePos;
use std::fmt::Debug;

// Input text to be parsed
#[derive(Debug, Clone, Copy)]
pub struct ParserInput<'a>
{
    pub text : &'a str,
    pub pos :  FilePos,
}

impl<'a> ParserInput<'a>
{
    pub fn new(to_parse : &'a str) -> ParserInput<'a>
    {
        ParserInput {
            text : to_parse,
            pos :  FilePos {
                line : 1, column : 0
            },
        }
    }
}

// Parser result data
pub trait PResData = Debug + Clone + PartialEq + Eq;

// Parser result (success)
#[derive(Debug, Clone, PartialEq)]
pub struct PRes<'a, DatT : PResData>
{
    pub val :       DatT,
    pub pos :       FilePos,
    pub remainder : &'a str,
}

// Parser error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PErr
{
    pub pos : FilePos,
}

// Paser out type
pub type POut<'a, DatT> = Result<PRes<'a, DatT>, PErr>;

pub fn get_p_out_pos<'a, DatT : PResData>(pout : &POut<'a, DatT>) -> FilePos
{
    match pout
    {
        Ok(p_succ) => p_succ.pos,
        Err(p_err) => p_err.pos,
    }
}

pub trait Parser<'a, DatT : PResData> = Fn(&ParserInput<'a>) -> POut<'a, DatT> + Clone;

pub trait Predicate<'a> = Fn(&'a str) -> bool + Clone;

impl<'a, DatT : PResData> PRes<'a, DatT>
{
    pub fn to_in(&self) -> ParserInput<'a>
    {
        ParserInput {
            pos :  self.pos,
            text : self.remainder,
        }
    }

    pub fn with_val<DatTOut : PResData>(&self, dat_out : DatTOut) -> PRes<'a, DatTOut>
    {
        PRes {
            val :       dat_out,
            pos :       self.pos,
            remainder : self.remainder,
        }
    }
}
