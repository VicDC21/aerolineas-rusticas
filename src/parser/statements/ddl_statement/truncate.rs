use crate::parser::table_name::TableName;

pub struct Truncate {
    table_name: TableName,
}

impl Truncate {
    pub fn new(table_name: TableName) -> Self {
        Truncate { table_name }
    }
}
