//! Módulo que detalla una estrategia de replicación de un keyspace.

use {
    protocol::aliases::types::Uint,
    serde::{Deserialize, Serialize},
};

/// Representa una estrategia de replicación.
#[derive(Serialize, Deserialize)]
pub enum ReplicationStrategy {
    /// SimpleStrategy(replicas)
    SimpleStrategy(Uint),
    /// NetworkTopologyStrategy(datacenter_and_replicas)
    NetworkTopologyStrategy(Vec<(String, Uint)>),
}

impl ReplicationStrategy {
    /// Obtiene la cantidad de réplicas de la estrategia de replicación simple.
    /// Si no es estrategia simple, retorna None.
    pub fn simple_replicas(&self) -> Option<Uint> {
        match self {
            ReplicationStrategy::SimpleStrategy(replicas) => Some(*replicas),
            _ => None,
        }
    }
}
