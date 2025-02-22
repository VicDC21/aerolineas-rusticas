use crate::data_types::cql_type::{collection_type::expect_token, cql_type_mod::CQLType};
use protocol::{aliases::results::Result, errors::error::Error};

/// Tipo de tupla.
///
/// tuple_type::= TUPLE '<' cql_type( ',' cql_type)* '>'
#[derive(Debug, PartialEq)]
pub enum TupleType {
    /// Tupla de tipos de datos.
    Tuple(Box<Vec<CQLType>>),
}

impl TupleType {
    /// Verifica si la lista de tokens es un tipo de tupla. Si lo es, lo retorna.
    /// Si no lo es, retorna None, o Error en caso de no poder parsearla.
    pub fn parse_tuple_type(tokens: &mut Vec<String>) -> Result<Option<TupleType>> {
        expect_token(tokens, "(")?;
        let mut values = Vec::new();
        loop {
            let r#type = match CQLType::check_kind_of_type(tokens)? {
                Some(value) => value,
                None => return Err(Error::SyntaxError(("Tipo de dato invalido").to_string())),
            };
            values.push(r#type);
            if expect_token(tokens, ")").is_ok() || tokens.is_empty() {
                break;
            }
        }

        Ok(Some(TupleType::Tuple(Box::new(values))))
    }
}
