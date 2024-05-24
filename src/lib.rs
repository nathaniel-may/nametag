use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Category {
    name: String,
    id: String,
    requirement: Option<usize>,
    keywords: HashSet<Keyword>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Keyword {
    name: String,
    id: String,
}
