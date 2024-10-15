use crate::{
    cassandra::errors::error::Error,
    parser::data_types::{cql_type::cql_type::CQLType, identifier::identifier::Identifier},
    parser::statements::ddl_statement::ddl_statement_parser::check_words,
};

/// column_definition::= column_name cql_type [ STATIC ] [ PRIMARY KEY]
pub struct ColumnDefinition {
    /// column_name::= identifier  
    pub column_name: Identifier,
    /// cql_type
    pub data_type: CQLType,
    /// [ STATIC ] // cuando no esta lo tomamos como false
    is_static: bool,
    ///
    primary_key: bool,
}

impl ColumnDefinition {
    pub fn new(
        column_name: Identifier,
        data_type: CQLType,
        is_static: bool,
        primary_key: bool,
    ) -> Self {
        ColumnDefinition {
            column_name,
            data_type,
            is_static,
            primary_key,
        }
    }

    pub fn parse(lista: &mut Vec<String>) -> Result<Self, Error> {
        let column_name = match Identifier::check_identifier(lista)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "El nombre de la columna no es valido".to_string(),
                ))
            }
        };
        let data_type = match CQLType::check_kind_of_type(lista)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    ("Tipo de dato no soportado").to_string(),
                ))
            }
        };
        let is_static = check_words(lista, "STATIC");
        let primary_key = check_words(lista, "PRIMARY KEY");
        Ok(ColumnDefinition {
            column_name,
            data_type,
            is_static,
            primary_key,
        })
    }
}
