use super::{Category, ExprU, Keyword, Schema, SchemaTypeCheckError};
use std::result::Result as StdResult;

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExprT {
    SchemaT(Schema),
    CategoryT(Category),
    KeywordT(Keyword),
}

type Result<T> = StdResult<T, SchemaTypeCheckError>;

pub fn typecheck(expr: ExprU) -> Result<Schema> {
    todo!()
}

fn typecheck_(expr: ExprU) -> Result<ExprT> {
    todo!()
}
