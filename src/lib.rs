pub mod filename;
pub mod gui;
pub mod schema;

use schema::{Category, Keyword};

type State = Vec<(Category, Vec<(Keyword, bool)>)>;
