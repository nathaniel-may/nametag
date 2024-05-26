pub mod parse;
pub mod typecheck;

use std::{error::Error as StdError, fmt};
use typecheck::Type;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Schema {
    pub delim: String,
    pub empty: String,
    pub categories: Vec<(Category, Vec<Keyword>)>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Category {
    pub name: String,
    pub requirement: Requirement,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Requirement {
    Exactly(u8),
    AtLeast(u8),
    AtMost(u8),
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exactly(n) => write!(f, "exactly {n}"),
            Self::AtLeast(n) => write!(f, "at least {n}"),
            Self::AtMost(n) => write!(f, "at most {n}"),
        }
    }
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchemaTypeCheckError {
    HeterogeneousList(Vec<Type>),
    TypeMismatch { expected: Type, got: Type },
    UnknownFunction { name: String, arg_types: Vec<Type> },
    ExpectedTopLevelSchema,
}

impl fmt::Display for SchemaTypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeterogeneousList(types) => {
                write!(
                    f,
                    "Heterogenous lists are not supported. Found elements of types {}",
                    display_types(types)
                )
            }
            Self::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch. Expected {expected}. Got {got}.")
            }
            Self::UnknownFunction { name, arg_types } => write!(
                f,
                "Unknown function \"{name}\" with arguments {}.",
                display_types(arg_types)
            ),
            Self::ExpectedTopLevelSchema => write!(f, "The top level value must be a schema."),
        }
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

fn display_types(types: &[Type]) -> String {
    let mut x = String::new();
    for t in types {
        x.push_str(&format!("{t}, "))
    }
    x.pop();
    x.pop();
    x
}
