pub mod app;
pub mod filename;
pub mod fs;
pub mod schema;

use schema::{Category, Keyword};

type State = Vec<(Category, Vec<(Keyword, bool)>)>;
