use crate::parser::data_types::{identifier::identifier::Identifier, term::Term};

pub enum Selector {
    ColumnName(Identifier),
    Term(Term),
}
