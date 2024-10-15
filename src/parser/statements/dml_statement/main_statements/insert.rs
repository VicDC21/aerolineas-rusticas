use crate::parser::{
    data_types::{identifier::identifier::Identifier, literal::tuple_literal::TupleLiteral},
    table_name::TableName,
};

pub struct Insert {
    table_name: TableName,
    names: Vec<Identifier>,
    values: TupleLiteral,
    if_not_exists: bool,
}

impl Insert {
    pub fn new(
        table_name: TableName,
        names: Vec<Identifier>,
        values: TupleLiteral,
        if_not_exists: bool,
    ) -> Insert {
        Insert {
            table_name,
            names,
            values,
            if_not_exists,
        }
    }
}

pub struct InsertBuilder {
    table_name: TableName,
    names: Vec<Identifier>,
    values: TupleLiteral,
    if_not_exists: bool,
}

// impl InsertBuilder {
//     fn set_table_name(&mut self, table_name: String) {
//         self.table_name = table_name;
//     }

//     fn set_columns(&mut self, columns: Vec<String>) {
//         self.columns = columns;
//     }

//     fn set_values(&mut self, set_values: Vec<String>) {
//         self.values = set_values;
//     }

//     fn set_if_not_exists(&mut self, if_not_exists: Option<bool>) {
//         self.if_not_exists = if_not_exists;
//     }

//     fn set_update_parameter(&mut self, update_parameter: Option<UpdateParameter>) {
//         self.update_parameter = update_parameter;
//     }

//     fn build(self) -> Insert {
//         Insert::new(
//             self.table_name,
//             self.columns,
//             self.values,
//             self.if_not_exists,
//             self.update_parameter,
//         )
//     }
// }
