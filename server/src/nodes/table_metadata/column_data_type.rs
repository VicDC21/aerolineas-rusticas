//! Módulo que detalla el tipo de dato de una columna.

use {
    protocol::messages::responses::result::col_type::ColType,
    serde::{Deserialize, Serialize},
};

/// Representa el tipo de dato de una columna.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ColumnDataType {
    /// Tipo de dato `String`.
    String,
    /// Tipo de dato `Timestamp`.
    Timestamp,
    /// Tipo de dato `Double`.
    Double,
    /// Tipo de dato `Int`.
    Int,
}

impl From<ColType> for ColumnDataType {
    fn from(col: ColType) -> Self {
        match col {
            ColType::Varchar => ColumnDataType::String,
            ColType::Timestamp => ColumnDataType::Timestamp,
            ColType::Double => ColumnDataType::Double,
            ColType::Int => ColumnDataType::Int,
            _ => ColumnDataType::String,
        }
    }
}

impl Into<ColType> for &ColumnDataType {
    fn into(self) -> ColType {
        match self {
            Self::String => ColType::Varchar,
            Self::Timestamp => ColType::Timestamp,
            Self::Double => ColType::Double,
            Self::Int => ColType::Int,
        }
    }
}
