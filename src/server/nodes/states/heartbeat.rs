//! Módulo para el _Heartbeat State_ de un nodo.

use chrono::Utc;
use std::cmp::{Ordering, PartialEq, PartialOrd};

/// El alias para el número de generación.
pub type GenType = i64;
/// El alias para el número de versión.
pub type VerType = u64;

/// Estructura para el _Heartbeat State_ de un nodo.
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
