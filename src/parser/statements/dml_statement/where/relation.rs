use crate::parser::data_types::identifier::identifier::Identifier;
use crate::parser::statements::dml_statement::r#where::operator::Operator;

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
