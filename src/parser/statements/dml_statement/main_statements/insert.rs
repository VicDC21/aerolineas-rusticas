use crate::parser::{
    data_types::{identifier::identifier::Identifier, literal::tuple_literal::TupleLiteral},
    table_name::TableName,
};

/// Representa una declaración CQL INSERT.
#[derive(Debug)]
pub struct Insert {
    /// Nombre de la tabla a insertar.
    pub table: TableName,
    /// Lista de nombres de columnas.
    pub names: Vec<Identifier>,
    /// Lista de valores a insertar.
    pub values: TupleLiteral,
    /// Indica si la inserción debe realizarse solo si no existe.
    pub if_not_exists: bool,
}

impl Insert {
    /// Crea una nueva sentencia INSERT.
    pub fn new(
        table: TableName,
        names: Vec<Identifier>,
        values: TupleLiteral,
        if_not_exists: bool,
    ) -> Insert {
        Insert {
            table,
            names,
            values,
            if_not_exists,
        }
    }
}
