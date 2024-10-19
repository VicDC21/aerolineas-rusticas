use super::expression::Expression;

/// Representa una operación lógica AND entre dos expresiones.
#[derive(Debug)]
pub struct And {
    /// Primera expresión.
    /// Puede ser una expresión simple o una expresión compuesta.
    _first_relation: Box<Expression>,
    /// Segunda expresión.
    _second_relation: Box<Expression>,
}

impl And {
    /// Crea una nueva operación lógica AND.
    pub fn new(_first_relation: Box<Expression>, _second_relation: Box<Expression>) -> Self {
        And {
            _first_relation,
            _second_relation,
        }
    }
}
