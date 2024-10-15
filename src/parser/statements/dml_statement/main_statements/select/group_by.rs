use crate::parser::data_types::identifier::identifier::Identifier;

pub struct GroupBy {
    columns: Vec<Identifier>,
}

impl GroupBy {
    pub fn new(columns: Vec<Identifier>) -> Self {
        GroupBy { columns }
    }
}
