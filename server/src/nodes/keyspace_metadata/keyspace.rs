//! Módulo que detalla un keyspace.

use {
    crate::nodes::keyspace_metadata::replication_strategy::ReplicationStrategy,
    protocol::aliases::types::Uint,
    serde::{Deserialize, Serialize},
};

/// Representa un keyspace en CQL.
#[derive(Serialize, Deserialize)]
pub struct Keyspace {
    /// Nombre del keyspace.
    pub name: String,
    /// Estrategia de replicación del keyspace.
    pub replication: ReplicationStrategy,
}

impl Keyspace {
    /// Crea un nuevo keyspace.
    pub fn new(name: String, replication: ReplicationStrategy) -> Self {
        Keyspace { name, replication }
    }

    /// Obtiene el nombre del keyspace.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Obtiene la cantidad de réplicas de la estrategia de replicación simple.
    /// Si no es estrategia simple, retorna None.
    pub fn simple_replicas(&self) -> Option<Uint> {
        self.replication.simple_replicas()
    }

    /// Establece la estrategia de replicación del keyspace.
    pub fn set_replication(&mut self, replication: ReplicationStrategy) {
        self.replication = replication;
    }
}
