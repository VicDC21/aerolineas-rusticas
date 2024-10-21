use super::constant::Constant;
use crate::protocol::errors::error::Error;

#[allow(dead_code)]
/// TODO: Desc básica

#[derive(Debug)]
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
}
