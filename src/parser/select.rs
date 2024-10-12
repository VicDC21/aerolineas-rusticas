use super::{data_types::keyspace_name::KeyspaceName, order_by::OrderBy, selector::Selector};
use crate::parser::{
    group_by::GroupBy, limit::Limit, per_partition_limit::PerPartitionLimit, r#where::Where,
};

pub struct Select {
    columns: KindOfColumns,
    from: KeyspaceName,
    the_where: Option<Where>,
    group_by: Option<GroupBy>,
    order_by: Option<OrderBy>,
    per_partition_limit: Option<PerPartitionLimit>,
    limit: Option<Limit>,
    allow_filtering: Option<bool>,
}

impl Select {
    pub fn new(
        columns: KindOfColumns,
        from: KeyspaceName,
        the_where: Option<Where>,
        group_by: Option<GroupBy>,
        order_by: Option<OrderBy>,
        per_partition_limit: Option<PerPartitionLimit>,
        limit: Option<Limit>,
        allow_filtering: Option<bool>,
    ) -> Select {
        Select {
            columns,
            from,
            the_where,
            group_by,
            order_by,
            per_partition_limit,
            limit,
            allow_filtering,
        }
    }
}

#[derive(Default)]

pub enum KindOfColumns {
    SelectClause(Vec<Selector>),
    #[default]
    All,
}

#[derive(Default)]
pub struct SelectBuilder {
    columns: KindOfColumns,
    from: KeyspaceName,
    the_where: Option<Where>,
    group_by: Option<GroupBy>,
    order_by: Option<OrderBy>,
    per_partition_limit: Option<PerPartitionLimit>,
    limit: Option<Limit>, // ver como crear los structs
    allow_filtering: Option<bool>,
}

impl SelectBuilder {
    pub fn set_select_clause(&mut self, clause: KindOfColumns) {
        self.columns = clause;
    }
    pub fn set_from(&mut self, table_name: KeyspaceName) {
        self.from = table_name;
    }
    pub fn set_where(&mut self, r#where: Option<Where>) {
        self.the_where = r#where;
    }
    pub fn set_group_by(&mut self, group_by: Option<GroupBy>) {
        self.group_by = group_by;
    }
    pub fn set_order_by(&mut self, order_by: Option<OrderBy>) {
        self.order_by = order_by;
    }
    pub fn set_per_partition_limit(&mut self, per_partition_limit: Option<PerPartitionLimit>) {
        self.per_partition_limit = per_partition_limit;
    }
    pub fn set_limit(&mut self, limit: Option<Limit>) {
        self.limit = limit
    }
    pub fn set_allow_filtering(&mut self, allow_filtering: Option<bool>) {
        self.allow_filtering = allow_filtering;
    }
    pub fn build(self) -> Select {
        Select::new(
            self.columns,
            self.from,
            self.the_where,
            self.group_by,
            self.order_by,
            self.per_partition_limit,
            self.limit,
            self.allow_filtering,
        )
    }
}
