//! Módulo que detalla el tipo de dato de una columna.

use std::{fmt, str::FromStr};

use crate::protocol::{
    aliases::results::Result, errors::error::Error, messages::responses::result::col_type::ColType,
};

/// Representa el tipo de dato de una columna.
#[derive(Clone, Debug)]
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

impl fmt::Display for ColumnDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ColumnDataType::String => "String",
            ColumnDataType::Timestamp => "Timestamp",
            ColumnDataType::Double => "Double",
            ColumnDataType::Int => "Int",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for ColumnDataType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "String" => Ok(ColumnDataType::String),
            "Timestamp" => Ok(ColumnDataType::Timestamp),
            "Double" => Ok(ColumnDataType::Double),
            "Int" => Ok(ColumnDataType::Int),
            _ => Err(Error::ServerError(
                "No se pudo parsear el tipo de dato de la columna".to_string(),
            )),
        }
    }
}
