use super::expression::Expression;

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
}
