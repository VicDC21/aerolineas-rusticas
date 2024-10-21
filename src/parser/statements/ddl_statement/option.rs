use crate::{
    parser::data_types::{
        identifier::identifier::Identifier, literal::map_literal::MapLiteral, term::Term,
    },
    protocol::errors::error::Error,
};

/// options::= option ( AND option )*
/// option::= identifier '=' ( identifier
///     | constant
///     | map_literal )
#[derive(Debug)]
pub enum Options {
    /// Representa un identificador.
    /// Ejemplo: `keyspace_name = 'keyspace'`
    Identifier(Identifier),
    /// Representa un término constante.
    /// Ejemplo: `keyspace_name = 'keyspace'`
    Constant(Term),
    /// Representa un literal de mapa.
    /// Ejemplo: `keyspace_name = {'keyspace': 'value'}`
    MapLiteral(MapLiteral),
}

impl Options {
    /// Verifica si la lista de tokens es un literal de mapa.
    pub fn check_options(lista: &mut Vec<String>) -> Result<Self, Error> {
        if let Some(identifier) = Identifier::check_identifier(lista)? {
            return Ok(Options::Identifier(identifier));
        } else if let Some(constant) = Term::is_term(lista)? {
            return Ok(Options::Constant(constant));
        } else if let Some(map) = MapLiteral::check_map_literal(lista)? {
            return Ok(Options::MapLiteral(map));
        };

        Err(Error::SyntaxError(
            "Error de sintaxis en las opciones".to_string(),
        ))
    }
}
