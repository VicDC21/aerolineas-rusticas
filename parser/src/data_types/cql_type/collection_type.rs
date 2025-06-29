use crate::data_types::cql_type::cql_type_mod::CQLType;
use protocol::{aliases::results::Result, errors::error::Error};

/// Tipo de colección.
#[derive(Debug, PartialEq)]
pub enum CollectionType {
    /// MAP '<' cql_type',' cql_type'>'
    Map(Box<CQLType>, Box<CQLType>),

    /// SET '<' cql_type '>'
    Set(Box<CQLType>),

    /// LIST '<' cql_type'>'
    List(Box<CQLType>),
}

impl CollectionType {
    /// Verifica si la lista de tokens es un tipo de colección. Si lo es, lo retorna.
    /// Si no lo es, retorna None, o Error en caso de no poder parsearla.
    pub fn parse_collection_type(tokens: &mut Vec<String>) -> Result<Option<CollectionType>> {
        CollectionType::parse_list_type(tokens)?;
        CollectionType::parse_map_type(tokens)?;
        CollectionType::parse_set_type(tokens)?;
        Ok(None)
    }

    fn parse_list_type(tokens: &mut Vec<String>) -> Result<CollectionType> {
        expect_token(tokens, "<")?;
        let inner_type = match CQLType::check_kind_of_type(tokens)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError(("Tipo de dato invalido").to_string())),
        };
        expect_token(tokens, ">")?;
        Ok(CollectionType::List(Box::new(inner_type)))
    }

    fn parse_set_type(tokens: &mut Vec<String>) -> Result<CollectionType> {
        expect_token(tokens, "<")?;
        let inner_type = match CQLType::check_kind_of_type(tokens)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError(("Tipo de dato invalido").to_string())),
        };
        expect_token(tokens, ">")?;
        Ok(CollectionType::Set(Box::new(inner_type)))
    }

    fn parse_map_type(tokens: &mut Vec<String>) -> Result<CollectionType> {
        expect_token(tokens, "<")?;
        let key_type = match CQLType::check_kind_of_type(tokens)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError(("Tipo de dato invalido").to_string())),
        };
        expect_token(tokens, ",")?;
        let value_type = match CQLType::check_kind_of_type(tokens)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError(("Tipo de dato invalido").to_string())),
        };
        expect_token(tokens, ">")?;
        Ok(CollectionType::Map(
            Box::new(key_type),
            Box::new(value_type),
        ))
    }
}

/// Verifica si el token actual es el esperado. Si no es el esperado, retorna un error.
pub fn expect_token(tokens: &mut Vec<String>, expected: &str) -> Result<()> {
    if tokens.is_empty() || tokens[0] != expected {
        Err(Error::SyntaxError(format!("Expected token: {expected}")))
    } else {
        tokens.remove(0);
        Ok(())
    }
}
