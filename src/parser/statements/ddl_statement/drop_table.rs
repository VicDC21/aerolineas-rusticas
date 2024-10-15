use crate::parser::table_name::TableName;

pub struct DropTable {
    if_exist: bool,
    table_name: TableName,
}

impl DropTable {
    pub fn new(if_exist: bool, table_name: TableName) -> Self {
        DropTable {
            if_exist,
            table_name,
        }
    }
}
