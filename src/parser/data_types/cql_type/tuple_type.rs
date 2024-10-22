use super::cql_type::CQLType;
use crate::parser::data_types::cql_type::collection_type::expect_token;
/// tuple_type::= TUPLE '<' cql_type( ',' cql_type)* '>'
use crate::protocol::errors::error::Error;

/// TODO: Desc básica
#[derive(Debug)]
pub enum TupleType {
    /// TODO: Desc básica
    Tuple(Box<Vec<CQLType>>),
}

impl TupleType {
    /// TODO: Desc básica
    pub fn parse_tuple_type(tokens: &mut Vec<String>) -> Result<Option<TupleType>, Error> {
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
