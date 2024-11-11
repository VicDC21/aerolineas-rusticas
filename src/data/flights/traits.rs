//! Módulo para _traits_ en común de los vuelos.

use chrono::{DateTime, Utc};

/// Un vuelo, ya sea entrante o saliente, con lógica en común.
pub trait Flight {
    /// Genera una instancia con el único propósito de funcionar con _matches_,
    /// no es útil para otra cosa.
    fn dummy() -> Self;

    /// Transforma el timestamp en una fecha.
    fn get_date(&self) -> Option<DateTime<Utc>>;
}
