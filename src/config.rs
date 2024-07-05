//! The user interface.
//!
//! All these types can be freely created from anywhere without restriction.
//! They must be converted into internal types to continue with program
//! execution which will reject certain states in the process.

use crate::error::{Error::ConfigParse, Result};
#[cfg(test)]
use quickcheck::Arbitrary;
use serde::Deserialize;
use std::fmt;
#[cfg(test)]
use Requirement::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub struct Schema {
    pub delim: String,
    pub blocks: Vec<Block>,
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
            blocks: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let cats = self
            .blocks
            .shrink()
            .map(|categories| Schema {
                delim: self.delim.clone(),
                blocks: categories,
            })
            .collect::<Vec<_>>();

        let delims = self.delim.shrink().map(|delim| Schema {
            delim,
            blocks: self.blocks.clone(),
        });

        let mut all = cats;
        all.extend(delims);

        Box::new(all.into_iter())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub enum Block {
    Category(Category),
    Salt(Salt),
}

#[cfg(test)]
impl Arbitrary for Block {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if Arbitrary::arbitrary(g) {
            Block::Category(Arbitrary::arbitrary(g))
        } else {
            Block::Salt(Arbitrary::arbitrary(g))
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let i = match self {
            Block::Category(x) => x
                .shrink()
                .map(Block::Category)
                .collect::<Vec<_>>()
                .into_iter(),
            Block::Salt(x) => x.shrink().map(Block::Salt).collect::<Vec<_>>().into_iter(),
        };

        Box::new(i)
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
                    rvalue: if self.rvalue == 0 { 0 } else { self.rvalue - 1 },
                    values,
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub struct Salt {
    pub rtype: Requirement,
    pub rvalue: usize,
    pub values: String,
}

#[cfg(test)]
impl Arbitrary for Salt {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Salt {
            rtype: Arbitrary::arbitrary(g),
            // use the number of digits in the size (one less to include zero)
            // so tests aren't generating salts with billions of digits.
            rvalue: g.size().checked_ilog10().unwrap_or(0) as usize,
            values: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let i = self
            .values
            .shrink()
            .map(|s| Salt {
                rtype: self.rtype,
                rvalue: self.rvalue.shrink().next().unwrap_or(self.rvalue),
                values: s,
            })
            .collect::<Vec<_>>()
            .into_iter();
        Box::new(i)
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
    let schema: Schema = serde_dhall::from_str(contents)
        .parse()
        .map_err(|e| ConfigParse(Box::new(e)))?;
    Ok(schema)
}
