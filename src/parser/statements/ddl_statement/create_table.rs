use crate::parser::{primary_key::PrimaryKey, table_name::TableName};

use super::column_definition::ColumnDefinition;

pub struct CreateTable {
    pub if_not_exists: bool,
    pub name: TableName,
    pub columns: Vec<ColumnDefinition>,
    pub primary_key: Option<PrimaryKey>,
    compact_storage: bool,
    clustering_order: Option<Vec<(String, String)>>,
}

impl CreateTable {
    pub fn new(
        if_not_exists: bool,
        name: TableName,
        columns: Vec<ColumnDefinition>,
        primary_key: Option<PrimaryKey>,
        compact_storage: bool,
        clustering_order: Option<Vec<(String, String)>>,
    ) -> Self {
        CreateTable {
            if_not_exists,
            name,
            columns,
            primary_key,
            compact_storage,
            clustering_order,
        }
    }
}
