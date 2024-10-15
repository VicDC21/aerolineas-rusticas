use crate::parser::data_types::identifier::identifier::Identifier;
use crate::parser::statements::dml_statement::r#where::operator::Operator;

/// Representa una relación en una cláusula WHERE con dos columnas y un operador.
pub struct Relation {
    /// Identificador de la primera columna.
    /// La primera columna es la columna de la izquierda en la relación.
    _first_column: Identifier,
    /// Operador de la relación.
    /// El operador se utiliza para comparar las dos columnas.
    _operator: Operator,
    /// Identificador de la segunda columna.
    /// La segunda columna es la columna de la derecha en la relación.
    _second_column: Identifier,
}

impl Relation {
    /// Constructor de la relación.
    pub fn new(_first_column: Identifier, _operator: Operator, _second_column: Identifier) -> Self {
        Relation {
            _first_column,
            _operator,
            _second_column,
        }
    }
}
