//! Módulo que detalla una estrategia de replicación de un keyspace.

use std::{fmt, str::FromStr};

use crate::protocol::{aliases::results::Result, errors::error::Error};

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

impl fmt::Display for ReplicationStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplicationStrategy::SimpleStrategy(replicas) => {
                write!(f, "SimpleStrategy~{}", replicas)
            }
            ReplicationStrategy::NetworkTopologyStrategy(_datacenter_and_replicas) => {
                todo!()
            }
        }
    }
}

impl FromStr for ReplicationStrategy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('~').collect();
        if parts.len() != 2 {
            return Err(Error::ServerError(
                "No se pudo parsear la estrategia de replicación".to_string(),
            ));
        }

        let replicas: u32 = parts[1].parse().map_err(|_| {
            Error::ServerError(
                "No se pudo parsear la cantidad de réplicas de la estrategia de replicación"
                    .to_string(),
            )
        })?;

        match parts[0] {
            "SimpleStrategy" => Ok(ReplicationStrategy::SimpleStrategy(replicas)),
            "NetworkTopologyStrategy" => todo!(),
            _ => Err(Error::ServerError(
                "No se pudo parsear la estrategia de replicación".to_string(),
            )),
        }
    }
}
