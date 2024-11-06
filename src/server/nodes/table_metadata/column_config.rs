//! Módulo que detalla la configuración de una columna.

use std::{fmt, str::FromStr};

use crate::protocol::{aliases::results::Result, errors::error::Error};

use super::column_data_type::ColumnDataType;

/// Representa la configuración de una columna.
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

impl fmt::Display for ColumnConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}", self.name, self.data_type)
    }
}

impl FromStr for ColumnConfig {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 2 {
            return Err(Error::ServerError(
                "No se pudo parsear la columna".to_string(),
            ));
        }

        let name: String = parts[0].to_string();
        let data_type: ColumnDataType = parts[1].parse()?;

        Ok(ColumnConfig::new(name, data_type))
    }
}
