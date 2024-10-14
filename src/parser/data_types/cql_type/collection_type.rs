use crate::cassandra::errors::error::Error;

use super::native_types::{parse_data_type, NativeType};

pub enum CollectionType {
    /// MAP '<' cql_type',' cql_type'>'
    Map(Box<NativeType>, Box<NativeType>),

    /// SET '<' cql_type '>'
    Set(Box<NativeType>),

    /// LIST '<' cql_type'>'
    List(Box<NativeType>),
}

fn parse_list_type(tokens: &mut Vec<String>) -> Result<CollectionType, Error> {
    expect_token(tokens, "<")?;
    let inner_type = parse_data_type(tokens)?;
    expect_token(tokens, ">")?;
    Ok(CollectionType::List(Box::new(inner_type)))
}

fn parse_set_type(tokens: &mut Vec<String>) -> Result<CollectionType, Error> {
    expect_token(tokens, "<")?;
    let inner_type = parse_data_type(tokens)?;
    expect_token(tokens, ">")?;
    Ok(CollectionType::Set(Box::new(inner_type)))
}

fn parse_map_type(tokens: &mut Vec<String>) -> Result<CollectionType, Error> {
    expect_token(tokens, "<")?;
    let key_type = parse_data_type(tokens)?;
    expect_token(tokens, ",")?;
    let value_type = parse_data_type(tokens)?;
    expect_token(tokens, ">")?;
    Ok(CollectionType::Map(
        Box::new(key_type),
        Box::new(value_type),
    ))
}

fn expect_token(tokens: &mut Vec<String>, expected: &str) -> Result<(), Error> {
    if tokens.is_empty() || tokens[0] != expected {
        Err(Error::SyntaxError(format!("Expected token: {}", expected)))
    } else {
        tokens.remove(0);
        Ok(())
    }
}
