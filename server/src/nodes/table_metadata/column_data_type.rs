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

/// Re turbio. Implementar un trait fuera de [ColType] significa que estas versiones de
/// `from()` e `into()` sólo van a funcar dentro de esta _crate_ y no en otro lado,
/// así que ojo con usarlo.
///
/// Era la única forma de hacer callar el _warning_. Implementar [Into] para [ColumnDataType]
/// no le gusta tampoco porque [From] implementa eso implícitamente.
impl From<&ColumnDataType> for ColType {
    fn from(value: &ColumnDataType) -> Self {
        match value {
            ColumnDataType::String => Self::Varchar,
            ColumnDataType::Timestamp => Self::Timestamp,
            ColumnDataType::Double => Self::Double,
            ColumnDataType::Int => Self::Int,
        }
    }
}
