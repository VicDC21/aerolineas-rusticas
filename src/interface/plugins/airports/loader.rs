//! Módulo de cargador de aeropuertos.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

use eframe::egui::{Painter, Response, Vec2};
use walkers::{Plugin, Position, Projector};

use crate::data::airports::airp::{Airport, AirportsMap};
use crate::protocol::aliases::results::Result;

/// Un hilo destinado a procesos paralelos, tal que no bloquee el flujo sincrónico
/// del hilo principal.
type ChildHandle = JoinHandle<Result<()>>;

/// El tipo de hilo hijo de área.
pub type AreaChild = (Option<ChildHandle>, Sender<PosRect>);

/// Un área de posiciciones geográficas.
pub type PosRect = (Position, Position);

/// Intervalo (en segundos) antes de cargar los aeropuertos de nuevo, como mínimo.
const AIRPORTS_INTERVAL_SECS: u64 = 1;
/// Cantidad máxima _(hardcodeada)_ de aeropuertos.
const MAX_AIRPORTS: usize = 6435;

/// Cargador de aeropuertos.
pub struct AirportsLoader {
    /// Los aeropuertos actualmente en memoria.
    airports: Option<Vec<Airport>>,

    /// La última vez que [crate::interface::plugins::airports::loader::AirportsLoader::airports]
    /// fue modificado.
    last_checked: Instant,

    /// Extremo de canal que recibe actualizaciones a los aeropuertos.
    receiver: Receiver<(Vec<Airport>, usize)>,

    /// Hilo hijo, para cargar aeropuertos en área.
    area_child: AreaChild,

    /// Cantidad de aeropuertos cargados hasta ahora.
    loaded_airps: usize,
}

impl AirportsLoader {
    /// Crea una nueva instancia del cargador de aeropuertos.
    pub fn new(
        airports: Option<Vec<Airport>>,
        receiver: Receiver<(Vec<Airport>, usize)>,
        last_checked: Instant,
        area_child: AreaChild,
    ) -> Self {
        Self {
            airports,
            receiver,
            last_checked,
            area_child,
            loaded_airps: 0,
        }
    }

    /// Genera el hilo de carga de datos.
    fn gen_airp_load(sender: Sender<AirportsMap>) -> JoinHandle<Result<()>> {
        spawn(move || Airport::get_all_channel(sender))
    }

    /// Genera el hilo cargador de área.
    fn gen_area_child(to_parent: Sender<(Vec<Airport>, usize)>) -> AreaChild {
        let (area_sender, area_receiver) = channel::<PosRect>();
        let (load_sender, load_receiver) = channel::<AirportsMap>();
        let mut cache = AirportsMap::new();

        let area_handle = spawn(move || {
            let equator_pos = Position::from_lat_lon(0.0, 0.0);
            let mut loading_handle = Self::gen_airp_load(load_sender.clone());

            loop {
                match area_receiver.recv() {
                    Ok((pos_min, pos_max)) => {
                        if (pos_min == equator_pos) && (pos_max == equator_pos) {
                            break;
                        }

                        // tratar de recargar los aeropuertos
                        if cache.is_empty() && loading_handle.is_finished() {
                            let _ = loading_handle.join(); // por las dudas
                            loading_handle = Self::gen_airp_load(load_sender.clone());
                        }

                        for airp in load_receiver.try_iter() {
                            cache.extend(airp);
                        }

                        let airports = Airport::by_area_cache(
                            (pos_min.lat(), pos_min.lon(), pos_max.lat(), pos_max.lon()),
                            &cache,
                        );

                        if let Err(err) = to_parent.send((airports, cache.len())) {
                            println!(
                                "Error al mandar a hilo principal los aeropuertos:\n\n{}",
                                err
                            );
                        }
                    }
                    Err(err) => println!(
                        "Ocurrió un error esperando mensajes del hilo principal:\n\n{}",
                        err
                    ),
                }
            }

            let _ = loading_handle.join();
            Ok(())
        });

        (Some(area_handle), area_sender.clone())
    }

    /// **Consume** la lista de aeropuertos actualmente en memoria para devolverla, y en su lugar
    /// deja [None].
    ///
    /// En caso de haber sido consumida en una iteración anterior, devuelve un vector vacío.
    pub fn take_airports(&mut self) -> Vec<Airport> {
        self.airports.take().unwrap_or_default()
    }

    /// Consigue la cantidad de aeropuertos cargados hasta ahora, comparados a los totales.
    pub fn get_loading_progress(&self) -> (usize, usize) {
        (self.loaded_airps, MAX_AIRPORTS)
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
}

impl Default for AirportsLoader {
    fn default() -> Self {
        let (main_sender, main_receiver) = channel::<(Vec<Airport>, usize)>();

        Self::new(
            Some(Vec::new()),
            main_receiver,
            Instant::now(),
            Self::gen_area_child(main_sender.clone()),
        )
    }
}

impl Plugin for &mut AirportsLoader {
    fn run(&mut self, response: &Response, _painter: Painter, projector: &Projector) {
        if self.elapsed_at_least(&Duration::from_secs(AIRPORTS_INTERVAL_SECS)) {
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
        }
        // y luego le pedimos si terminó (puede no ser en este frame)
        if let Ok((new_airports, new_len)) = self.receiver.try_recv() {
            self.loaded_airps = new_len;
            if !new_airports.is_empty() {
                self.airports = Some(new_airports);
            }
        }
    }
}

impl Drop for AirportsLoader {
    fn drop(&mut self) {
        self.wait_children();
    }
}
