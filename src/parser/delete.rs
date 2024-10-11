use super::{r#where::Where, update_parameter::UpdateParameter};

pub struct Delete {
    cols : Vec<String>,
    from: String,
    the_where: Option<Where>,
    update_parameter: Option<UpdateParameter>,
}

impl Delete {
    pub fn new(
        cols: Vec<String>,
        from: String,
        the_where: Option<Where>,
        update_parameter: Option<UpdateParameter>,
    ) -> Delete {
        Delete {
            cols,
            from,
            the_where,
            update_parameter
        }
    }
}

pub struct DeleteBuilder {
    cols : Vec<String>,
    from: String,
    the_where: Option<Where>,
    update_parameter: Option<UpdateParameter>,
}

impl DeleteBuilder {
    fn set_cols(&mut self, cols: Vec<String>) {
        self.cols = cols;
    }
    fn set_from(&mut self, table_name: String) {
        self.from = table_name;
    }
    fn set_where(&mut self, r#where: Option<Where>) {
        self.the_where = r#where;
    }
    fn set_update_parameter(&mut self, update_parameter: Option<UpdateParameter>) {
        self.update_parameter = update_parameter;
    }

    fn build(self) -> Delete {
        Delete::new(
            self.cols,
            self.from,
            self.the_where,
            self.update_parameter,
        )
    }
}
