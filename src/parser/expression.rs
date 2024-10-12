use crate::cassandra::errors::error::Error;

use super::{and::And, operator::Operator, relation::Relation, statements::dml_statement::dml_statement_parser::is_column_name, r#where::Where};

pub enum Expression{
    Expression(Box<Expression>),
    And(And),
    Relation(Relation)

}

pub fn expression(lista: &mut Vec<String>) -> Result<Option<Box<Expression>>, Error>{
    match and(lista)?{
        Some(expression) => return Ok(Some(expression)),
        None => return Ok(None)
    }
}

pub fn and(lista: &mut Vec<String>) -> Result<Option<Box<Expression>>, Error>{
    let a_relation = match relation(lista)?{
        Some(relation) => relation,
        None => return Ok(None)
    };
    and_recursive(lista, a_relation)
}

pub fn and_recursive(lista: &mut Vec<String>, a_relation: Box<Expression>) -> Result<Option<Box<Expression>>, Error>{
    if !lista.is_empty(){
        let value = lista.remove(0);
        if value == "AND"{
            let second_parameter = match relation(lista)?{
                Some(relation) => relation,
                None => return Ok(None)
            };
            let an_and = And::new(a_relation, second_parameter);
            let exp = Box::new(Expression::And(an_and));
            return and_recursive(lista, exp)
            
        }
    } else{
        return Ok(Some(a_relation))
    }
    Ok(None)
}



pub fn relation(lista: &mut Vec<String>) -> Result<Option<Box<Expression>>, Error>{
    // if lista[1] == "("{
    //     return Ok(expression(lista)?);
    // }
    if lista.len() >= 3{
        if let Some(operator) = Operator::is_operator(&lista[1]){
            lista.remove(1);
            let first = match is_column_name(lista)?{
                Some(value) => value,
                None => return Err(Error::SyntaxError("Falta un operator".to_string()))
            };
            let second = match is_column_name(lista)?{
                Some(value) => value,
                None => return Err(Error::SyntaxError("Falta un operator".to_string()))
            };
            let relation = Relation::new(first, operator, second);
            return Ok(Some(Box::new(Expression::Relation(relation))))
        }
    }
    Ok(None)
}