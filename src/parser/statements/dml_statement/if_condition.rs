use crate::parser::data_types::{identifier::identifier::Identifier, term::Term};

use super::r#where::operator::Operator;


/// Representa una condición IF en una declaración DML.
pub enum IfCondition {
    /// Representa una condición IF EXISTS.    
    Exists,
    /// Representa una condición IF con una lista de condiciones.
    Conditions(Vec<Condition>),
    /// Representa una condición IF sin condiciones.
    None,
}

/// Representa una condición en una declaración DML.
pub struct Condition {
    /// Identificador de la primera columna.
    /// La primera columna es la columna de la izquierda en la condicion.
    _first_column: Identifier,
    /// Operador de la condicion.
    /// El operador se utiliza para comparar las dos columnas.
    _operator: Operator,
    /// Termino de la segunda columna.
    /// La segunda columna es la columna de la derecha en la condicion.
    _second_column: Term,
}



impl IfCondition {
    /// Crea una nueva condición IF EXISTS.
    pub fn new_exists() -> Self {
        IfCondition::Exists
    }

    /// Crea una nueva condición IF con una lista de condiciones.
    pub fn new_conditions(conditions: Vec<Condition>) -> Self {
        IfCondition::Conditions(conditions)
    }

    /// Crea una nueva condición IF sin condiciones.
    pub fn new_none() -> Self {
        IfCondition::None
    }
}

impl Condition{
    /// Crea un nuevo Condition
    pub fn new(_first_column: Identifier, _operator: Operator, _second_column: Term) -> Condition{
        Condition{ _first_column, _operator, _second_column
        }

    }
}