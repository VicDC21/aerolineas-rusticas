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
    pub items: Vec<Term>,
}

impl TupleLiteral {
    /// TODO: Desc básica
    pub fn check_tuple_literal(lista: &mut Vec<String>) -> Result<Option<Self>, Error> {
        let mut items: Vec<Term> = Vec::new();
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
                items.push(term);
            }
            Ok(Some(TupleLiteral { items }))
        } else {
            Ok(None)
        }
    }

    /// TODO: Desc básica
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// Devuelve los valores de la tupla como una lista de Strings.
    pub fn get_values_as_string(&self) -> Vec<String> {
        self.values
            .iter()
            .map(|term| term.get_value_as_string())
            .collect()
    }
}
