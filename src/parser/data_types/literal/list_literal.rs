use crate::{
    parser::{
        data_types::term::Term, statements::ddl_statement::ddl_statement_parser::check_words,
    },
    protocol::errors::error::Error,
};

#[allow(dead_code)]
/// Literal de tipo lista.
#[derive(Debug)]
pub struct ListLiteral {
    /// Valores de la lista, términos.
    values: Vec<Term>,
}

impl ListLiteral {
    /// Verifica si la lista de tokens es una lista de literales. Si lo es, lo retorna.
    /// Si no lo es, retorna None, o Error en caso de no cumplir con la sintaxis.
    pub fn check_list_literal(lista: &mut Vec<String>) -> Result<Option<Self>, Error> {
        let mut values: Vec<Term> = Vec::new();
        if check_words(lista, "[") {
            while check_words(lista, ",") || !check_words(lista, "]") {
                let term = match Term::is_term(lista)? {
                    Some(value) => value,
                    None => {
                        return Err(Error::SyntaxError(
                            "Sintaxis de lista incorrecta".to_string(),
                        ))
                    }
                };
                values.push(term);
            }
        }
        Ok(Some(ListLiteral { values }))
    }
}
