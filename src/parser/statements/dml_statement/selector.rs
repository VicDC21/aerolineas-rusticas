use crate::parser::data_types::{identifiers::identifier::Identifier, term::Term};

pub enum Selector {
    ColumnName(Identifier),
    Term(Term),
}
