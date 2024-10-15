use crate::cassandra::errors::error::Error;

use super::constant::Constant;

pub enum Term {
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
    pub fn is_term(lista: &mut Vec<String>) -> Result<Option<Term>, Error> {
        // Todo: falta corroborar que el largo de la lista sea de al menos X largo asi no rompe con remove
        if Constant::check_string(&lista[0], &lista[2]) {
            lista.remove(0);
            let string = Constant::String(lista.remove(0));
            lista.remove(0);
            return Ok(Some(Term::Constant(string)));
        } else if Constant::check_integer(&lista[0]) {
            let integer_string: String = lista.remove(0);
            let int = Constant::new_integer(integer_string)?;
            return Ok(Some(Term::Constant(int)));
        } else if Constant::check_float(&lista[0]) {
            let float_string = lista.remove(0);
            let float = Constant::new_float(float_string)?;
            return Ok(Some(Term::Constant(float)));
        } else if Constant::check_boolean(&lista[0]) {
            let bool = lista.remove(0);
            let bool = Constant::new_boolean(bool)?;
            return Ok(Some(Term::Constant(bool)));
        } else if Constant::check_uuid(&lista[0]) {
            let uuid = lista.remove(0);
            let uuid = Constant::new_uuid(uuid)?;
            return Ok(Some(Term::Constant(uuid)));
        } else if Constant::check_blob(&lista[0]) {
            let blob = Constant::new_blob(lista.remove(0))?;
            return Ok(Some(Term::Constant(blob)));
        }
        Ok(None)
    }
}
