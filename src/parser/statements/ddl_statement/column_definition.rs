use crate::{cassandra::errors::error::Error, parser::data_types::cql_type::{cql_type::CQLType, native_types::parse_data_type}};


pub struct ColumnDefinition {
    pub name: String,
    pub data_type: CQLType,
    is_static: bool,
    primary_key: bool
}

impl ColumnDefinition {
    pub fn new(name: String, data_type: CQLType, is_static: bool, primary_key: bool) -> Self {
        ColumnDefinition {
            name,
            data_type,
            is_static,
            primary_key
        }
    }

    pub fn parse(lista: &mut Vec<String>) -> Result<Self, Error> {
        let name = lista.remove(0);
        let native_type = parse_data_type(lista)?;
        let data_type = CQLType::NativeType(native_type);
        let is_static = !lista.is_empty() && lista.remove(0) == "STATIC";
        let primary_key = true; // ESTA HARDCODEADO, REVISAR
        Ok(ColumnDefinition {
            name,
            data_type,
            is_static,
            primary_key
        })
    }
}
