use crate::app::State;
use crate::error::{Error::ConfigParse, Result};
#[cfg(test)]
use quickcheck::Arbitrary;
use serde::Deserialize;
use std::fmt;
use std::result::Result as StdResult;
#[cfg(test)]
use Requirement::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilenameParseError {}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub struct Schema {
    pub delim: String,
    pub categories: Vec<Category>,
}

impl Schema {
    fn parse(&self, input: &str) -> StdResult<State, FilenameParseError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod prop_tests {
    use crate::{app::State, filename::gen_salt};

    use super::Schema;
    use quickcheck::{Gen, QuickCheck};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    // schemas should be able to parse the filenames they generate
    // TODO this does not include the salt and it should
    #[test]
    fn parse_generated_schemas() {
        fn closed_loop(schema: Schema, state: State, seed: u64) -> bool {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            // replace the arbitrary salt, so it can be used to compare
            let mut state = state;
            state.salt = gen_salt(&mut rng);

            match crate::filename::selection_to_filename(&schema, &state) {
                Err(_) => false,
                Ok(filename) => match schema.parse(&filename) {
                    Err(_) => false,
                    Ok(parsed_state) => parsed_state == state,
                },
            }
        }

        QuickCheck::new()
            .gen(Gen::new(5))
            .quickcheck(closed_loop as fn(Schema, State, u64) -> bool);
    }
}

#[cfg(test)]
impl Arbitrary for Schema {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut delim = char::arbitrary(g).to_string();
        if bool::arbitrary(g) {
            delim.push(char::arbitrary(g))
        }

        Schema {
            delim,
            categories: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.categories
                .shrink()
                .map(|categories| Schema {
                    // two char delims will cause different problems than single char delims. don't shrink.
                    delim: self.delim.clone(),
                    categories,
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub struct Category {
    pub name: String,
    pub rtype: Requirement,
    pub rvalue: usize,
    pub values: Vec<String>,
}

#[cfg(test)]
impl Arbitrary for Category {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Category {
            name: Arbitrary::arbitrary(g),
            rtype: Arbitrary::arbitrary(g),
            rvalue: *g.choose(&[0, 1, 2, 3]).unwrap(),
            values: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.values
                .shrink()
                .map(|values| Category {
                    name: self.name.shrink().next().unwrap_or(self.name.clone()),
                    rtype: self.rtype,
                    // relying on usize overflow semantics such that 0_usize - 1 == 0
                    rvalue: self.rvalue - 1,
                    values,
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize)]
pub enum Requirement {
    Exactly,
    AtLeast,
    AtMost,
}

#[cfg(test)]
impl Arbitrary for Requirement {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        *g.choose(&[Exactly, AtLeast, AtMost]).unwrap()
    }

    // no way to shrink this value
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exactly => write!(f, "exactly"),
            Self::AtLeast => write!(f, "at least"),
            Self::AtMost => write!(f, "at most"),
        }
    }
}

pub fn parse_schema(contents: &str) -> Result<Schema> {
    serde_dhall::from_str(contents)
        .parse()
        .map_err(|e| ConfigParse(Box::new(e)))
}

#[cfg(test)]
#[test]
fn init_config_file_parses() {
    use std::fs;
    use std::path::Path;

    use crate::schema::Category;
    use crate::schema::Requirement::*;

    let expected = Schema {
        delim: "-".to_string(),
        categories: vec![
            Category {
                name: "Medium".to_string(),
                rtype: Exactly,
                rvalue: 1,
                values: vec![
                    "art".to_string(),
                    "photo".to_string(),
                    "ai".to_string(),
                    "other".to_string(),
                ],
            },
            Category {
                name: "Subject".to_string(),
                rtype: AtLeast,
                rvalue: 0,
                values: vec![
                    "plants".to_string(),
                    "animals".to_string(),
                    "people".to_string(),
                ],
            },
        ],
    };

    match parse_schema(&fs::read_to_string(Path::new("./src/init.dhall")).unwrap()) {
        Err(e) => panic!("{e}"),
        Ok(schema) => assert_eq!(expected, schema),
    }
}
