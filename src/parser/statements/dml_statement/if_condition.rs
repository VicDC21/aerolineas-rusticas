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
// /// Representa una condición en una declaración DML.
// pub enum Condition {
//     /// Representa una condición de igualdad.
//     Equals(Identifier, Term),
//     /// Representa una condición de desigualdad.
//     NotEquals(Identifier, Term),
//     /// Representa una condición de mayor que.
//     GreaterThan(Identifier, Term),
//     /// Representa una condición de mayor o igual que.
//     GreaterThanOrEqual(Identifier, Term),
//     /// Representa una condición de menor que.
//     LessThan(Identifier, Term),
//     /// Representa una condición de menor o igual que.
//     LessThanOrEqual(Identifier, Term),
//     /// Representa una condición de IN.
//     /// IN (value1, value2, value3, ...)
//     In(Identifier, Vec<Value>),
// }

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



// #[derive(Debug, Clone)]
// /// Representa un valor en una condición de una declaración DML.
// pub enum Value {
//     /// Representa un valor de tipo cadena.
//     String(String),
//     /// Representa un valor de tipo entero.
//     Integer(i64),
//     /// Representa un valor de tipo flotante.
//     Float(f64),
//     /// Representa un valor de tipo booleano.
//     Boolean(bool),
//     /// Representa un valor de tipo lista.
//     List(Vec<Value>),
//     /// Representa un valor de tipo identificador.
//     Identifier(String),
// }

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



// impl Condition {
//     /// Crea una nueva condición de comparación de valores.
//     pub fn column(&self) -> &str {
//         match self {
//             Condition::Equals(col, _) => col,
//             Condition::NotEquals(col, _) => col,
//             Condition::GreaterThan(col, _) => col,
//             Condition::GreaterThanOrEqual(col, _) => col,
//             Condition::LessThan(col, _) => col,
//             Condition::LessThanOrEqual(col, _) => col,
//             Condition::In(col, _) => col,
//         }
//     }

//     /// Idem, siendo que el valor de la columna es un identificador, compara si el valor en ese identificador es igual a otro.
//     pub fn value(&self) -> &Value {
//         match self {
//             Condition::Equals(_, val) => val,
//             Condition::NotEquals(_, val) => val,
//             Condition::GreaterThan(_, val) => val,
//             Condition::GreaterThanOrEqual(_, val) => val,
//             Condition::LessThan(_, val) => val,
//             Condition::LessThanOrEqual(_, val) => val,
//             Condition::In(_, vals) => &vals[0],
//         }
//     }
// }
