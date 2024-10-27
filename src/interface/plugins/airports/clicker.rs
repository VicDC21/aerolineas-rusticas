//! Módulo para plugin que detecta clicks.

use std::sync::Arc;

use eframe::egui::{Painter, Response};
use walkers::{Plugin, Projector};

use crate::data::{airports::Airport, utils::distances::distance_euclidean_pos2};
use crate::interface::plugins::utils::zoom_is_showable;

// Distancia mínima para un potencial click.
const MIN_CLICK_DIST: f64 = 13.0;

/// Rastrea el mouse y decta clicks.
pub struct ScreenClicker {
    // La lista de aeropuertos actualmente en memoria.
    airports: Arc<Vec<Airport>>,

    // El nivel de zoom actual.
    zoom: f32,

    // El último aeropuerto clickeado.
    current_airport: Option<Option<Airport>>,
}

impl ScreenClicker {
    /// Crea una nueva instancia con la referencia al aeropuerto actual.
    pub fn new(
        airports: Arc<Vec<Airport>>,
        zoom: f32,
        current_airport: Option<Option<Airport>>,
    ) -> Self {
        Self {
            airports,
            zoom,
            current_airport,
        }
    }

    // Actualiza el valor de la lista de aeropuertos.
    ///
    /// Devuelve esta misma instancia para encadenar funciones.
    pub fn sync_airports(&mut self, real_airports: Arc<Vec<Airport>>) -> &mut Self {
        if !real_airports.is_empty() {
            self.airports = real_airports;
        }
        self
    }

    /// Actualiza el valor de zoom desde afuera.
    ///
    /// Devuelve esta misma instancia para encadenar funciones.
    pub fn sync_zoom(&mut self, real_zoom: f32) -> &mut Self {
        self.zoom = real_zoom;
        self
    }

    /// **Consume** el aeropuerto actualmente seleccionado y lo devuelve.
    /// En su lugar deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_cur_airport(&mut self) -> Option<Option<Airport>> {
        self.current_airport.take()
    }
}

impl Default for ScreenClicker {
    fn default() -> Self {
        Self::new(Arc::new(Vec::new()), 0.0, None)
    }
}

impl Plugin for &mut ScreenClicker {
    fn run(&mut self, response: &Response, _painter: Painter, projector: &Projector) {
        let cur_opt = response.interact_pointer_pos();
        if !response.clicked() {
            // Si arrastró o hizo otra cosa no nos interesa
            return;
        }

        if let Some(cur_pos) = cur_opt {
            for airport in self.airports.iter() {
                let airport_pos = projector.project(airport.position).to_pos2();
                if zoom_is_showable(&airport.airport_type, self.zoom)
                    && distance_euclidean_pos2(&cur_pos, &airport_pos) < MIN_CLICK_DIST
                {
                    self.current_airport = Some(Some(airport.clone()));
                    return;
                }
            }
        }

        // hubo click pero no cerca de ningún aeropuerto
        self.current_airport = Some(None);
    }
}
