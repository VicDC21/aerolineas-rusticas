use super::cql_type::CQLType;
/// tuple_type::= TUPLE '<' cql_type( ',' cql_type)* '>'
use crate::cassandra::errors::error::Error;
use crate::parser::data_types::cql_type::collection_type::expect_token;

pub enum TupleType {
    Tuple(Box<Vec<CQLType>>),
}

impl TupleType {
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
