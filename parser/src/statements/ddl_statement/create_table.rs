use crate::{
    data_types::cql_type::{
        cql_type_mod::CQLType,
        native_types::NativeType,
    },
    primary_key::PrimaryKey,
    statements::ddl_statement::column_definition::ColumnDefinition,
    table_name::TableName,
};
use protocol::{aliases::results::Result, errors::error::Error};
use server::nodes::{
    table_metadata::{
        column_config::ColumnConfig,
        column_data_type::ColumnDataType,
    },
};

/// Representa una declaración CREATE TABLE en CQL.
#[derive(Debug)]
pub struct CreateTable {
    /// Indica si se debe verificar la existencia de la tabla.
    pub if_not_exists: bool,
    /// Nombre de la tabla a crear.
    pub name: TableName,
    /// Columnas de la tabla.
    pub columns: Vec<ColumnDefinition>,
    /// Clave primaria de la tabla.
    pub primary_key: Option<PrimaryKey>,
    /// Indica si la tabla tiene almacenamiento compacto.
    pub compact_storage: bool,
    /// Orden de agrupamiento de las columnas.
    pub clustering_order: Option<Vec<(String, String)>>,
}

impl CreateTable {
    /// Crea una nueva instancia de `CreateTable`.
    pub fn new(
        if_not_exists: bool,
        name: TableName,
        columns: Vec<ColumnDefinition>,
        primary_key: Option<PrimaryKey>,
        compact_storage: bool,
        clustering_order: Option<Vec<(String, String)>>,
    ) -> Self {
        CreateTable {
            if_not_exists,
            name,
            columns,
            primary_key,
            compact_storage,
            clustering_order,
        }
    }

    /// Obtiene el nombre de la tabla.
    pub fn get_name(&self) -> String {
        self.name.get_name()
    }

    /// Obtiene el nombre del keyspace al que pertenece la tabla.
    pub fn get_keyspace(&self) -> Option<String> {
        self.name.get_keyspace()
    }

    /// Obtiene las columnas de la tabla.
    pub fn get_columns(&self) -> Result<Vec<ColumnConfig>> {
        let mut vec = Vec::new();
        for column in self.columns.iter() {
            let vec_column = column.get_column_name();
            let data_type: ColumnDataType = match column.get_data_type() {
                CQLType::NativeType(native_type) => self.get_cql_type(native_type)?,
                _ => {
                    return Err(Error::Invalid(
                        "Solo es soportado el tipo de dato nativo.".to_string(),
                    ))
                }
            };
            vec.push(ColumnConfig::new(vec_column, data_type));
        }
        Ok(vec)
    }

    fn get_cql_type(&self, native_type: &NativeType) -> Result<ColumnDataType> {
        match native_type {
            NativeType::Double => Ok(ColumnDataType::Double),
            NativeType::Int => Ok(ColumnDataType::Int),
            NativeType::Text => Ok(ColumnDataType::String),
            NativeType::TimeStamp => Ok(ColumnDataType::Timestamp),
            _ => Err(Error::SyntaxError(
                "No se proporciono un tipo de dato soportado".to_string(),
            )),
        }
    }
}
