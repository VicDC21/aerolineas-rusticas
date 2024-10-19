/// Representa diferentes tipos de operadores en una declaración DML.
#[derive(Debug)]
pub enum Operator {
    /// Representa el operador de igualdad (`=`).
    Equal,
    /// Representa el operador de menor que (`<`).
    Minor,
    /// Representa el operador de mayor que (`>`).
    Mayor,
    /// Representa el operador de menor o igual que (`<=`).
    MinorEqual,
    /// Representa el operador de mayor o igual que (`>=`).
    MayorEqual,
    /// Representa el operador de desigualdad (`!=`).
    Distinct,
    /// Representa el operador `IN`.
    In,
    /// Representa el operador `CONTAINS`.
    Contains,
    /// Representa el operador `CONTAINS KEY`.
    ContainsKey,
}
impl Operator {
    /// Verifica si una cadena es un operador.
    /// Si la cadena es un operador, devuelve el tipo de operador correspondiente.
    pub fn is_operator(operator: &String) -> Option<Operator> {
        if operator == "<" {
            Some(Operator::Minor)
        } else if operator == ">" {
            Some(Operator::Mayor)
        } else if operator == "=" {
            Some(Operator::Equal)
        } else if operator == "<=" {
            Some(Operator::MinorEqual)
        } else if operator == ">=" {
            Some(Operator::MayorEqual)
        } else if operator == "!=" {
            Some(Operator::Distinct)
        } else if operator == "IN" {
            Some(Operator::In)
        } else if operator == "CONTAINS" {
            Some(Operator::Contains)
        } else if operator == "CONTAINS KEY" {
            Some(Operator::ContainsKey)
        } else {
            None
        }
    }
}
