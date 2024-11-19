use crate::{
    parser::statements::ddl_statement::ddl_statement_parser::check_words,
    protocol::errors::error::Error,
};

use super::super::term::Term;

#[allow(dead_code)]
/// Literal de tipo mapa.

#[derive(Debug, PartialEq, Clone)]
pub struct MapLiteral {
    /// Valores del mapa, pares de términos.
    pub values: Vec<(Term, Term)>,
}

impl MapLiteral {
    /// Verifica si la lista de tokens es un mapa de términos. Si lo es, lo retorna.
    /// Si no lo es, retorna None, o Error en caso de no cumplir con la sintaxis.
    pub fn check_map_literal(lista: &mut Vec<String>) -> Result<Option<Self>, Error> {
        let mut values: Vec<(Term, Term)> = Vec::new();
        if check_words(lista, "{") {
            while check_words(lista, ",") || !check_words(lista, "}") {
                let term = match Term::is_term(lista)? {
                    Some(value) => value,
                    None => {
                        return Err(Error::SyntaxError(
                            "Sintaxis de mapa incorrecta".to_string(),
                        ))
                    }
                };
                if !check_words(lista, ":") {
                    return Err(Error::SyntaxError(
                        "Sintaxis de mapa incorrecta".to_string(),
                    ));
                }
                let term2 = match Term::is_term(lista)? {
                    Some(value) => value,
                    None => {
                        return Err(Error::SyntaxError(
                            "Sintaxis de mapa incorrecta".to_string(),
                        ))
                    }
                };
                values.push((term, term2));
            }
        }
        Ok(Some(MapLiteral { values }))
    }

    /// Obtiene los valores del mapa.
    pub fn get_values(&self) -> &Vec<(Term, Term)> {
        &self.values
    }
}
