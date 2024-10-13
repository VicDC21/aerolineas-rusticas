use crate::parser::data_types::option::Options;
use crate::parser::{
    column_definition::ColumnDefinition, primary_key::PrimaryKey, table_name::TableName,
};

pub struct CreateTable {
    pub if_not_exists: bool,
    pub name: TableName,
    pub columns: Vec<ColumnDefinition>,
    pub primary_key: Option<PrimaryKey>,
    pub options: Option<Vec<Options>>,
}

impl CreateTable {
    pub fn new(
        if_not_exists: bool,
        name: TableName,
        columns: Vec<ColumnDefinition>,
        primary_key: Option<PrimaryKey>,
        options: Option<Vec<Options>>,
    ) -> Self {
        CreateTable {
            if_not_exists,
            name,
            columns,
            primary_key,
            options,
        }
    }
}
