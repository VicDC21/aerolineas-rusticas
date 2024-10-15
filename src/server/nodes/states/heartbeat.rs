//! Módulo para el _Heartbeat State_ de un nodo.

use chrono::Utc;
use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    convert::TryFrom,
};

use crate::protocol::{aliases::types::Byte, errors::error::Error, traits::Byteable};

/// El alias para el número de generación.
pub type GenType = i64;

/// El alias para el número de versión.
pub type VerType = u64;

/// Estructura para el _Heartbeat State_ de un nodo.
#[derive(Clone)]
pub struct HeartbeatState {
    /// Momento de generación del nodo.
    gen: GenType,

    /// Tiempo desde la creación del nodo.
    ver: VerType,
}

impl HeartbeatState {
    /// Genera un nuevo estado instantáneo.
    pub fn new(gen: i64, ver: u64) -> Self {
        Self { gen, ver }
    }

    /// Aumenta en 1 la versión del estado.
    pub fn beat(&mut self) -> VerType {
        self.ver += 1;
        self.ver
    }

    /// Devuelve el estado como una tupla.
    pub fn as_tuple(&self) -> (GenType, VerType) {
        (self.gen, self.ver)
    }

    /// Devuelve un estado que siempre va a ser menor que otros.
    pub fn minimal() -> Self {
        Self::new(0, 0)
    }
}

impl Default for HeartbeatState {
    fn default() -> Self {
        Self::new(Utc::now().timestamp(), 0)
    }
}

impl PartialEq for HeartbeatState {
    fn eq(&self, other: &Self) -> bool {
        (self.gen == other.gen) && (self.ver == other.ver)
    }
}

impl PartialOrd for HeartbeatState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.gen < other.gen {
            Some(Ordering::Less)
        } else if self.gen > other.gen {
            Some(Ordering::Greater)
        } else if self.ver < other.ver {
            Some(Ordering::Less)
        } else if self.ver > other.ver {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Byteable for HeartbeatState {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes = Vec::<Byte>::new();
        bytes.extend_from_slice(&self.gen.to_be_bytes());
        bytes.extend_from_slice(&self.ver.to_be_bytes());
        bytes
    }
}

impl TryFrom<&[Byte]> for HeartbeatState {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self, Self::Error> {
        let bytes_len = bytes.len();
        if bytes_len < 16 {
            return Err(Error::ServerError(format!(
                "Se esperaba al menos 16 bytes para el estado de heartbeat, no {}.",
                bytes_len
            )));
        }

        let mut i = 0;
        let gen = i64::from_be_bytes([
            bytes[i],
            bytes[i + 1],
            bytes[i + 2],
            bytes[i + 3],
            bytes[i + 4],
            bytes[i + 5],
            bytes[i + 6],
            bytes[i + 7],
        ]);
        i += 8;

        let ver = u64::from_be_bytes([
            bytes[i],
            bytes[i + 1],
            bytes[i + 2],
            bytes[i + 3],
            bytes[i + 4],
            bytes[i + 5],
            bytes[i + 6],
            bytes[i + 7],
        ]);

        Ok(HeartbeatState::new(gen, ver))
    }
}
