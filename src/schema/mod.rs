pub mod parse;
pub mod typecheck;

use std::{error::Error as StdError, fmt};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Schema {
    delim: String,
    empty: String,
    categories: Vec<(Category, Vec<Keyword>)>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Category {
    pub name: String,
    pub id: String,
    pub requirement: Requirement,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Requirement {
    Exactly(usize),
    AtLeast(usize),
    AtMost(usize),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Keyword {
    pub name: String,
    pub id: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SchemaParseError {
    MustStartWithSchemaConstructor,
    UnexpectedInput(String),
}

impl fmt::Display for SchemaParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MustStartWithSchemaConstructor => {
                write!(f, "Expected \"schema\" constructor")
            }
            Self::UnexpectedInput(input) => write!(f, "Unexpected input: {input}"),
        }
    }
}

impl StdError for SchemaParseError {}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SchemaTypeCheckError {}

impl fmt::Display for SchemaTypeCheckError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl StdError for SchemaTypeCheckError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprU {
    KeywordU { name: String, id: String },
    StringU(String),
    FnU { name: String, args: Vec<ExprU> },
    ListU(Vec<ExprU>),
    NatU(u8),
}
