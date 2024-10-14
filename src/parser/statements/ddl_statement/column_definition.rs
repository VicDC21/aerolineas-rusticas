use crate::{
    cassandra::errors::error::Error,
    parser::data_types::cql_type::cql_type::CQLType,
};

pub struct ColumnDefinition {
    pub name: String,
    pub data_type: CQLType,
    is_static: bool,
    primary_key: bool,
}

impl ColumnDefinition {
    pub fn new(name: String, data_type: CQLType, is_static: bool, primary_key: bool) -> Self {
        ColumnDefinition {
            name,
            data_type,
            is_static,
            primary_key,
        }
    }

    pub fn parse(lista: &mut Vec<String>) -> Result<Self, Error> {
        let name = lista.remove(0);
        let native_type = match CQLType::check_kind_of_type(lista)?{
            Some(value) => value,
            None => return Err(Error::SyntaxError(("Tipo de dato no soportado").to_string()))
        };
        let data_type = native_type;
        let is_static = !lista.is_empty() && lista.remove(0) == "STATIC";
        let primary_key = true; // ESTA HARDCODEADO, REVISAR
        Ok(ColumnDefinition {
            name,
            data_type,
            is_static,
            primary_key,
        })
    }
}
