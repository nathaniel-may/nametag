use crate::app::UiBlock;
use crate::config;
use crate::error::Error;
use crate::error::Result;
use crate::util::NametagIterExt;
#[cfg(test)]
use quickcheck::Arbitrary;
use std::collections::HashSet;
use std::fmt;
use std::result::Result as StdResult;
use FilenameParseError::*;
#[cfg(test)]
use Requirement::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilenameParseError {
    UnexpectedTag(String),
    MissingSalt,
    FailedToParseSalt(String),
    MisplacedDelim,
}

impl fmt::Display for FilenameParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnexpectedTag(tag) => write!(f, "Unexpected tag: {tag}"),
            MissingSalt => write!(f, "Missing salt"),
            FailedToParseSalt(salt) => write!(f, "Failed to parse salt. Found {salt}"),
            MisplacedDelim => write!(
                f,
                "Misplaced delimiter found. All delimiters must be seperating two values."
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Schema {
    delim: String,
    blocks: Vec<Block>,
}

impl Schema {
    pub fn requires_salt(&self) -> bool {
        self.blocks.iter().any(|x| matches!(x, Block::Salt(_)))
    }

    pub fn get_category_requirements(&self, name: &str) -> Option<Requirement> {
        self.blocks.iter().find_map(|x| match x {
            Block::Category(cat) => {
                if cat.name == name {
                    Some(cat.req())
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    fn char_allowed(c: char) -> bool {
        // no control characters. They can't all be read back after being written. // TODO is that actually right?
        (c as u32) >= 32 && !['\0'].contains(&c)
    }

    // This is the only way to go from the input type config::Schema to the internal state of schema::Schema.
    pub fn from_config(config: config::Schema) -> Result<Schema> {
        if config.delim.is_empty() {
            return Err(Error::EmptyDelimiter);
        }
        if config.blocks.is_empty() {
            return Err(Error::NoBlocks);
        }
        for c in config.delim.chars() {
            if !Schema::char_allowed(c) {
                return Err(Error::InvalidCharacterInDelim(c));
            }
        }

        let mut m: HashSet<&str> = HashSet::new();

        for block in &config.blocks {
            match block {
                config::Block::Salt(config::Salt { rtype, rvalue, .. }) => match rtype {
                    config::Requirement::AtMost => {
                        return Err(Error::SaltDefinitionMustExcludeEmptySalts)
                    }
                    config::Requirement::Exactly | config::Requirement::AtLeast if *rvalue == 0 => {
                        return Err(Error::SaltDefinitionMustExcludeEmptySalts)
                    }
                    _ => (),
                },
                config::Block::Category(config::Category {
                    name,
                    rtype,
                    rvalue,
                    values,
                }) => {
                    if values.is_empty() {
                        return Err(Error::CategoryWithNoTags {
                            category_name: name.clone(),
                        });
                    }
                    if values.contains(&String::new()) {
                        return Err(Error::EmptyStringNotValidTag);
                    }

                    for v in values {
                        if !m.insert(v) {
                            return Err(Error::TagsMustBeUnique {
                                category_name: name.clone(),
                                duplicated_tag: v.clone(),
                            });
                        }
                        if v.contains(&config.delim) {
                            return Err(Error::DelimiterFoundInTag {
                                category_name: name.clone(),
                                tag: v.clone(),
                            });
                        }
                        for c in v.chars() {
                            if !Schema::char_allowed(c) {
                                return Err(Error::InvalidCharacterInTag(c));
                            }
                        }
                    }
                }
            }
        }

        let mut blocks: Vec<Block> = Vec::with_capacity(config.blocks.len());
        for block in config.blocks {
            match block {
                config::Block::Salt(config::Salt {
                    rtype,
                    rvalue,
                    values,
                }) => {
                    let salt = Salt {
                        req: (rtype, rvalue).into(),
                        values,
                    };
                    blocks.push(Block::Salt(salt));
                }
                config::Block::Category(config::Category {
                    name,
                    rtype,
                    rvalue,
                    values,
                }) => {
                    let cat = Category {
                        name,
                        req: (rtype, rvalue).into(),
                        values,
                    };
                    blocks.push(Block::Category(cat));
                }
            }
        }
        let schema = Schema {
            delim: config.delim,
            blocks,
        };
        Ok(schema)
    }

    pub fn delim(&self) -> &str {
        self.delim.as_str()
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn parse(&self, input: &str) -> StdResult<Vec<UiBlock>, FilenameParseError> {
        let tags = input.split(&self.delim).collect::<Vec<_>>();
        if tags.contains(&"") && !input.is_empty() {
            return Err(MisplacedDelim);
        }
        let mut tags = tags.into_iter().peekable();

        let mut blocks = Vec::with_capacity(self.blocks.len());
        for block in &self.blocks[..] {
            match block {
                Block::Salt(Salt { req, values }) => {
                    let maybe_salt = tags.next().ok_or(MissingSalt)?;
                    let drained: String = maybe_salt
                        .chars()
                        .peekable()
                        .drain_while(|&x| values.contains([x]))
                        .iter()
                        .collect();
                    if maybe_salt == drained {
                        blocks.push(UiBlock::Salt {
                            value: maybe_salt.into(),
                            definition: Salt {
                                req: *req,
                                values: values.clone(),
                            },
                        });
                    } else {
                        return Err(FailedToParseSalt(maybe_salt.into()));
                    }
                }
                Block::Category(Category { name, req, values }) => {
                    let applied_tags = tags.drain_while(|tag| values.contains(&tag.to_string()));

                    let values = values
                        .clone()
                        .into_iter()
                        .map(|name| (name.clone(), applied_tags.contains(&name.as_str())))
                        .collect();

                    blocks.push(UiBlock::Category {
                        name: name.clone(),
                        values,
                    });
                }
            }
        }

        match tags.next() {
            // Some("") happens when the filename is completely empty
            None | Some("") => Ok(blocks),
            Some(tag) => Err(FilenameParseError::UnexpectedTag(tag.into())),
        }
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Category {
    name: String,
    req: Requirement,
    values: Vec<String>,
}

impl Category {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn req(&self) -> Requirement {
        self.req
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }
}

#[cfg(test)]
impl Arbitrary for Category {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Category {
            name: Arbitrary::arbitrary(g),
            req: Arbitrary::arbitrary(g),
            values: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.values
                .shrink()
                .map(|values| Category {
                    name: self.name.shrink().next().unwrap_or(self.name.clone()),
                    req: self.req.shrink().next().unwrap_or(self.req),
                    values,
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Salt {
    req: Requirement,
    values: String,
}

impl Salt {
    pub fn req(&self) -> Requirement {
        self.req
    }

    pub fn values(&self) -> &str {
        &self.values
    }
}

#[cfg(test)]
impl Arbitrary for Salt {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Salt {
            req: Arbitrary::arbitrary(g),
            values: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let i = self
            .values
            .shrink()
            .map(|values| Salt {
                req: self.req.shrink().next().unwrap_or(self.req),
                values,
            })
            .collect::<Vec<_>>()
            .into_iter();
        Box::new(i)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Requirement {
    Exactly(usize),
    AtLeast(usize),
    AtMost(usize),
}

impl From<(config::Requirement, usize)> for Requirement {
    fn from(value: (config::Requirement, usize)) -> Self {
        match value.0 {
            config::Requirement::AtLeast => Requirement::AtLeast(value.1),
            config::Requirement::AtMost => Requirement::AtMost(value.1),
            config::Requirement::Exactly => Requirement::Exactly(value.1),
        }
    }
}

#[cfg(test)]
impl Arbitrary for Requirement {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let n = Arbitrary::arbitrary(g);
        *g.choose(&[Exactly(n), AtLeast(n), AtMost(n)]).unwrap()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            AtLeast(n) => Box::new(n.shrink().map(AtLeast).collect::<Vec<_>>().into_iter()),
            AtMost(n) => Box::new(n.shrink().map(AtMost).collect::<Vec<_>>().into_iter()),
            Exactly(n) => Box::new(n.shrink().map(Exactly).collect::<Vec<_>>().into_iter()),
        }
    }
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

#[cfg(test)]
mod unit_tests {
    use crate::app::{to_empty_state, UiBlock};
    use crate::config::{self, parse_schema, Block};
    use crate::error::Error;
    use crate::filename::selection_to_filename;
    use crate::schema::Schema;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn blocks_with_tag(tag: &str) -> config::Schema {
        let categories = vec![
            config::Block::Salt(config::Salt {
                rtype: config::Requirement::Exactly,
                rvalue: 3,
                values: "ABC123".into(),
            }),
            config::Block::Category(config::Category {
                name: "Animals".to_string(),
                rtype: config::Requirement::AtLeast,
                rvalue: 0,
                values: vec![tag.to_string()],
            }),
        ];
        config::Schema {
            delim: "-".to_string(),
            blocks: categories,
        }
    }

    #[test]
    fn init_config_file_parses() {
        use std::fs;
        use std::path::Path;

        let expected = config::Schema {
            delim: "-".to_string(),
            blocks: vec![
                config::Block::Salt(config::Salt {
                    rtype: config::Requirement::Exactly,
                    rvalue: 6,
                    values: "ABCDEFGHIJKLMNPQRSTUVWXYZ123456789".to_string(),
                }),
                config::Block::Category(config::Category {
                    name: "Medium".to_string(),
                    rtype: config::Requirement::Exactly,
                    rvalue: 1,
                    values: vec![
                        "art".to_string(),
                        "photo".to_string(),
                        "ai".to_string(),
                        "other".to_string(),
                    ],
                }),
                config::Block::Category(config::Category {
                    name: "Subject".to_string(),
                    rtype: config::Requirement::AtLeast,
                    rvalue: 0,
                    values: vec![
                        "plants".to_string(),
                        "animals".to_string(),
                        "people".to_string(),
                    ],
                }),
            ],
        };

        match parse_schema(&fs::read_to_string(Path::new("./src/init.dhall")).unwrap()) {
            Err(e) => panic!("{e}"),
            Ok(schema) => assert_eq!(expected, schema),
        }
    }

    #[test]
    fn disallow_empty_tags() {
        let schema = config::Schema {
            delim: "-".to_string(),
            blocks: vec![config::Block::Category(config::Category {
                name: "Animals".to_string(),
                rtype: config::Requirement::AtMost,
                rvalue: 2,
                values: vec![],
            })],
        };

        match Schema::from_config(schema) {
            Err(Error::CategoryWithNoTags { category_name }) => {
                assert_eq!(category_name, "Animals")
            }
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn disallow_empty_salt_values() {
        let at_least0 = config::Salt {
            rtype: config::Requirement::AtLeast,
            rvalue: 0,
            values: "ABC123".to_string(),
        };
        let exactly0 = config::Salt {
            rtype: config::Requirement::Exactly,
            rvalue: 0,
            values: "ABC123".to_string(),
        };
        let at_most2 = config::Salt {
            rtype: config::Requirement::AtMost,
            rvalue: 2,
            values: "ABC123".to_string(),
        };

        for salt in [at_least0, exactly0, at_most2] {
            let schema = config::Schema {
                delim: "-".to_string(),
                blocks: vec![config::Block::Salt(salt)],
            };
            match Schema::from_config(schema) {
                Err(Error::SaltDefinitionMustExcludeEmptySalts) => (),
                Err(e) => panic!("{e:?}"),
                Ok(x) => panic!("{x:?}"),
            }
        }
    }

    #[test]
    fn disallow_empty_string_tag() {
        match Schema::from_config(blocks_with_tag("")) {
            Err(Error::EmptyStringNotValidTag) => (),
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn disallow_null_tag() {
        match Schema::from_config(blocks_with_tag("\0")) {
            Err(Error::InvalidCharacterInTag(c)) => assert_eq!(c, '\0'),
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn disallow_empty_string_delim() {
        let mut schema = blocks_with_tag("cat");
        schema.delim = "".into();
        match Schema::from_config(schema) {
            Err(Error::EmptyDelimiter) => (),
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn disallow_null_delim() {
        let mut schema = blocks_with_tag("cat");
        schema.delim = "\0".into();
        match Schema::from_config(schema) {
            Err(Error::InvalidCharacterInDelim(c)) => assert_eq!(c, '\0'),
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn no_tags_can_contain_delimiter() {
        let mut schema = blocks_with_tag("super-cat");
        schema.delim = "-".into();
        match Schema::from_config(schema) {
            Err(Error::DelimiterFoundInTag { tag, .. }) => assert_eq!(tag, "super-cat"),
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn all_tags_must_be_unique() {
        let mut schema = blocks_with_tag("cat");
        schema
            .blocks
            .push(config::Block::Category(config::Category {
                name: "People".to_string(),
                rtype: config::Requirement::AtLeast,
                rvalue: 0,
                values: vec!["chris".to_string(), "cat".to_string(), "nathan".to_string()],
            }));

        match Schema::from_config(schema) {
            Err(Error::TagsMustBeUnique {
                category_name,
                duplicated_tag,
            }) => {
                assert_eq!(category_name, "People");
                assert_eq!(duplicated_tag, "cat");
            }
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn basic_parse_two_categories() {
        let mut schema = blocks_with_tag("cat");
        schema
            .blocks
            .push(config::Block::Category(config::Category {
                name: "People".to_string(),
                rtype: config::Requirement::AtLeast,
                rvalue: 0,
                values: vec!["chris".to_string(), "nathan".to_string()],
            }));
        let schema = Schema::from_config(schema).unwrap();
        let mut state = to_empty_state(&schema, &mut ChaCha8Rng::seed_from_u64(0));
        state.iter_mut().for_each(|block| {
            if let UiBlock::Category { values, .. } = block {
                values
                    .iter_mut()
                    .for_each(|(tag, selected)| *selected = tag == "cat" || tag == "chris")
            }
        });

        let filename = selection_to_filename(&schema, &state).unwrap();
        assert_eq!(filename, "2C2-cat-chris");
        let parsed_state = schema.parse(&filename).unwrap();
        assert_eq!(state, parsed_state)
    }
}

#[cfg(test)]
mod prop_tests {
    use super::Schema;
    use crate::{
        app::{to_empty_state, UiBlock},
        config,
        filename::selection_to_filename,
    };
    use quickcheck::{Gen, QuickCheck, TestResult};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    // schemas should be able to parse the filenames they generate
    // TODO this does not include the salt and it should
    #[test]
    fn parse_generated_schemas() {
        fn closed_loop(schema: config::Schema, selection: u32, seed: u64) -> TestResult {
            let schema = match Schema::from_config(schema) {
                Err(_) => return TestResult::discard(),
                Ok(x) => x,
            };

            // quickcheck doesn't have a great way to generate bool values larger than the gen size
            // so I'm using this u32 like each bit is an arbitrary bool.
            let mut bool_selection = Vec::with_capacity(32);
            for i in 0..32 {
                let test = 1 << i;
                bool_selection.push(test & selection == test)
            }

            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut state = to_empty_state(&schema, &mut rng);
            let mut selection = bool_selection.to_vec();
            state.iter_mut().for_each(|block| match block {
                UiBlock::Category { values, .. } => {
                    let tags = values.clone().into_iter().map(|(s, _)| s);
                    let size = tags.len();
                    *values = tags.zip(selection.drain(0..size)).collect();
                }
                UiBlock::Salt { .. } => (),
            });

            match selection_to_filename(&schema, &state) {
                // The random state doesn't add up to a valid filename given the category restrictions
                Err(_) => TestResult::discard(),
                Ok(filename) => match schema.parse(&filename) {
                    Err(e) => {
                        println!("error:    {e}");
                        println!("schema:   {schema:?}");
                        println!("filename: {filename}");
                        println!("chars:    {:?}", filename.chars());
                        println!("state:    {state:?}");
                        println!("-----------------");
                        TestResult::failed()
                    }
                    Ok(parsed_state) => {
                        // for debugging with --nocapture:
                        if parsed_state != state {
                            println!("schema:   {schema:?}");
                            println!("filename: {filename}");
                            println!("chars:    {:?}", filename.chars());
                            println!("state:    {state:?}");
                            println!("parsed:   {parsed_state:?}");
                            println!("-----------------");
                        }
                        TestResult::from_bool(parsed_state == state)
                    }
                },
            }
        }

        QuickCheck::new()
            .gen(Gen::new(5))
            .quickcheck(closed_loop as fn(config::Schema, u32, u64) -> TestResult);
    }
}
