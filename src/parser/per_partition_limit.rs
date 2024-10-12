pub struct PerPartitionLimit {
    limit: i32
    // bind _marker
}

impl PerPartitionLimit{
    pub fn new(limit: i32) ->Self{
        PerPartitionLimit{limit}
    }
}