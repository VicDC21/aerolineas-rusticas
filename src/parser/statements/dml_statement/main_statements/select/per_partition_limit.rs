use crate::protocol::aliases::types::Int;

#[derive(Debug)]
/// Representa un límite por partición.
pub struct PerPartitionLimit {
    /// Límite por partición.
    pub limit: Int, // bind _marker
}

impl PerPartitionLimit {
    /// Crea un nuevo límite por partición.
    pub fn new(limit: Int) -> Self {
        PerPartitionLimit { limit }
    }
}
