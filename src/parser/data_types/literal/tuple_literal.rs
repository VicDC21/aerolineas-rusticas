use crate::{
    parser::{
        data_types::term::Term, statements::ddl_statement::ddl_statement_parser::check_words,
    },
    protocol::errors::error::Error,
};

#[allow(dead_code)]
/// tuple_literal::= '(' term( ',' term )* ')'
#[derive(Debug)]
pub struct TupleLiteral {
    /// TODO: Desc básica
    values: Vec<Term>,
}

impl TupleLiteral {
    /// TODO: Desc básica
    pub fn check_tuple_literal(lista: &mut Vec<String>) -> Result<Option<Self>, Error> {
        let mut values: Vec<Term> = Vec::new();
        if check_words(lista, "(") {
            while check_words(lista, ",") || !check_words(lista, ")") {
                let term = match Term::is_term(lista)? {
                    Some(value) => value,
                    None => {
                        return Err(Error::SyntaxError(
                            "Sintaxis de tupla incorrecta".to_string(),
                        ))
                    }
                };
                values.push(term);
            }
            Ok(Some(TupleLiteral { values }))
        } else {
            Ok(None)
        }
    }
}
