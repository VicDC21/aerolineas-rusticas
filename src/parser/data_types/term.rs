use super::constant::Constant;
use crate::protocol::errors::error::Error;
use std::cmp::Ordering;

#[allow(dead_code)]
/// TODO: Desc básica
#[derive(Debug, Clone)]
pub enum Term {
    /// TODO: Desc básica
    Constant(Constant),
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Term::Constant(c1), Term::Constant(c2)) => c1 == c2,
        }
    }
}

impl Term {
    /// TODO: Desc básica
    pub fn get_value(&self) -> String {
        match self {
            Term::Constant(c) => c.get_value(),
        }
    }

    /// TODO: Desc básica
    pub fn is_term(lista: &mut Vec<String>) -> Result<Option<Term>, Error> {
        if let Some(constant) = Constant::is_constant(lista)? {
            return Ok(Some(Term::Constant(constant)));
        }
        Ok(None)
    }

    /// Devuelve el valor del término como un String.
    pub fn get_value_as_string(&self) -> String {
        match self {
            Term::Constant(constant) => constant.get_value_as_string(),
        }
    }
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Term::Constant(c1), Term::Constant(c2)) => c1.partial_cmp(c2),
        }
    }
}
