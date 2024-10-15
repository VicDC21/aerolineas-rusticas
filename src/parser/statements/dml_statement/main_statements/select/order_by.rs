use crate::parser::data_types::identifier::identifier::Identifier;

/// ordering_clause::= column_name [ ASC | DESC ] ( ',' column_name [ ASC | DESC ] )*
pub struct OrderBy {
    columns: Vec<(Identifier, Option<Ordering>)>,
}

impl OrderBy {
    pub fn new(columns: Vec<(Identifier, Option<Ordering>)>) -> Self {
        OrderBy { columns }
    }
}

pub enum Ordering {
    Asc,
    Desc,
}
