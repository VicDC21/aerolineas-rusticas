use crate::{
    parser::data_types::{cql_type::cql_type::CQLType, identifier::identifier::Identifier},
    parser::statements::ddl_statement::ddl_statement_parser::check_words,
    protocol::errors::error::Error,
};

/// column_definition::= column_name cql_type [ STATIC ] [ PRIMARY KEY]
#[derive(Debug, PartialEq)]
pub struct ColumnDefinition {
    /// column_name::= identifier  
    pub column_name: Identifier,
    /// cql_type
    pub data_type: CQLType,
    /// [ STATIC ] // cuando no esta lo tomamos como false
    pub is_static: bool,
    /// [ PRIMARY KEY ] // cuando no esta lo tomamos como false
    pub primary_key: bool,
}

impl ColumnDefinition {
    /// Crea una nueva instancia de `ColumnDefinition`.
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

    /// Analiza una lista de cadenas para crear una `ColumnDefinition`.
    ///
    /// # Argumentos
    ///
    /// * `lista` - Una referencia mutable a un vector de cadenas que representa la definición de la columna.
    ///
    /// # Retornos
    ///
    /// * `Result<Self, Error>` - Devuelve una `ColumnDefinition` en caso de éxito o un `Error` en caso de fallo.
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
