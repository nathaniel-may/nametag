pub mod parse;

use crate::schema::{
    Category,
    Requirement::{self, *},
    Schema,
};
use crate::State;
use core::fmt;
use rand::Rng;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
};
use std::error::Error as StdError;
use GenerateFilenameError::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerateFilenameError {
    RequirementMismatch {
        category: Category,
        expected: Requirement,
        got: usize,
    },
}

impl fmt::Display for GenerateFilenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequirementMismatch { category, expected, got } => write!(f, "Category {} has a tag requirement of {expected}, but there were {got} keywords found.", category.name)
        }
    }
}

impl StdError for GenerateFilenameError {}

pub fn generate(schema: &Schema, state: &State) -> Result<String, GenerateFilenameError> {
    let mut name = String::new();
    for (cat, kws) in state {
        let ids: Vec<String> = kws
            .iter()
            .filter_map(|(kw, tf)| if *tf { Some(kw.id.clone()) } else { None })
            .collect();
        match cat.requirement {
            expected @ Exactly(n) if ids.len() != (n as usize) => Err(RequirementMismatch {
                category: cat.clone(),
                expected,
                got: ids.len(),
            }),
            expected @ AtMost(n) if ids.len() > (n as usize) => Err(RequirementMismatch {
                category: cat.clone(),
                expected,
                got: ids.len(),
            }),
            expected @ AtLeast(n) if ids.len() < (n as usize) => Err(RequirementMismatch {
                category: cat.clone(),
                expected,
                got: ids.len(),
            }),
            _ => {
                if ids.is_empty() {
                    name.push_str(&schema.empty);
                    name.push_str(&schema.delim)
                }
                for id in ids {
                    name.push_str(&id);
                    name.push_str(&schema.delim)
                }
                Ok(())
            }
        }?;
    }

    // remove the last delimeter added
    name.pop();
    Ok(name)
}

pub fn gen_rand_id(rng: &mut ThreadRng) -> String {
    (0..6)
        .map(|_| rng.sample(IDChars) as char)
        .collect::<String>()
}

struct IDChars;

impl Distribution<u8> for IDChars {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u8 {
        const RANGE: usize = 25 + 9;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNPQRSTUVWXYZ123456789";
        let range = Uniform::new(0, RANGE);
        CHARSET[range.sample(rng)]
    }
}
