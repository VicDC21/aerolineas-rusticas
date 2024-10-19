use crate::protocol::errors::error::Error;

use super::{
    data_types::{
        identifier::identifier::Identifier, literal::list_literal::ListLiteral, term::Term,
    },
    statements::ddl_statement::ddl_statement_parser::check_words,
};

/// assignment: simple_selection'=' term
/// `| column_name'=' column_name ( '+' | '-' ) term
/// | column_name'=' list_literal'+' column_name
#[derive(Debug)]
pub enum Assignment {
    /// simple_selection'=' term
    ColumnNameTerm(Identifier, Term),
    /// `| column_name'=' column_name ( '+' | '-' ) term
    ColumnNameColTerm(Identifier, Identifier, Term),
    /// | column_name'=' list_literal'+' column_name
    ColumnNameListCol(Identifier, ListLiteral, Identifier),
}

impl Assignment {
    /// Revisa que tipo de Assignment tiene el proximo valor de la lista, si el primer tipo de valor no es el esperado entonces devuelve None.
    /// Una vez comprobado que el primer parametro es correcto, entonces en cualquier caso donde se encuentre un error o
    /// falte un tipo de dato se devuelve un error. Si los datos son los esperados entonces devuelve un Assignment.
    pub fn check_kind_of_assignment(lista: &mut Vec<String>) -> Result<Option<Assignment>, Error> {
        let column_name = match Identifier::check_identifier(lista)? {
            Some(value) => value,
            None => return Ok(None),
        };
        if !check_words(lista, "=") {
            return Err(Error::SyntaxError("Falto un '='".to_string()));
        }
        if let Some(term) = Assignment::check_column_name_term(lista)? {
            return Ok(Some(Assignment::ColumnNameTerm(column_name, term)));
        }
        if let Some(values) = Assignment::check_column_name_col_term(lista)? {
            return Ok(Some(Assignment::ColumnNameColTerm(
                column_name,
                values.0,
                values.1,
            )));
        }
        if let Some(values) = Assignment::check_column_name_list_col(lista)? {
            return Ok(Some(Assignment::ColumnNameListCol(
                column_name,
                values.0,
                values.1,
            )));
        };
        Err(Error::SyntaxError(
            "Tipo de dato incorrecto al hacer SET".to_string(),
        ))
    }

    fn check_column_name_term(lista: &mut Vec<String>) -> Result<Option<Term>, Error> {
        let term = match Term::is_term(lista)? {
            Some(value) => value,
            None => return Ok(None),
        };
        Ok(Some(term))
    }

    fn check_column_name_col_term(
        lista: &mut Vec<String>,
    ) -> Result<Option<(Identifier, Term)>, Error> {
        let column_name = match Identifier::check_identifier(lista)? {
            Some(value) => value,
            None => return Ok(None),
        };
        if !check_words(lista, "+") && !check_words(lista, "-") {
            return Err(Error::SyntaxError("Falto un '+' o '-'".to_string()));
        }
        let term = match Term::is_term(lista)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError("Tipo de dato incorrecto".to_string())),
        };
        Ok(Some((column_name, term)))
    }

    fn check_column_name_list_col(
        lista: &mut Vec<String>,
    ) -> Result<Option<(ListLiteral, Identifier)>, Error> {
        let term = match ListLiteral::check_list_literal(lista)? {
            Some(value) => value,
            None => return Ok(None),
        };
        if !check_words(lista, "+") {
            return Err(Error::SyntaxError("Falto un '+'".to_string()));
        }
        let column_name = match Identifier::check_identifier(lista)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError("Tipo de dato incorrecto".to_string())),
        };
        Ok(Some((term, column_name)))
    }
}
