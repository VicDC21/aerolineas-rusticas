#[derive(Debug, Clone)]

/// Representa una condición IF en una declaración DML.
pub enum IfCondition {
    /// Representa una condición IF EXISTS.    
    Exists,
    /// Representa una condición IF con una lista de condiciones.
    Conditions(Vec<Condition>),
}
#[derive(Debug, Clone)]
/// Representa una condición en una declaración DML.
pub enum Condition {
    /// Representa una condición de igualdad.
    Equals(String, Value),
    /// Representa una condición de desigualdad.
    NotEquals(String, Value),
    /// Representa una condición de mayor que.
    GreaterThan(String, Value),
    /// Representa una condición de mayor o igual que.
    GreaterThanOrEqual(String, Value),
    /// Representa una condición de menor que.
    LessThan(String, Value),
    /// Representa una condición de menor o igual que.
    LessThanOrEqual(String, Value),
    /// Representa una condición de IN.
    /// IN (value1, value2, value3, ...)
    In(String, Vec<Value>),
}

#[derive(Debug, Clone)]
/// Representa un valor en una condición de una declaración DML.
pub enum Value {
    /// Representa un valor de tipo cadena.
    String(String),
    /// Representa un valor de tipo entero.
    Integer(i64),
    /// Representa un valor de tipo flotante.
    Float(f64),
    /// Representa un valor de tipo booleano.
    Boolean(bool),
    /// Representa un valor de tipo lista.
    List(Vec<Value>),
    /// Representa un valor de tipo identificador.
    Identifier(String),
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
}

impl Condition {
    /// Crea una nueva condición de comparación de valores.
    pub fn column(&self) -> &str {
        match self {
            Condition::Equals(col, _) => col,
            Condition::NotEquals(col, _) => col,
            Condition::GreaterThan(col, _) => col,
            Condition::GreaterThanOrEqual(col, _) => col,
            Condition::LessThan(col, _) => col,
            Condition::LessThanOrEqual(col, _) => col,
            Condition::In(col, _) => col,
        }
    }

    /// Idem, siendo que el valor de la columna es un identificador, compara si el valor en ese identificador es igual a otro.
    pub fn value(&self) -> &Value {
        match self {
            Condition::Equals(_, val) => val,
            Condition::NotEquals(_, val) => val,
            Condition::GreaterThan(_, val) => val,
            Condition::GreaterThanOrEqual(_, val) => val,
            Condition::LessThan(_, val) => val,
            Condition::LessThanOrEqual(_, val) => val,
            Condition::In(_, vals) => &vals[0],
        }
    }
}
