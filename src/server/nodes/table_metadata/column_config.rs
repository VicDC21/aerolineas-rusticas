//! Módulo que detalla la configuración de una columna.

use serde::{Deserialize, Serialize};

use super::column_data_type::ColumnDataType;

/// Representa la configuración de una columna.
#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnConfig {
    /// Nombre de la columna.
    pub name: String,
    /// Tipo de dato de una columna.
    pub data_type: ColumnDataType,
}

impl ColumnConfig {
    /// Crea una nueva configuración de columna.
    pub fn new(name: String, data_type: ColumnDataType) -> Self {
        ColumnConfig { name, data_type }
    }

    /// Obtiene el nombre de la columna.
    pub fn get_name(&self) -> String {
        self.name.to_string()
    }

    /// Obtiene el tipo de dato de la columna.
    pub fn get_data_type(&self) -> ColumnDataType {
        self.data_type.clone()
    }
}
