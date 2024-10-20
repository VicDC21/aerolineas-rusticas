use crate::{
    parser::data_types::identifier::identifier::Identifier, protocol::errors::error::Error,
};

use super::{and::And, operator::Operator, relation::Relation};

/// Representa diferentes tipos de expresiones en el analizador sintáctico.
#[derive(Debug)]
pub enum Expression {
    /// Representa una expresión simple.
    /// Una expresión simple consta de un solo término.
    Expression(Box<Expression>),
    /// Representa una operación lógica AND entre dos expresiones.
    /// AND ::= expression AND expression
    And(And),
    /// Representa una relación entre dos términos.
    /// relation ::= term operator term
    Relation(Relation),
}

/// Parsea una expresión recursivamente.
/// Una expresión puede ser una expresión simple o una expresión compuesta.
pub fn expression(lista: &mut Vec<String>) -> Result<Option<Box<Expression>>, Error> {
    match and(lista)? {
        Some(expression) => Ok(Some(expression)),
        None => Ok(None),
    }
}

/// Parsea una operación lógica AND entre dos expresiones.
pub fn and(lista: &mut Vec<String>) -> Result<Option<Box<Expression>>, Error> {
    let a_relation = match relation(lista)? {
        Some(relation) => relation,
        None => return Ok(None),
    };
    and_recursive(lista, a_relation)
}

/// Parsea una operación lógica AND entre dos expresiones recursivamente.
pub fn and_recursive(
    lista: &mut Vec<String>,
    a_relation: Box<Expression>,
) -> Result<Option<Box<Expression>>, Error> {
    if !lista.is_empty() {
        if lista[0] == "IF" || lista[0] == ";" {
            return Ok(Some(a_relation));
        }

        let value = lista.remove(0);
        if value == "AND" {
            let second_parameter = match relation(lista)? {
                Some(relation) => relation,
                None => return Ok(None),
            };
            let an_and = And::new(a_relation, second_parameter);
            let exp = Box::new(Expression::And(an_and));
            return and_recursive(lista, exp);
        }
    } else {
        return Ok(Some(a_relation));
    }
    Ok(None)
}

/// Parsea una relación entre dos términos.
pub fn relation(lista: &mut Vec<String>) -> Result<Option<Box<Expression>>, Error> {
    if lista.len() >= 3 {
        let first = match Identifier::check_identifier(lista)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError("Falta un operator".to_string())),
        };

        let operator = match Operator::is_operator(&lista[0]) {
            Some(value) => value,
            None => return Err(Error::SyntaxError("Falta un operator".to_string())),
        };

        lista.remove(0);

        let second = match Identifier::check_identifier(lista)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError("Falta un operator".to_string())),
        };
        let relation = Relation::new(first, operator, second);
        return Ok(Some(Box::new(Expression::Relation(relation))));
    }
    Ok(None)
}
