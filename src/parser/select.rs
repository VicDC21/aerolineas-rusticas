use super::data_types::keyspace_name::KeyspaceName;
use super::r#where::Where;

pub struct Select {
    select_clause: String,
    from: KeyspaceName,
    the_where: Option<Where>,
    // group_by: Option<GroupBy>,
    // per_partition_limit: Option<PerPartitionLimit>,
    // limit: Option<Limit>,  // ver como crear los structs
    allow_filtering: Option<bool>,
}
