/// Flags individuales para keyspaces
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum KeyspaceFlag {
    /// Flag para estrategia de replicación
    ReplicationStrategy = 0x01,
    /// Flag para factor de replicación
    ReplicationFactor = 0x02,
    /// Flag para durable writes
    DurableWrites = 0x04,
}
