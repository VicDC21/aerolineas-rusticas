use super::data_types::keyspace_name::KeyspaceName;
use crate::parser::{
    group_by::GroupBy, limit::Limit, per_partition_limit::PerPartitionLimit, r#where::Where,
};

pub struct Select {
    select_clause: String,
    from: KeyspaceName,
    the_where: Option<Where>,
    group_by: Option<GroupBy>,
    per_partition_limit: Option<PerPartitionLimit>,
    limit: Option<Limit>,
    allow_filtering: Option<bool>,
}

impl Select {
    pub fn new(
        select_clause: String,
        from: KeyspaceName,
        the_where: Option<Where>,
        group_by: Option<GroupBy>,
        per_partition_limit: Option<PerPartitionLimit>,
        limit: Option<Limit>,
        allow_filtering: Option<bool>,
    ) -> Select {
        Select {
            select_clause,
            from,
            the_where,
            group_by,
            per_partition_limit,
            limit,
            allow_filtering,
        }
    }
}

pub struct SelectBuilder {
    select_clause: String,
    from: KeyspaceName,
    the_where: Option<Where>,
    group_by: Option<GroupBy>,
    per_partition_limit: Option<PerPartitionLimit>,
    limit: Option<Limit>, // ver como crear los structs
    allow_filtering: Option<bool>,
}

impl SelectBuilder {
    fn set_select_clause(&mut self, clause: String) {
        self.select_clause = clause;
    }
    fn set_from(&mut self, table_name: KeyspaceName) {
        self.from = table_name;
    }
    fn set_where(&mut self, r#where: Option<Where>) {
        self.the_where = r#where;
    }
    fn set_group_by(&mut self, group_by: Option<GroupBy>) {
        self.group_by = group_by;
    }
    fn set_per_partition_limit(&mut self, per_partition_limit: Option<PerPartitionLimit>) {
        self.per_partition_limit = per_partition_limit;
    }
    fn set_limit(&mut self, limit: Option<Limit>) {
        self.limit = limit
    }
    fn set_allow_filtering(&mut self, allow_filtering: Option<bool>) {
        self.allow_filtering = allow_filtering;
    }
    fn build(self) -> Select {
        Select::new(
            self.select_clause,
            self.from,
            self.the_where,
            self.group_by,
            self.per_partition_limit,
            self.limit,
            self.allow_filtering,
        )
    }
}
