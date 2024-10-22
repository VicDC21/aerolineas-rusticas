/// Representa un límite por partición.
#[derive(Debug)]
pub struct PerPartitionLimit {
    /// Límite por partición.
    pub limit: i32, // bind _marker
}

impl PerPartitionLimit {
    /// Crea un nuevo límite por partición.
    pub fn new(limit: i32) -> Self {
        PerPartitionLimit { limit }
    }
}
