//! Módulo que detalla una estrategia de replicación de un keyspace.

/// Representa una estrategia de replicación.
pub enum ReplicationStrategy {
    /// SimpleStrategy(replicas)
    SimpleStrategy(u32),
    /// NetworkTopologyStrategy(datacenter_and_replicas)
    NetworkTopologyStrategy(Vec<(String, u32)>),
}

impl ReplicationStrategy {
    /// Crea una nueva estrategia de replicación simple.
    pub fn new_simple(replicas: u32) -> Self {
        ReplicationStrategy::SimpleStrategy(replicas)
    }

    /// Crea una nueva estrategia de replicación de red.
    pub fn new_network(datacenter_and_replicas: Vec<(String, u32)>) -> Self {
        ReplicationStrategy::NetworkTopologyStrategy(datacenter_and_replicas)
    }

    /// Obtiene la cantidad de réplicas de la estrategia de replicación simple.
    /// Si no es estrategia simple, retorna None.
    pub fn simple_replicas(&self) -> Option<u32> {
        match self {
            ReplicationStrategy::SimpleStrategy(replicas) => Some(*replicas),
            _ => None,
        }
    }
}
