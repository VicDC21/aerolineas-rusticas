//! Módulo que detalla un keyspace.

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use crate::protocol::{aliases::results::Result, errors::error::Error};

use super::replication_strategy::ReplicationStrategy;

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
    pub fn simple_replicas(&self) -> Option<u32> {
        self.replication.simple_replicas()
    }

    /// Establece la estrategia de replicación del keyspace.
    pub fn set_replication(&mut self, replication: ReplicationStrategy) {
        self.replication = replication;
    }
}

impl fmt::Display for Keyspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.name, self.replication)
    }
}

impl FromStr for Keyspace {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(Error::ServerError(
                "No se pudo parsear el keyspace".to_string(),
            ));
        }

        let name: String = parts[0].to_string();
        let replication: ReplicationStrategy = parts[1].parse()?;

        Ok(Keyspace::new(name, replication))
    }
}
