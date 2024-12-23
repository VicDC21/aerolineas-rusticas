use crate::parser::data_types::{identifier::identifier_mod::Identifier, term::Term};

/// Representa un selector en una declaración SQL.
#[derive(Debug, PartialEq)]
pub enum Selector {
    /// Nombre de una columna.
    ColumnName(Identifier),
    /// Término.
    Term(Term),
}

impl Selector {
    /// Obtiene el nombre del selector.
    pub fn get_name(&self) -> String {
        match self {
            Selector::ColumnName(column_name) => column_name.get_name().to_string(),
            Selector::Term(term) => term.get_value_as_string(),
        }
    }
}
