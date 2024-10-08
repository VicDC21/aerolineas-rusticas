use super::data_types::keyspace_name::KeyspaceName;
use crate::parser::{
    group_by::GroupBy, limit::Limit, per_partition_limit::PerPartitionLimit, r#where::Where,
};

pub struct Select {
    select_clause: String,
    from: KeyspaceName,
    where_clause: Option<Where>,
    group_by: Option<GroupBy>,
    per_partition_limit: Option<PerPartitionLimit>,
    limit: Option<Limit>,
    allow_filtering: Option<bool>,
}
