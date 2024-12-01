//! Módulo que detalla una estrategia de replicación de un keyspace.

use serde::{Deserialize, Serialize};

/// Representa una estrategia de replicación.
#[derive(Serialize, Deserialize)]
pub enum ReplicationStrategy {
    /// SimpleStrategy(replicas)
    SimpleStrategy(u32),
    /// NetworkTopologyStrategy(datacenter_and_replicas)
    NetworkTopologyStrategy(Vec<(String, u32)>),
}

impl ReplicationStrategy {
    /// Obtiene la cantidad de réplicas de la estrategia de replicación simple.
    /// Si no es estrategia simple, retorna None.
    pub fn simple_replicas(&self) -> Option<u32> {
        match self {
            ReplicationStrategy::SimpleStrategy(replicas) => Some(*replicas),
            _ => None,
        }
    }
}
