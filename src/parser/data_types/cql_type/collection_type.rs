use crate::protocol::errors::error::Error;

use super::cql_type::CQLType;

/// TODO: Desc básica
#[derive(Debug)]
pub enum CollectionType {
    /// MAP '<' cql_type',' cql_type'>'
    Map(Box<CQLType>, Box<CQLType>),

    /// SET '<' cql_type '>'
    Set(Box<CQLType>),

    /// LIST '<' cql_type'>'
    List(Box<CQLType>),
}

impl CollectionType {
    /// TODO: Desc básica
    pub fn parse_collection_type(
        tokens: &mut Vec<String>,
    ) -> Result<Option<CollectionType>, Error> {
        CollectionType::parse_list_type(tokens)?;
        CollectionType::parse_map_type(tokens)?;
        CollectionType::parse_set_type(tokens)?;
        Ok(None)
    }

    fn parse_list_type(tokens: &mut Vec<String>) -> Result<CollectionType, Error> {
        expect_token(tokens, "<")?;
        let inner_type = match CQLType::check_kind_of_type(tokens)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError(("Tipo de dato invalido").to_string())),
        };
        expect_token(tokens, ">")?;
        Ok(CollectionType::List(Box::new(inner_type)))
    }

    fn parse_set_type(tokens: &mut Vec<String>) -> Result<CollectionType, Error> {
        expect_token(tokens, "<")?;
        let inner_type = match CQLType::check_kind_of_type(tokens)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError(("Tipo de dato invalido").to_string())),
        };
        expect_token(tokens, ">")?;
        Ok(CollectionType::Set(Box::new(inner_type)))
    }

    fn parse_map_type(tokens: &mut Vec<String>) -> Result<CollectionType, Error> {
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

/// TODO: Desc básica
pub fn expect_token(tokens: &mut Vec<String>, expected: &str) -> Result<(), Error> {
    if tokens.is_empty() || tokens[0] != expected {
        Err(Error::SyntaxError(format!("Expected token: {}", expected)))
    } else {
        tokens.remove(0);
        Ok(())
    }
}
