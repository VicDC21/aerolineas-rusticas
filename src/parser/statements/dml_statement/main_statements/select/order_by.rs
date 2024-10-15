use crate::parser::data_types::identifier::identifier::Identifier;

/// ordering_clause::= column_name [ ASC | DESC ] ( ',' column_name [ ASC | DESC ] )*
pub struct OrderBy {
    /// Lista de columnas y dirección de ordenación.
    pub columns: Vec<(Identifier, Option<Ordering>)>,
}

impl OrderBy {
    /// Crea una nueva cláusula ORDER BY.
    pub fn new(columns: Vec<(Identifier, Option<Ordering>)>) -> Self {
        OrderBy { columns }
    }
}

/// Representa la dirección de ordenación en una cláusula ORDER BY.
pub enum Ordering {
    /// Orden ascendente.
    Asc,
    /// Orden descendente.
    Desc,
}
