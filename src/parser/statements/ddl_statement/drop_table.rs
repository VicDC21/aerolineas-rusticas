use crate::parser::table_name::TableName;

/// Representa una sentencia CQL DROP TABLE.
#[derive(Debug)]
pub struct DropTable {
    /// Nombre de la tabla a eliminar.
    pub table_name: TableName,
}

impl DropTable {
    /// Crea una nueva instancia de `DropTable`.
    pub fn new(table_name: TableName) -> Self {
        DropTable { table_name }
    }
}
