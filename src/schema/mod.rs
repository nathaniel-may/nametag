use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub struct Schema {
    pub delim: String,
    pub categories: Vec<Category>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub struct Category {
    pub name: String,
    pub rtype: Requirement,
    pub rvalue: usize,
    pub values: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize)]
pub enum Requirement {
    Exactly,
    AtLeast,
    AtMost,
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exactly => write!(f, "Exactly"),
            Self::AtLeast => write!(f, "AtLeast"),
            Self::AtMost => write!(f, "AtMost"),
        }
    }
}
