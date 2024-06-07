use crate::error::{Error::ConfigParse, Result};
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
