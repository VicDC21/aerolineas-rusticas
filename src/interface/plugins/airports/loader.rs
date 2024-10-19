//! Módulo de cargador de aeropuertos.

use std::time::{Duration, Instant};

use eframe::egui::{Painter, Response};
use walkers::{Plugin, Position, Projector};

use crate::data::airports::Airport;
use crate::protocol::aliases::results::Result;

/// Cargador de aeropuertos.
pub struct AirportsLoader {
    /// Los aeropuertos actualmente en memoria.
    airports: Vec<Airport>,

    /// La última vez que [crate::interface::plugins::airports::loader::AirportsLoader::airports]
    /// fue modificado.
    last_checked: Instant,
}

impl AirportsLoader {
    /// Crea una nueva instancia del cargador de aeropuertos.
    pub fn new(airports: Vec<Airport>, last_checked: Instant) -> Self {
        Self {
            airports,
            last_checked,
        }
    }

    /// Actualiza el cache de aeropuertos dado una posición y distancia.
    pub fn update_airports_by_distance(&mut self, pos: &Position, min_distance: f64) -> Result<()> {
        self.last_checked = Instant::now();
        self.airports = Airport::by_distance(pos, min_distance)?;
        Ok(())
    }

    /// Verifica si ha pasado un mínimo de tiempo dado desde la última vez
    /// que se editaron los puertos.
    pub fn elapsed_at_least(&self, duration: &Duration) -> bool {
        &self.last_checked.elapsed() >= duration
    }
}

impl Default for AirportsLoader {
    fn default() -> Self {
        Self::new(Vec::new(), Instant::now())
    }
}

impl Plugin for &mut AirportsLoader {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        let desired_time = Duration::from_secs(5);
        let center = response.rect.center();
        if response.dragged() && self.elapsed_at_least(&desired_time) {
            let geo_pos = projector.unproject(center.to_vec2());
            if let Err(err) = self.update_airports_by_distance(&geo_pos, 5.0) {
                println!("Error de aeropuertos:\n\n{}", err);
            } else {
                println!("¡Actualizado!\nAeropuertos: {}\n\n", self.airports.len());
            }
        }
    }
}
