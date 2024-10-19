use crate::parser::data_types::identifier::identifier::Identifier;

/// Representa una cláusula CQL GROUP BY.
#[derive(Debug)]
pub struct GroupBy {
    /// Columnas por las que se agrupará.
    pub columns: Vec<Identifier>,
}

impl GroupBy {
    /// Crea una nueva instancia de `GroupBy`.
    pub fn new(columns: Vec<Identifier>) -> Self {
        GroupBy { columns }
    }
}
