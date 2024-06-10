use crate::app::{State, UiCategory};
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
}

impl fmt::Display for FilenameParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnexpectedTag(tag) => write!(f, "Unexpected tag: {tag}"),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Schema {
    delim: String,
    categories: Vec<Category>,
}

impl Schema {
    fn char_allowed(c: char) -> bool {
        // no control characters. They can't all be read back after being written.
        (c as u32) >= 32 && !['\0'].contains(&c)
    }

    // This is the only way to go from the input type config::Schema to the internal state of schema::Schema.
    pub fn from_config(config: config::Schema) -> Result<Schema> {
        if config.delim.is_empty() {
            return Err(Error::EmptyDelimiter);
        }
        for c in config.delim.chars() {
            if !Schema::char_allowed(c) {
                return Err(Error::InvalidCharacterInDelim(c));
            }
        }

        let mut m: HashSet<&str> = HashSet::new();

        for cat in &config.categories {
            if cat.values.is_empty() {
                return Err(Error::CategoryWithNoTags {
                    category_name: cat.name.clone(),
                });
            }
            if cat.values.contains(&String::new()) {
                return Err(Error::EmptyStringNotValidTag);
            }

            for v in &cat.values {
                if !m.insert(v) {
                    return Err(Error::TagsMustBeUnique {
                        category_name: cat.name.clone(),
                        duplicated_tag: v.clone(),
                    });
                }
                if v.contains(&config.delim) {
                    return Err(Error::DelimiterFoundInTag {
                        category_name: cat.name.clone(),
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

        let mut categories = Vec::with_capacity(config.categories.len());
        for cat in config.categories {
            let cat = Category {
                name: cat.name,
                req: (cat.rtype, cat.rvalue).into(),
                values: cat.values,
            };
            categories.push(cat);
        }
        let schema = Schema {
            delim: config.delim,
            categories,
        };
        Ok(schema)
    }

    pub fn delim(&self) -> &str {
        self.delim.as_str()
    }

    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    pub fn parse(&self, input: &str) -> StdResult<State, FilenameParseError> {
        let mut tags = input.split(&self.delim).peekable();
        // todo actually parse valid salts.
        let salt = tags.next().unwrap();
        let mut categories = Vec::with_capacity(self.categories.len());
        for cat in &self.categories[..] {
            let applied_tags = tags.drain_while(|tag| cat.values.contains(&tag.to_string()));

            let values = cat
                .values
                .clone()
                .into_iter()
                .map(|name| (name.clone(), applied_tags.contains(&name.as_str())))
                .collect();

            categories.push(UiCategory {
                name: cat.name.clone(),
                values,
            });
        }

        match &tags.collect::<Vec<_>>()[..] {
            [] => {
                let state = State {
                    salt: salt.to_string(),
                    categories,
                };
                Ok(state)
            }
            [h, ..] => Err(FilenameParseError::UnexpectedTag(h.to_string())),
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
            categories: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let cats = self
            .categories
            .shrink()
            .map(|categories| Schema {
                delim: self.delim.clone(),
                categories,
            })
            .collect::<Vec<_>>();

        let delims = self.delim.shrink().map(|delim| Schema {
            delim,
            categories: self.categories.clone(),
        });

        let mut all = cats;
        all.extend(delims);

        Box::new(all.into_iter())
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
    use crate::app::to_empty_state;
    use crate::config::{self, parse_schema};
    use crate::error::Error;
    use crate::filename::selection_to_filename;
    use crate::schema::Schema;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn schema_with_tag(tag: &str) -> config::Schema {
        let categories = vec![config::Category {
            name: "Animals".to_string(),
            rtype: config::Requirement::AtLeast,
            rvalue: 0,
            values: vec![tag.to_string()],
        }];
        config::Schema {
            delim: "-".to_string(),
            categories,
        }
    }

    #[test]
    fn init_config_file_parses() {
        use std::fs;
        use std::path::Path;

        let expected = config::Schema {
            delim: "-".to_string(),
            categories: vec![
                config::Category {
                    name: "Medium".to_string(),
                    rtype: config::Requirement::Exactly,
                    rvalue: 1,
                    values: vec![
                        "art".to_string(),
                        "photo".to_string(),
                        "ai".to_string(),
                        "other".to_string(),
                    ],
                },
                config::Category {
                    name: "Subject".to_string(),
                    rtype: config::Requirement::AtLeast,
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

    #[test]
    fn disallow_empty_tags() {
        let schema = config::Schema {
            delim: "-".to_string(),
            categories: vec![config::Category {
                name: "Animals".to_string(),
                rtype: config::Requirement::AtMost,
                rvalue: 2,
                values: vec![],
            }],
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
    fn disallow_empty_string_tag() {
        match Schema::from_config(schema_with_tag("")) {
            Err(Error::EmptyStringNotValidTag) => (),
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn disallow_null_tag() {
        match Schema::from_config(schema_with_tag("\0")) {
            Err(Error::InvalidCharacterInTag(c)) => assert_eq!(c, '\0'),
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn disallow_empty_string_delim() {
        let mut schema = schema_with_tag("cat");
        schema.delim = "".into();
        match Schema::from_config(schema) {
            Err(Error::EmptyDelimiter) => (),
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn disallow_null_delim() {
        let mut schema = schema_with_tag("cat");
        schema.delim = "\0".into();
        match Schema::from_config(schema) {
            Err(Error::InvalidCharacterInDelim(c)) => assert_eq!(c, '\0'),
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn no_tags_can_contain_delimiter() {
        let mut schema = schema_with_tag("super-cat");
        schema.delim = "-".into();
        match Schema::from_config(schema) {
            Err(Error::DelimiterFoundInTag { tag, .. }) => assert_eq!(tag, "super-cat"),
            Err(e) => panic!("{e:?}"),
            Ok(x) => panic!("{x:?}"),
        }
    }

    #[test]
    fn all_tags_must_be_unique() {
        let mut schema = schema_with_tag("cat");
        schema.categories.push(config::Category {
            name: "People".to_string(),
            rtype: config::Requirement::AtLeast,
            rvalue: 0,
            values: vec!["chris".to_string(), "cat".to_string(), "nathan".to_string()],
        });

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
        let mut schema = schema_with_tag("cat");
        schema.categories.push(config::Category {
            name: "People".to_string(),
            rtype: config::Requirement::AtLeast,
            rvalue: 0,
            values: vec!["chris".to_string(), "nathan".to_string()],
        });
        let schema = Schema::from_config(schema).unwrap();
        let mut state = to_empty_state(&schema, &mut ChaCha8Rng::seed_from_u64(0));
        state.categories[0].values[0] = ("cat".into(), true);
        state.categories[1].values[0] = ("chris".into(), true);

        let filename = selection_to_filename(&schema, &state).unwrap();
        assert_eq!(filename, "ZQYC5T-cat-chris");
        let parsed_state = schema.parse(&filename).unwrap();
        assert_eq!(state, parsed_state)
    }
}

#[cfg(test)]
mod prop_tests {
    use super::Schema;
    use crate::{app::to_empty_state, config, filename::selection_to_filename};
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
            for cat in &mut state.categories[..] {
                let tags = cat.values.clone().into_iter().map(|(s, _)| s);
                let size = tags.len();
                cat.values = tags.zip(selection.drain(0..size)).collect();
            }

            match selection_to_filename(&schema, &state) {
                // The random state doesn't add up to a valid filename given the category restrictions
                Err(_) => TestResult::discard(),
                Ok(filename) => match schema.parse(&filename) {
                    Err(_) => TestResult::failed(),
                    Ok(parsed_state) => {
                        // for debugging with --nocapture:
                        if parsed_state != state {
                            println!("schema:   {schema:?}");
                            println!("filename: {filename}");
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
