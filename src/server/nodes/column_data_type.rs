//! Módulo que detalla el tipo de dato de una columna.

/// Representa el tipo de dato de una columna.
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
