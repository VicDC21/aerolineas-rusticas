use super::{data_types::identifier::Identifier, operator::Operator};

pub struct Relation {
    first_column: Identifier,
    operator: Operator,
    second_column: Identifier,
}

impl Relation {
    pub fn new(first_column: Identifier, operator: Operator, second_column: Identifier) -> Self {
        Relation {
            first_column,
            operator,
            second_column,
        }
    }
}
