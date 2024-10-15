use crate::parser::table_name::TableName;

/// Representa una declaración TRUNCATE en definicion de datos CQL.
pub struct Truncate {
    /// Nombre de la tabla a truncar.
    pub table_name: TableName,
}

impl Truncate {
    /// Crea una nueva instancia de `Truncate`.
    pub fn new(table_name: TableName) -> Self {
        Truncate { table_name }
    }
}
