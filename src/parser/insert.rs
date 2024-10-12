use super::update_parameter::UpdateParameter;

pub struct Insert {
    table_name: String,
    columns: Vec<String>,
    values: Vec<String>,
    if_not_exists: Option<bool>,
    update_parameter: Option<UpdateParameter>,
}

impl Insert {
    pub fn new(
        table_name: String,
        columns: Vec<String>,
        values: Vec<String>,
        if_not_exists: Option<bool>,
        update_parameter: Option<UpdateParameter>,
        ) -> Insert {
        Insert {
            table_name,
            columns,
            values,
            if_not_exists,
            update_parameter,
        }
    }
}

pub struct InsertBuilder {
    table_name: String,
    columns: Vec<String>,
    values: Vec<String>,
    if_not_exists: Option<bool>,
    update_parameter: Option<UpdateParameter>,
}

impl InsertBuilder {
    fn set_table_name(&mut self, table_name: String) {
        self.table_name = table_name;
    }

    fn set_columns(&mut self, columns: Vec<String>) {
        self.columns = columns;
    }

    fn set_values(&mut self, set_values: Vec<String>) {
        self.set_values = set_values;
    }

    fn set_if_not_exists(&mut self, if_not_exists: Option<bool>) {
        self.if_not_exists = if_not_exists;
    }

    fn set_update_parameter(&mut self, update_parameter: Option<UpdateParameter>) {
        self.update_parameter = update_parameter;
    }

    fn build(self) -> Insert {
        Insert::new(
            self.table_name,
            self.columns,
            self.values,
            self.if_not_exists,
            self.update_parameter,
        )
    }
}
