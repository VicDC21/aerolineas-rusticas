//! Módulo que dibuja aeropuertos.

use std::collections::HashMap;
use std::sync::Arc;

use eframe::egui::{Context, Painter, Pos2, Response, Rgba, Stroke};
use walkers::{extras::Image, Plugin, Projector, Texture};

use crate::data::{
    airports::{airp::Airport, types::AirportType},
    utils::distances::distance_euclidean_pos2,
};
use crate::interface::plugins::utils::{load_egui_img, zoom_is_showable};

/// Mapa de íconos de tipos de aeropuertos.
pub type IconsMap = HashMap<AirportType, Option<Texture>>;

/// Ruta de ícono de aeropuerto grande.
const LARGE_AIRPORT_ICON: &str = "media/img/airports/large_plane.png";
/// Ruta de ícono de aeropuerto mediano.
const MEDIUM_AIRPORT_ICON: &str = "media/img/airports/medium_plane.png";
/// Ruta de ícono de aeropuerto chico.
const SMALL_AIRPORT_ICON: &str = "media/img/airports/small_plane.png";
/// Ruta de ícono de helipuerto.
const HELIPORT_ICON: &str = "media/img/airports/heliport.png";
/// Ruta de ícono de aeropuerto de hidroaviones.
const SEAPLANE_ICON: &str = "media/img/airports/seaplane.png";
/// Ruta de ícono de globo de aire caliente.
const BALLOON_ICON: &str = "media/img/airports/hot_air_balloon.png";
/// Ruta de ícono de aeropuerto cerrado.
const CLOSED_ICON: &str = "media/img/airports/closed.png";

/// Reducción de dimensiones mínimas para que entre en la pantalla.
const BASE_DIM_RED: f32 = 0.02;

// Distancia mínima para un cursor sin clickear.
const MIN_HOVER_DIST: f64 = 13.0;
// Aumento de dimensiones al apoyar el cursor sobre un objeto.
const HOVER_INCR: f32 = 0.02;

/// Este plugin se encarga de dibujar la información en pantalla de los
/// aeropuertos cargados por [AirportsLoader](crate::interface::plugins::airports::loader::AirportsLoader).
pub struct AirportsDrawer {
    /// Lista de aeropuertos actualmente en memoria.
    airports: Arc<Vec<Airport>>,

    // Íconos de aeropuertos.
    icons: IconsMap,

    /// Propiedad de zoom.
    zoom: f32,
}

impl AirportsDrawer {
    /// Crea una nueva instancia del renderizador.
    pub fn new(airports: Arc<Vec<Airport>>, zoom: f32, ctx: &Context) -> Self {
        Self {
            airports,
            icons: Self::load_icons(ctx),
            zoom,
        }
    }

    /// Crea una instancia con un contexto dado.
    pub fn with_ctx(ctx: &Context) -> Self {
        Self::new(
            Arc::new(Vec::new()),
            0.0, // Esto debería cambiarse lo antes posible en subsecuentes iteraciones
            ctx,
        )
    }

    /// Cargta los íconos de aeropuertos en memoria.
    fn load_icons(ctx: &Context) -> IconsMap {
        let mut icons = IconsMap::new();
        let types = [
            AirportType::LargeAirport,
            AirportType::MediumAirport,
            AirportType::SmallAirport,
            AirportType::Heliport,
            AirportType::SeaplaneBase,
            AirportType::BalloonBase,
            AirportType::Closed,
        ];

        for airport_type in types {
            let path = Self::img_path_by_type(&airport_type);
            let texture = match load_egui_img(path) {
                Err(_) => None,
                Ok(color_img) => Some(Texture::from_color_image(color_img, ctx)),
            };
            icons.insert(airport_type, texture);
        }

        icons
    }

    /// Actualiza el valor de la lista de aeropuertos.
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

    fn img_path_by_type(airport_type: &AirportType) -> &str {
        match airport_type {
            AirportType::LargeAirport => LARGE_AIRPORT_ICON,
            AirportType::MediumAirport => MEDIUM_AIRPORT_ICON,
            AirportType::SmallAirport => SMALL_AIRPORT_ICON,
            AirportType::Heliport => HELIPORT_ICON,
            AirportType::SeaplaneBase => SEAPLANE_ICON,
            AirportType::BalloonBase => BALLOON_ICON,
            AirportType::Closed => CLOSED_ICON,
        }
    }

    /// Devuelve las redimensiones de una imagen para que se muestre bien entre
    /// los límites de la pantalla actual.
    pub fn scale_img_by_type(airport_type: &AirportType) -> f32 {
        let extra = match airport_type {
            AirportType::LargeAirport => 0.01,
            AirportType::MediumAirport => 0.007,
            AirportType::SmallAirport => 0.005,
            AirportType::Heliport => 0.005,
            AirportType::SeaplaneBase => 0.005,
            AirportType::BalloonBase => 0.005,
            AirportType::Closed => 0.007,
        };
        BASE_DIM_RED + extra
    }

    /// Redimensiona una imagen si el cursor está cerca.
    pub fn scale_img_by_pos(img: &mut Image, base: f32, cur_pos: &Pos2, airport_pos: &Pos2) {
        if distance_euclidean_pos2(cur_pos, airport_pos) <= MIN_HOVER_DIST {
            img.scale(base + HOVER_INCR, base + HOVER_INCR);
        }
    }

    fn draw_circle(airport: &Airport, painter: &Painter, projector: &Projector) {
        let (rad, color, stroke) = AirportsDrawer::circle_by_airport_type(&airport.airport_type);
        painter.circle(
            projector.project(airport.position).to_pos2(),
            rad,
            color,
            stroke,
        );
    }
}

impl Plugin for &mut AirportsDrawer {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        for airport in self.airports.iter() {
            if !zoom_is_showable(&airport.airport_type, self.zoom) {
                // Sólo mostrar los aeropuertos con el nivel de zoom correcto
                continue;
            }

            let icon = self.icons.get(&airport.airport_type);
            if let Some(Some(texture)) = icon {
                let mut img = Image::new(texture.clone(), airport.position);
                let extra = AirportsDrawer::scale_img_by_type(&airport.airport_type);
                img.scale(extra, extra);
                if let Some(hover_pos) = response.hover_pos() {
                    AirportsDrawer::scale_img_by_pos(
                        &mut img,
                        extra,
                        &hover_pos,
                        &projector.project(airport.position).to_pos2(),
                    );
                }
                img.draw(response, painter.clone(), projector);
            } else {
                AirportsDrawer::draw_circle(airport, &painter, projector);
            }
        }
    }
}
