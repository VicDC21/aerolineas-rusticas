use crate::parser::{data_types::identifier::identifier::Identifier, data_types::term::Term};
use crate::protocol::{aliases::results::Result, errors::error::Error};

use super::{and::And, operator::Operator, relation::Relation};

/// Representa diferentes tipos de expresiones en el analizador sintáctico.
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
pub fn expression(lista: &mut Vec<String>) -> Result<Option<Box<Expression>>> {
    match and(lista)? {
        Some(expression) => Ok(Some(expression)),
        None => Ok(None),
    }
}

/// Parsea una operación lógica AND entre dos expresiones.
pub fn and(lista: &mut Vec<String>) -> Result<Option<Box<Expression>>> {
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
) -> Result<Option<Box<Expression>>> {
    if !lista.is_empty() {
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
pub fn relation(lista: &mut Vec<String>) -> Result<Option<Box<Expression>>> {
    if lista.len() >= 3 {
        if let Some(operator) = Operator::is_operator(&lista[1]) {
            lista.remove(1);
            let first = match Identifier::check_identifier(lista)? {
                Some(value) => value,
                None => {
                    return Err(Error::SyntaxError(
                        "Falta un operator en el where".to_string(),
                    ))
                }
            };
            let second = match Term::is_term(lista)? {
                Some(value) => value,
                None => {
                    return Err(Error::SyntaxError(
                        "Falta un operator en el where".to_string(),
                    ))
                }
            };
            let relation = Relation::new(first, operator, second);
            return Ok(Some(Box::new(Expression::Relation(relation))));
        }
    }
    Ok(None)
}

impl Expression {
    /// Evalúa la expresión de la cláusula WHERE.
    pub fn evaluate(&self, line_to_review: &[String], general_columns: &[String]) -> Result<bool> {
        let result = match &self {
            Expression::Expression(another_expression) => {
                another_expression.evaluate(line_to_review, general_columns)?
            }
            Expression::And(and) => and.evaluate(line_to_review, general_columns)?,
            Expression::Relation(relation) => relation.evaluate(line_to_review, general_columns)?,
        };
        Ok(result)
    }
}
