//! Módulo de cargador de aeropuertos.

use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

use eframe::egui::{Painter, Response};
use walkers::{Plugin, Position, Projector};

use crate::data::airports::Airport;
use crate::protocol::aliases::results::Result;

/// Un hilo destinado a procesos paralelos, tal que no bloquee el flujo sincrónico
/// del hilo principal.
pub type ChildHandle = JoinHandle<Result<()>>;

/// Cargador de aeropuertos.
pub struct AirportsLoader {
    /// Los aeropuertos actualmente en memoria.
    airports: Vec<Airport>,

    /// La última vez que [crate::interface::plugins::airports::loader::AirportsLoader::airports]
    /// fue modificado.
    last_checked: Instant,

    /// Hilo hijo, para correr procesos en paralelo.
    children: Vec<Option<ChildHandle>>,
}

impl AirportsLoader {
    /// Crea una nueva instancia del cargador de aeropuertos.
    pub fn new(
        airports: Vec<Airport>,
        last_checked: Instant,
        children: Vec<Option<ChildHandle>>,
    ) -> Self {
        Self {
            airports,
            last_checked,
            children,
        }
    }

    /// Resetea el chequeo al [Instant] actual.
    pub fn reset_instant(&mut self) {
        self.last_checked = Instant::now();
    }

    /// Actualiza el cache de aeropuertos dado una posición y distancia.
    pub fn update_airports_by_distance(&mut self, pos: &Position, min_distance: f64) -> Result<()> {
        self.reset_instant();
        self.airports = Airport::by_distance(pos, min_distance)?;
        Ok(())
    }

    /// Actualiza el cache de aeropuertos dado un área.
    pub fn update_airports_by_area(&mut self, area: (&Position, &Position)) -> Result<()> {
        self.reset_instant();
        self.airports = Airport::by_area(area)?;
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
        let mut handlers = Vec::<Option<ChildHandle>>::new();

        Self::new(Vec::new(), Instant::now(), handlers)
    }
}

impl Plugin for &mut AirportsLoader {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        let desired_time = Duration::from_secs(5);
        if response.dragged() && self.elapsed_at_least(&desired_time) {
            let geo_min = projector.unproject(response.rect.min.to_vec2());
            let geo_max = projector.unproject(response.rect.max.to_vec2());
            let area = (
                &Position::from_lat_lon(
                    geo_min.lat().min(geo_max.lat()),
                    geo_min.lon().min(geo_max.lon()),
                ),
                &Position::from_lat_lon(
                    geo_min.lat().max(geo_max.lat()),
                    geo_min.lon().max(geo_max.lon()),
                ),
            );

            if let Err(err) = self.update_airports_by_area(area) {
                println!("Error de aeropuertos:\n\n{}", err);
            } else {
                println!("¡Actualizado!\nAeropuertos: {}\n\n", self.airports.len());
            }
        }
    }
}

impl Drop for AirportsLoader {
    fn drop(&mut self) {
        for child in &mut self.children {
            if let Some(hanging) = child.take() {
                if hanging.join().is_err() {
                    println!("Error esperando a que un hilo hijo termine.")
                }
            }
        }
    }
}
