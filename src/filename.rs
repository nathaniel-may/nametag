use crate::{
    app::UiBlock,
    schema::{
        self,
        Requirement::{self, *},
        Schema,
    },
};
use core::fmt;
use rand::distributions::{Distribution, Uniform};
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
    state: &[UiBlock],
) -> Result<String, GenerateFilenameError> {
    let mut output = String::new();
    for block in state {
        match block {
            UiBlock::Salt { value, .. } => {
                output.push_str(value);
                output.push_str(schema.delim())
            }
            UiBlock::Category { name, values } => {
                let req = schema
                    .get_category_requirements(name)
                    // since States are generated from Schemas, this should be safe
                    .unwrap();

                let tags: Vec<String> = values
                    .iter()
                    .filter_map(|(tag, tf)| if *tf { Some(tag.clone()) } else { None })
                    .collect();

                match req {
                    expected @ Exactly(n) if tags.len() != n => Err(RequirementMismatch {
                        category_name: name.clone(),
                        expected: (expected, n),
                        selected: tags.len(),
                    }),
                    expected @ AtMost(n) if tags.len() > n => Err(RequirementMismatch {
                        category_name: name.clone(),
                        expected: (expected, n),
                        selected: tags.len(),
                    }),
                    expected @ AtLeast(n) if tags.len() < n => Err(RequirementMismatch {
                        category_name: name.clone(),
                        expected: (expected, n),
                        selected: tags.len(),
                    }),
                    _ => {
                        if tags.is_empty() {
                            output.push_str(schema.delim())
                        }
                        for tag in tags {
                            output.push_str(&tag);
                            output.push_str(schema.delim())
                        }
                        Ok(())
                    }
                }?;
            }
        }
    }

    // remove the last delimeter added
    for _ in schema.delim().chars() {
        output.pop();
    }
    Ok(output)
}

pub fn gen_salt(salt: &schema::Salt, rng: &mut ChaCha8Rng) -> String {
    let mut output = String::new();
    let chars = salt.values().chars().collect::<Vec<char>>();

    let n = match salt.req() {
        Requirement::Exactly(n) => n,
        Requirement::AtLeast(n) => n,
        Requirement::AtMost(n) => n,
    };

    if n == 0 {
        String::new()
    } else {
        for _ in 0..n {
            let i = Uniform::new(0, chars.len()).sample(rng);
            output.push(*chars.get(i).unwrap());
        }

        output
    }
}
