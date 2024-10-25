//! Módulo que detalla una tabla.

use super::column_config::ColumnConfig;
use crate::parser::statements::dml_statement::main_statements::select::ordering::Ordering;

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
    pub clustering_key_and_order: Option<Vec<(String, Ordering)>>,
}

impl Table {
    /// Crea una nueva tabla.
    pub fn new(
        name: String,
        keyspace: String,
        columns: Vec<ColumnConfig>,
        partition_key: String,
        clustering_key_and_order: Option<Vec<(String, Ordering)>>,
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
}
