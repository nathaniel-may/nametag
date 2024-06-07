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
        expected: (Requirement, usize),
        selected: usize,
    },
}

impl fmt::Display for GenerateFilenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequirementMismatch {
                category,
                expected: (rtype, rvalue),
                selected,
            } => write!(
                f,
                "{} must have {rtype} {rvalue} tag but {selected} are selected.",
                category.name
            ),
        }
    }
}

impl StdError for GenerateFilenameError {}

pub fn generate(schema: &Schema, state: &State) -> Result<String, GenerateFilenameError> {
    let mut name = String::new();
    for (cat, kws) in state {
        let tags: Vec<String> = kws
            .iter()
            .filter_map(|(tag, tf)| if *tf { Some(tag.clone()) } else { None })
            .collect();
        match cat.rtype {
            expected @ Exactly if tags.len() != cat.rvalue => Err(RequirementMismatch {
                category: cat.clone(),
                expected: (expected, cat.rvalue),
                selected: tags.len(),
            }),
            expected @ AtMost if tags.len() > cat.rvalue => Err(RequirementMismatch {
                category: cat.clone(),
                expected: (expected, cat.rvalue),
                selected: tags.len(),
            }),
            expected @ AtLeast if tags.len() < (cat.rvalue) => Err(RequirementMismatch {
                category: cat.clone(),
                expected: (expected, cat.rvalue),
                selected: tags.len(),
            }),
            _ => {
                if tags.is_empty() {
                    name.push_str(&schema.delim)
                }
                for id in tags {
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
