use super::{r#where::Where, set_parameter::SetParameter, update_parameter::UpdateParameter};

pub struct Update {
    table_name: String,
    update_parameter: Option<UpdateParameter>,
    set_parameter: Option<SetParameter>,
    the_where: Option<Where>,
    if_exists: Option<bool>,
}

impl Update {
    pub fn new(
        table_name: String,
        update_parameter: Option<UpdateParameter>,
        set_parameter: Option<SetParameter>,
        the_where: Option<Where>,
        if_exists: Option<bool>,
    ) -> Update {
        Update {
            table_name,
            update_parameter,
            set_parameter,
            the_where,
            if_exists,
        }
    }
}

#[derive(Default)]
pub struct UpdateBuilder {
    table_name: String,
    update_parameter: Option<UpdateParameter>,
    set_parameter: Option<SetParameter>,
    the_where: Option<Where>,
    if_exists: Option<bool>,
}

impl UpdateBuilder {
    pub fn set_table_name(&mut self, table_name: String) {
        self.table_name = table_name;
    }

    pub fn set_update_parameter(&mut self, update_parameter: Option<UpdateParameter>) {
        self.update_parameter = update_parameter;
    }
    pub fn set_set_parameter(&mut self, set_parameter: Option<SetParameter>) {
        self.set_parameter = set_parameter;
    }

    pub fn set_where(&mut self, r#where: Option<Where>) {
        self.the_where = r#where;
    }
    pub fn set_if_exists(&mut self, if_exists: Option<bool>) {
        self.if_exists = if_exists;
    }

    pub fn build(self) -> Update {
        Update::new(
            self.table_name,
            self.update_parameter,
            self.set_parameter,
            self.the_where,
            self.if_exists,
        )
    }
}
