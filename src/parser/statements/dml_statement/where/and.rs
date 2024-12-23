use crate::{
    parser::statements::dml_statement::r#where::expression::Expression,
    protocol::aliases::results::Result,
};

/// Representa una operación lógica AND entre dos expresiones.
#[derive(Debug)]
pub struct And {
    /// Primera expresión.
    /// Puede ser una expresión simple o una expresión compuesta.
    pub first_relation: Box<Expression>,
    /// Segunda expresión.
    pub second_relation: Box<Expression>,
}

impl And {
    /// Crea una nueva operación lógica AND.
    pub fn new(first_relation: Box<Expression>, second_relation: Box<Expression>) -> Self {
        And {
            first_relation,
            second_relation,
        }
    }

    /// Evalúa la operación lógica AND.
    pub fn evaluate(&self, line_to_review: &[String], general_columns: &[String]) -> Result<bool> {
        let passed_filter: bool = self
            .first_relation
            .evaluate(line_to_review, general_columns)?
            && self
                .second_relation
                .evaluate(line_to_review, general_columns)?;
        Ok(passed_filter)
    }
}
