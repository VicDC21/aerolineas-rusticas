use crate::{
    data_types::{identifier::identifier_mod::Identifier, term::Term},
    statements::dml_statement::r#where::operator::Operator,
};

/// Representa una condición IF en una declaración DML.
#[derive(Debug, PartialEq)]
pub enum IfCondition {
    /// Representa una condición IF EXISTS.    
    Exists,
    /// Representa una condición IF con una lista de condiciones.
    Conditions(Vec<Condition>),
    /// Representa una condición IF sin condiciones.
    None,
}

/// Representa una condición en una declaración DML.
#[derive(Debug, PartialEq)]
pub struct Condition {
    /// Identificador de la primera columna.
    /// La primera columna es la columna de la izquierda en la condicion.
    pub first_column: Identifier,
    /// Operador de la condicion.
    /// El operador se utiliza para comparar las dos columnas.
    pub operator: Operator,
    /// Termino de la segunda columna.
    /// La segunda columna es la columna de la derecha en la condicion.
    pub second_column: Term,
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

impl Condition {
    /// Crea un nuevo Condition
    pub fn new(first_column: Identifier, operator: Operator, second_column: Term) -> Condition {
        Condition {
            first_column,
            operator,
            second_column,
        }
    }
}
