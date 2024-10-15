use crate::parser::data_types::{identifier::identifier::Identifier, term::Term};

/// Representa un selector en una declaración SQL.
pub enum Selector {
    /// Nombre de una columna.
    ColumnName(Identifier),
    /// Término.
    Term(Term),
}
