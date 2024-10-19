use crate::parser::table_name::TableName;

/// Representa una sentencia CQL DROP TABLE.
#[derive(Debug)]
pub struct DropTable {
    /// Indica si se debe verificar la existencia de la tabla.
    pub if_exist: bool,
    /// Nombre de la tabla a eliminar.
    pub table_name: TableName,
}

impl DropTable {
    /// Crea una nueva instancia de `DropTable`.
    pub fn new(if_exist: bool, table_name: TableName) -> Self {
        DropTable {
            if_exist,
            table_name,
        }
    }
}
