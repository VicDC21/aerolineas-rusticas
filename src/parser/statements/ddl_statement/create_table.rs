use crate::parser::{primary_key::PrimaryKey, table_name::TableName};

use super::column_definition::ColumnDefinition;

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
}
