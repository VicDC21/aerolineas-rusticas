//! Módulo para vuelos entre 2 aeropuertos.

use chrono::{DateTime, TimeZone, Utc};

use crate::protocol::aliases::types::{Int, Long};

/// Un vuelo entre dos [aeropuertos](crate::data::airports::Airport).
pub struct Flight {
    /// El ID único del vuelo.
    pub id: Int,

    /// El [identificador](crate::data::airports::Airport::ident) del aeropuerto de origen.
    pub orig: String,

    /// El [identificador](crate::data::airports::Airport::ident) del aeropuerto de destino.
    pub dest: String,

    /// El momento en que el vuelo se generó.
    pub timestamp: Long,
}

impl Flight {
    /// Crea una nueva instancia de vuelo.
    pub fn new(id: Int, orig: String, dest: String, timestamp: Long) -> Self {
        Self {
            id,
            orig,
            dest,
            timestamp,
        }
    }

    /// Transforma el timestamp en una fecha.
    pub fn get_date(&self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.timestamp, 0).single()
    }
}
