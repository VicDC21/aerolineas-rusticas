//! Módulo que dibuja aeropuertos.

use eframe::egui::{Painter, Response, Rgba, Stroke};
use walkers::{Plugin, Projector};

use crate::data::{airport_types::AirportType, airports::Airport};

/// Este plugin se encarga de dibujar la información en pantalla de los
/// aeropuertos cargados por [AirportsLoader](crate::interface::plugins::airports::loader::AirportsLoader).
pub struct AirportsDrawer {
    /// Lista de aeropuertos actualmente en memoria.
    airports: Vec<Airport>,

    /// Propiedad de zoom.
    zoom: f32,
}

impl AirportsDrawer {
    /// Crea una nueva instancia del renderizador.
    pub fn new(airports: Vec<Airport>, zoom: f32) -> Self {
        Self { airports, zoom }
    }

    /// Actualiza el valor de la lista de aeropuertos.
    ///
    /// Devuelve esta misma instancia para encadenar funciones.
    pub fn sync_airports(&mut self, real_airports: Vec<Airport>) -> &mut Self {
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

    /// Devuelve las propiedades necesarias para dibujar un círculo según el tipo de aeropuerto.
    fn circle_by_airport_type(airport_type: &AirportType) -> (f32, Rgba, Stroke) {
        match airport_type {
            AirportType::LargeAirport => (
                5.5,
                Rgba::from_srgba_premultiplied(255, 0, 0, 255),
                Stroke::new(1.0, Rgba::from_srgba_premultiplied(70, 60, 50, 100)),
            ),
            AirportType::MediumAirport => (
                5.0,
                Rgba::from_srgba_premultiplied(50, 150, 200, 255),
                Stroke::new(1.0, Rgba::from_srgba_premultiplied(70, 60, 50, 100)),
            ),
            AirportType::SmallAirport => (
                4.5,
                Rgba::from_srgba_premultiplied(100, 255, 100, 255),
                Stroke::new(1.0, Rgba::from_srgba_premultiplied(70, 60, 50, 100)),
            ),
            AirportType::Heliport => (
                4.0,
                Rgba::from_srgba_premultiplied(255, 200, 0, 255),
                Stroke::new(1.0, Rgba::from_srgba_premultiplied(70, 60, 50, 100)),
            ),
            AirportType::SeaplaneBase => (
                4.0,
                Rgba::from_srgba_premultiplied(0, 230, 255, 255),
                Stroke::new(1.0, Rgba::from_srgba_premultiplied(70, 60, 50, 100)),
            ),
            AirportType::BalloonBase => (
                3.5,
                Rgba::from_srgba_premultiplied(255, 0, 100, 255),
                Stroke::new(1.0, Rgba::from_srgba_premultiplied(70, 60, 50, 100)),
            ),
            AirportType::Closed => (
                4.5,
                Rgba::from_srgba_premultiplied(155, 0, 0, 200),
                Stroke::new(1.0, Rgba::from_srgba_premultiplied(70, 60, 50, 100)),
            ),
        }
    }

    /// Devuelve el nivel de zoom aceptable para mostrar el aeropuerto según el tipo.
    fn zoom_is_showable(&self, airport_type: &AirportType) -> bool {
        match airport_type {
            AirportType::LargeAirport => self.zoom >= 0.0,
            AirportType::MediumAirport => self.zoom >= 5.0,
            AirportType::SmallAirport => self.zoom >= 10.0,
            AirportType::Heliport => self.zoom >= 10.0,
            AirportType::SeaplaneBase => self.zoom >= 10.0,
            AirportType::BalloonBase => self.zoom >= 10.0,
            AirportType::Closed => self.zoom >= 10.0,
        }
    }
}

impl Default for AirportsDrawer {
    fn default() -> Self {
        Self::new(
            Vec::new(),
            0.0, // Esto debería cambiarse lo antes posible en subsecuentes iteraciones
        )
    }
}

impl Plugin for &mut AirportsDrawer {
    fn run(&mut self, _response: &Response, painter: Painter, projector: &Projector) {
        for airport in &self.airports {
            if !self.zoom_is_showable(&airport.airport_type) {
                // Sólo mostrar los aeropuertos con el nivel de zoom correcto
                continue;
            }

            let (rad, color, stroke) =
                AirportsDrawer::circle_by_airport_type(&airport.airport_type);
            painter.circle(
                projector.project(airport.position).to_pos2(),
                rad,
                color,
                stroke,
            );
        }
    }
}
