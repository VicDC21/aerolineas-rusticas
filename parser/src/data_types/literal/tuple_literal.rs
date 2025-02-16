use crate::data_types::term::Term, statements::ddl_statement::ddl_statement_parser::check_words;
use protocol::{aliases::results::Result, errors::error::Error};

/// Literal de tipo tupla.
///
/// tuple_literal::= '(' term( ',' term )* ')'
#[derive(Debug)]
pub struct TupleLiteral {
    /// Elementos de la tupla, términos.
    pub items: Vec<Term>,
}

impl TupleLiteral {
    /// Verifica si la lista de tokens es una tupla de términos. Si lo es, lo retorna.
    /// Si no lo es, retorna None, o Error en caso de no cumplir con la sintaxis.
    pub fn check_tuple_literal(lista: &mut Vec<String>) -> Result<Option<Self>> {
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

    /// Obtiene la longitud de la tupla.
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// Devuelve los valores de la tupla como una lista de Strings.
    pub fn get_values_as_string(&self) -> Vec<String> {
        self.items
            .iter()
            .map(|term| term.get_value_as_string())
            .collect()
    }
}
