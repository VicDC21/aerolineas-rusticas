//! Módulo que detalla una tabla.

use super::{column_config::ColumnConfig, column_data_type::ColumnDataType};
use crate::parser::statements::dml_statement::main_statements::select::ordering::ProtocolOrdering;

/// Representa una tabla en CQL.
pub struct Table {
    /// Nombre de la tabla.
    pub name: String,
    /// Nombre del keyspace al que pertenece la tabla.
    pub keyspace: String,
    /// Columnas de la tabla.
    pub columns: Vec<ColumnConfig>,
    /// Clave primaria de la tabla.
    pub partition_key: String,
    /// Clave de clustering de la tabla y orden de agrupamiento de las columnas.
    pub clustering_key_and_order: Option<Vec<(String, ProtocolOrdering)>>,
}

impl Table {
    /// Crea una nueva tabla.
    pub fn new(
        name: String,
        keyspace: String,
        columns: Vec<ColumnConfig>,
        partition_key: String,
        clustering_key_and_order: Option<Vec<(String, ProtocolOrdering)>>,
    ) -> Self {
        Table {
            name,
            keyspace,
            columns,
            partition_key,
            clustering_key_and_order,
        }
    }

    /// Obtiene el nombre de la tabla.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Obtiene la partition key de la tabla.
    pub fn get_partition_key(&self) -> String {
        self.partition_key.to_string()
    }

    /// Obtiene los nombres de las columnas de la tabla.
    pub fn get_columns_names(&self) -> Vec<String> {
        self.columns
            .iter()
            .map(|column| column.get_name())
            .collect()
    }

    /// Obtiene los tipos de datos de las columnas de la tabla.
    pub fn get_columns_data_type(&self) -> Vec<ColumnDataType> {
        self.columns
            .iter()
            .map(|column| column.get_data_type())
            .collect()
    }

    /// Obtiene los nombres y tipos de datos de las columnas de la tabla.
    pub fn get_columns_name_and_data_type(&self) -> Vec<(String, ColumnDataType)> {
        self.columns
            .iter()
            .map(|column| (column.get_name(), column.get_data_type()))
            .collect()
    }
}
