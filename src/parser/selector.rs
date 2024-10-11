use crate::parser::data_types::{identifier::Identifier, term::Term};

pub enum Selector{
    ColumnName(Identifier),
    Term(Term)
}