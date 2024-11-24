//! Módulo para plugin que va actualizando los vuelos en curso.

use std::sync::Arc;

use eframe::egui::{Painter, Response};
use walkers::{Plugin, Projector};

use crate::interface::plugins::flights::loader::LiveDataMap;

/// Actualizador de aeropuertos.
pub struct FlightsUpdater {
    /// Los datos de vuelos entrantes actualmente en memoria.
    incoming_tracking: Arc<LiveDataMap>,

    /// Los datos de vuelos salientes actualmente en memoria.
    departing_tracking: Arc<LiveDataMap>,
}

impl FlightsUpdater {
    /// Crea una nueva instancia del simulador de vuelos.
    pub fn new(incoming_tracking: Arc<LiveDataMap>, departing_tracking: Arc<LiveDataMap>) -> Self {
        Self {
            incoming_tracking,
            departing_tracking,
        }
    }

    /// Sincroniza los datos de vuelo entrantes.
    pub fn sync_incoming_tracking(&mut self, new_inc_tracking: Arc<LiveDataMap>) -> &mut Self {
        if !new_inc_tracking.is_empty() {
            self.incoming_tracking = new_inc_tracking;
        }

        self
    }

    /// Sincroniza los datos de vuelo salientes.
    pub fn sync_departing_tracking(&mut self, new_dep_tracking: Arc<LiveDataMap>) -> &mut Self {
        if !new_dep_tracking.is_empty() {
            self.departing_tracking = new_dep_tracking;
        }

        self
    }
}

impl Default for FlightsUpdater {
    fn default() -> Self {
        Self::new(Arc::new(LiveDataMap::new()), Arc::new(LiveDataMap::new()))
    }
}

impl Plugin for &mut FlightsUpdater {
    fn run(&mut self, _response: &Response, _painter: Painter, _projector: &Projector) {}
}
