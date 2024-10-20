//! Módulo de cargador de aeropuertos.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

use eframe::egui::{Painter, Response, Rgba, Stroke, Vec2};
use walkers::{Plugin, Position, Projector};

use crate::data::airport_types::AirportType;
use crate::data::airports::Airport;
use crate::protocol::aliases::results::Result;

/// Un hilo destinado a procesos paralelos, tal que no bloquee el flujo sincrónico
/// del hilo principal.
pub type ChildHandle = JoinHandle<Result<()>>;

/// Un área de posiciciones geográficas.
pub type PosRect = (Position, Position);

/// Intervalo (en segundos) antes de cargar los aeropuertos de nuevo, como mínimo.
const AIRPORTS_INTERVAL_SECS: u64 = 5;

/// Cargador de aeropuertos.
pub struct AirportsLoader {
    /// Los aeropuertos actualmente en memoria.
    airports: Vec<Airport>,

    /// La última vez que [crate::interface::plugins::airports::loader::AirportsLoader::airports]
    /// fue modificado.
    last_checked: Instant,

    /// Extremo de canal que recibe actualizaciones a los aeropuertos.
    receiver: Receiver<Vec<Airport>>,

    /// Hilo hijo, para cargar aeropuertos en área.
    area_child: (Option<ChildHandle>, Sender<PosRect>),
}

impl AirportsLoader {
    /// Crea una nueva instancia del cargador de aeropuertos.
    pub fn new(
        airports: Vec<Airport>,
        receiver: Receiver<Vec<Airport>>,
        last_checked: Instant,
        area_child: (Option<ChildHandle>, Sender<PosRect>),
    ) -> Self {
        Self {
            airports,
            receiver,
            last_checked,
            area_child,
        }
    }

    /// Resetea el chequeo al [Instant] actual.
    pub fn reset_instant(&mut self) {
        self.last_checked = Instant::now();
    }

    /// Verifica si ha pasado un mínimo de tiempo dado desde la última vez
    /// que se editaron los puertos.
    pub fn elapsed_at_least(&self, duration: &Duration) -> bool {
        &self.last_checked.elapsed() >= duration
    }

    /// Apaga y espera a todos los hilos hijos.
    pub fn wait_children(&mut self) {
        let (area, area_sender) = &mut self.area_child;
        if let Some(hanging) = area.take() {
            // esto es el mensaje secreto para que pare
            if area_sender
                .send((
                    Position::from_lat_lon(0.0, 0.0),
                    Position::from_lat_lon(0.0, 0.0),
                ))
                .is_err()
            {
                println!("Error mandando un mensaje para parar hilo de área.")
            }
            if hanging.join().is_err() {
                println!("Error esperando a que un hilo hijo termine.")
            }
        }
    }

    /// Devuelve las propiedades necesarias para dibujar un círculo según el tipo de aeropuerto.
    pub fn circle_by_airport_type(airport: &Airport) -> (f32, Rgba, Stroke) {
        match airport.airport_type {
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
                Rgba::from_srgba_premultiplied(255, 0, 0, 255),
                Stroke::new(1.0, Rgba::from_srgba_premultiplied(70, 60, 50, 100)),
            ),
        }
    }
}

impl Default for AirportsLoader {
    fn default() -> Self {
        let (main_sender, main_receiver) = channel::<Vec<Airport>>();

        // el proceso de cargador de área
        let (area_sender, area_receiver) = channel::<PosRect>();
        let area_handle = spawn(move || {
            let to_parent = main_sender.clone();
            let equator_pos = Position::from_lat_lon(0.0, 0.0);

            loop {
                if let Ok((pos_min, pos_max)) = area_receiver.recv() {
                    if (pos_min == equator_pos) && (pos_max == equator_pos) {
                        break;
                    }

                    if let Err(err) = to_parent.send(Airport::by_area((&pos_min, &pos_max))?) {
                        println!(
                            "Error al mandar a hilo principal los aeropuertos:\n\n{}",
                            err
                        );
                    }
                }
            }

            Ok(())
        });

        Self::new(
            Vec::new(),
            main_receiver,
            Instant::now(),
            (Some(area_handle), area_sender.clone()),
        )
    }
}

impl Plugin for &mut AirportsLoader {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        if response.dragged() && self.elapsed_at_least(&Duration::from_secs(AIRPORTS_INTERVAL_SECS))
        {
            self.reset_instant();
            // Necesitamos correrlo para arriba a la izquierda porque el proyector calcula
            // cosas desde el centro de la pantalla.
            let offset_area = response.rect.translate(Vec2::new(
                -(response.rect.max.x / 2.0),
                -(response.rect.max.y / 2.0),
            ));
            let geo_min = projector.unproject(offset_area.min.to_vec2());
            let geo_max = projector.unproject(offset_area.max.to_vec2());
            let area = (
                Position::from_lat_lon(
                    geo_min.lat().min(geo_max.lat()),
                    geo_min.lon().min(geo_max.lon()),
                ),
                Position::from_lat_lon(
                    geo_min.lat().max(geo_max.lat()),
                    geo_min.lon().max(geo_max.lon()),
                ),
            );

            // primero le pedimos al cargador que vaya procesando el área
            let (_, area_sender) = &mut self.area_child;
            if let Err(err) = area_sender.send(area) {
                println!("Error al enviar área al cargador:\n\n{}", err);
            }

            // y luego le pedimos si terminó (puede no ser en este frame)
            if let Ok(new_airports) = self.receiver.try_recv() {
                if !new_airports.is_empty() {
                    self.airports = new_airports;
                }
            }
        }

        for airport in &self.airports {
            let (rad, color, stroke) = AirportsLoader::circle_by_airport_type(airport);
            painter.circle(
                projector.project(airport.position).to_pos2(),
                rad,
                color,
                stroke,
            );
        }
    }
}

impl Drop for AirportsLoader {
    fn drop(&mut self) {
        self.wait_children();
    }
}
