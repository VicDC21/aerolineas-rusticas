use super::data_types::{cql_type::CQLType, native_types::parse_data_type};
use crate::cassandra::errors::error::Error;

pub struct ColumnDefinition {
    pub name: String,
    pub data_type: CQLType,
    is_static: bool,
}

impl ColumnDefinition {
    pub fn new(name: String, data_type: CQLType, is_static: bool) -> Self {
        ColumnDefinition {
            name,
            data_type,
            is_static,
        }
    }

    pub fn parse(lista: &mut Vec<String>) -> Result<Self, Error> {
        let name = lista.remove(0);
        let native_type = parse_data_type(lista)?;
        let data_type = CQLType::NativeType(native_type);
        let is_static = !lista.is_empty() && lista.remove(0) == "STATIC";
        Ok(ColumnDefinition {
            name,
            data_type,
            is_static,
        })
    }
}
