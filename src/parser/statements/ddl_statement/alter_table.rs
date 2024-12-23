use crate::parser::{
    statements::ddl_statement::{column_definition::ColumnDefinition, option::Options},
    table_name::TableName,
};

/// Representa diferentes instrucciones que se pueden aplicar para alterar una tabla.
#[derive(Debug, PartialEq)]
pub enum AlterTableInstruction {
    /// Agregar columnas a la tabla.
    AddColumns(bool, Vec<ColumnDefinition>),
    /// Eliminar columnas de la tabla.
    DropColumns(bool, Vec<String>),
    /// Modificar opciones de la tabla.
    WithOptions(bool, Vec<Options>),
    /// Renombrar columnas en la tabla.
    RenameColumns(bool, Vec<(String, String)>),
}

/// Representa una sentencia CQL `ALTER TABLE`.
#[derive(Debug)]
pub struct AlterTable {
    /// El nombre de la tabla a alterar.
    pub name: TableName,
    /// La instrucción a aplicar.
    pub instruction: AlterTableInstruction,
}

impl AlterTable {
    /// Crea una nueva instancia de `AlterTable`.
    pub fn new(name: TableName, instruction: AlterTableInstruction) -> Self {
        AlterTable { name, instruction }
    }
}
