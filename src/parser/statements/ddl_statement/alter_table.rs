use super::column_definition::ColumnDefinition;
use super::option::Options;
use crate::parser::table_name::TableName;

pub enum AlterTableInstruction {
    AddColumns(Vec<ColumnDefinition>),
    DropColumns(Vec<String>),
    WithOptions(Vec<Options>),
}

pub struct AlterTable {
    pub name: TableName,
    pub instruction: AlterTableInstruction,
}

impl AlterTable {
    pub fn new(name: TableName, instruction: AlterTableInstruction) -> Self {
        AlterTable { name, instruction }
    }
}
