use crate::{
    app::State,
    schema::{
        Requirement::{self, *},
        Schema,
    },
};
use core::fmt;
use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::error::Error as StdError;
use GenerateFilenameError::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerateFilenameError {
    RequirementMismatch {
        category_name: String,
        expected: (Requirement, usize),
        selected: usize,
    },
}

impl fmt::Display for GenerateFilenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequirementMismatch {
                category_name,
                expected: (rtype, rvalue),
                selected,
            } => write!(
                f,
                "{category_name} must have {rtype} {rvalue} tag but {selected} are selected."
            ),
        }
    }
}

impl StdError for GenerateFilenameError {}

pub fn selection_to_filename(
    schema: &Schema,
    state: &State,
) -> Result<String, GenerateFilenameError> {
    let mut name = state.salt.clone();
    name.push_str(&schema.delim);
    for cat in &state.categories[..] {
        let cat_def = schema
            .categories
            .iter()
            .find(|s_cat| s_cat.name == cat.name)
            // since States are generated from Schemas, this should be safe
            .unwrap();

        let tags: Vec<String> = cat
            .values
            .iter()
            .filter_map(|(tag, tf)| if *tf { Some(tag.clone()) } else { None })
            .collect();

        match cat_def.rtype {
            expected @ Exactly if tags.len() != cat_def.rvalue => Err(RequirementMismatch {
                category_name: cat.name.clone(),
                expected: (expected, cat_def.rvalue),
                selected: tags.len(),
            }),
            expected @ AtMost if tags.len() > cat_def.rvalue => Err(RequirementMismatch {
                category_name: cat.name.clone(),
                expected: (expected, cat_def.rvalue),
                selected: tags.len(),
            }),
            expected @ AtLeast if tags.len() < (cat_def.rvalue) => Err(RequirementMismatch {
                category_name: cat.name.clone(),
                expected: (expected, cat_def.rvalue),
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

pub fn gen_salt(rng: &mut ChaCha8Rng) -> String {
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
