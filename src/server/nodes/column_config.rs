//! Módulo que detalla la configuración de una columna.

use super::column_data_type::ColumnDataType;

/// Representa la configuración de una columna.
pub struct ColumnConfig {
    /// Nombre de la columna.
    pub name: String,
    /// Tipo de dato de una columna.
    pub data_type: ColumnDataType,
}

impl ColumnConfig {
    pub fn new(name: String, data_type: ColumnDataType) -> Self {
        ColumnConfig { name, data_type }
    }

    pub fn get_name(&self) -> String {
        self.name.to_string()
    }
}
