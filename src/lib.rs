#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Category {
    pub name: String,
    pub id: String,
    pub requirement: Option<usize>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Keyword {
    pub name: String,
    pub id: String,
}
