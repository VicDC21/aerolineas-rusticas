use crate::parser::{
    data_types::keyspace_name::KeyspaceName,
    statements::dml_statement::{if_condition::IfCondition, r#where::r#where_parser::Where},
};

pub struct Delete {
    cols: Vec<String>,
    from: KeyspaceName,
    the_where: Option<Where>,
    if_condition: Option<IfCondition>,
}

impl Delete {
    pub fn new(
        cols: Vec<String>,
        from: KeyspaceName,
        the_where: Option<Where>,
        if_condition: Option<IfCondition>,
    ) -> Delete {
        Delete {
            cols,
            from,
            the_where,
            if_condition,
        }
    }
}

#[derive(Default)]
pub struct DeleteBuilder {
    cols: Vec<String>,
    from: KeyspaceName,
    the_where: Option<Where>,
    if_condition: Option<IfCondition>,
}

impl DeleteBuilder {
    pub fn set_cols(&mut self, cols: Vec<String>) {
        self.cols = cols;
    }
    pub fn set_from(&mut self, from: KeyspaceName) {
        self.from = from;
    }
    pub fn set_where(&mut self, r#where: Option<Where>) {
        self.the_where = r#where;
    }
    pub fn set_if_condition(&mut self, if_condition: Option<IfCondition>) {
        self.if_condition = if_condition;
    }

    pub fn build(self) -> Delete {
        Delete::new(self.cols, self.from, self.the_where, self.if_condition)
    }
}
