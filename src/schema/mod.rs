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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Requirement {
    Exactly(u8),
    AtLeast(u8),
    AtMost(u8),
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
    HeterogeneousListIsNotCoercable(Vec<Type>),
    TypeMismatch { expected: Type, got: Type },
    UnknownFunction(String),
    ExpectedTopLevelSchema,
}

impl fmt::Display for SchemaTypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeterogeneousListIsNotCoercable(types) => {
                let mut x = String::new();
                for t in types {
                    x.push_str(&format!("{t}, "))
                }
                x.pop();
                x.pop();
                write!(
                    f,
                    "Heterogenous list is not coercable. Found elements of types {x}"
                )
            }
            Self::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch. Expected {expected}. Got {got}.")
            }
            Self::UnknownFunction(name) => write!(f, "Unknown function \"{name}\"."),
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
