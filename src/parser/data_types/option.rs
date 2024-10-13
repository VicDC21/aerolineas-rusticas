use crate::{
    cassandra::errors::error::Error,
    parser::statements::dml_statement::dml_statement_parser::is_term,
};

use super::{identifier::Identifier, map_literal::MapLiteral, term::Term};

// options::= option ( AND option )*
// option::= identifier '=' ( identifier
// 	| constant
// 	| map_literal )
pub enum Options {
    Identifier(Identifier),
    Constant(Term), //termine usando Term porque tengo una funcion que me devuelve este tipo de dato
    // y ademas Term casi que equivale a Constant.
    MapLiteral(MapLiteral),
}

impl Options {
    pub fn check_options(lista: &mut Vec<String>) -> Result<Self, Error> {
        if let Some(map) = MapLiteral::check_map_literal(lista)? {
            return Ok(Options::MapLiteral(map));
        }
        if let Some(constant) = is_term(lista)? {
            return Ok(Options::Constant(constant));
        }
        if let Some(identifier) = Identifier::check_identifier(lista)? {
            return Ok(Options::Identifier(identifier));
        };

        Err(Error::SyntaxError(
            "Error de sinstaxis en las opciones".to_string(),
        ))
    }
}
