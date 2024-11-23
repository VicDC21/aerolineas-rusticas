//! Módulo para la estructura de la aplicación en sí.

use std::env::var;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local};
use eframe::egui::{CentralPanel, Context};
use eframe::{App, Frame};
use egui_extras::install_image_loaders;
use walkers::sources::OpenStreetMap;
use walkers::{HttpOptions, HttpTiles, Map, MapMemory, Position};

use crate::client::cli::Client;
use crate::interface::{
    data::app_details::AirlinesDetails,
    map::{
        panels::show::{cur_airport_info, extra_airport_info},
        windows::{airports_progress, clock_selector, date_selector, go_to_my_position, zoom},
    },
    plugins::{
        airports::{clicker::ScreenClicker, drawer::AirportsDrawer, loader::AirportsLoader},
        flights::loader::FlightsLoader,
    },
};

/// Latitud de la coordenada de origen de nuestro mapa.
pub const ORIG_LAT: f64 = -34.61760464833609;
/// Longitud de la coordenada de origen de nuestro mapa.
pub const ORIG_LONG: f64 = -58.36909719124974;

/// La app de aerolíneas misma.
pub struct AerolineasApp {
    /// El cliente interno de la aplicación.
    client: Arc<Mutex<Client>>,

    /// Guarda el estado del widget del mapa.
    map_memory: MapMemory,

    /// El proveedor de las _tiles_ del mapa.
    map_tiles: HttpTiles,

    /// El cargador de aeropuertos.
    airports_loader: AirportsLoader,

    /// El renderizador de aeropuertos.
    airports_drawer: AirportsDrawer,

    /// El mirador de clicks.
    screen_clicker: ScreenClicker,

    /// El cargador de vuelos.
    flights_loader: FlightsLoader,

    /// La fecha actual.
    datetime: DateTime<Local>,

    /// Los detalles de lso aeropuertos.
    airlines_details: AirlinesDetails,
}

impl AerolineasApp {
    /// Crea una nueva instancia de la aplicación.
    pub fn new(egui_ctx: Context) -> Self {
        install_image_loaders(&egui_ctx);

        let mut mem = MapMemory::default();
        let _ = mem.set_zoom(8.0); // Queremos un zoom más lejos

        Self {
            client: Arc::new(Mutex::new(Client::default())),
            map_memory: mem,
            map_tiles: Self::open_street_tiles(&egui_ctx),
            airports_loader: AirportsLoader::default(),
            airports_drawer: AirportsDrawer::with_ctx(&egui_ctx),
            screen_clicker: ScreenClicker::default(),
            flights_loader: FlightsLoader::default(),
            datetime: Local::now(),
            airlines_details: AirlinesDetails::default(),
        }
    }

    fn open_street_tiles(ctx: &Context) -> HttpTiles {
        HttpTiles::with_options(
            OpenStreetMap,
            HttpOptions {
                cache: if cfg!(target_os = "android") || var("NO_HTTP_CACHE").is_ok() {
                    None
                } else {
                    Some(".cache".into())
                },
                ..Default::default()
            },
            ctx.clone(),
        )
    }
}

impl App for AerolineasApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let zoom_lvl = self.map_memory.zoom(); // necesariamente antes de crear el mapa
            let map = Map::new(
                Some(&mut self.map_tiles),
                &mut self.map_memory,
                Position::from_lat_lon(ORIG_LAT, ORIG_LONG),
            );

            self.airlines_details
                .set_airports(self.airports_loader.take_airports());
            self.airlines_details
                .set_incoming_flights(self.flights_loader.take_fl_incoming(), false);
            self.airlines_details
                .set_departing_flights(self.flights_loader.take_fl_departing(), false);
            self.airlines_details
                .set_incoming_tracking(self.flights_loader.take_tr_incoming());
            self.airlines_details
                .set_departing_tracking(self.flights_loader.take_tr_departing());

            self.airports_drawer
                .sync_airports(self.airlines_details.get_airports())
                .sync_zoom(zoom_lvl);
            self.screen_clicker
                .sync_airports(self.airlines_details.get_airports())
                .sync_zoom(zoom_lvl);

            let (airps_start, airps_end) = self.airports_loader.get_loading_progress();

            // necesariamente antes de agregar al mapa
            if let Some(cur_airport) = self.screen_clicker.take_cur_airport() {
                // preferiblemente después de asignar las lsitas de vuelos
                self.airlines_details.set_selected_airport(cur_airport);
            }
            if let Some(ex_airport) = self.screen_clicker.take_extra_airport() {
                self.airlines_details.set_extra_airport(ex_airport);
            }

            self.flights_loader
                .sync_date(self.datetime)
                .sync_client(Arc::clone(&self.client))
                .sync_selected_airport(self.airlines_details.get_selected_airport());

            let map = map
                .with_plugin(&mut self.airports_loader)
                .with_plugin(&mut self.airports_drawer)
                .with_plugin(&mut self.screen_clicker)
                .with_plugin(&mut self.flights_loader);

            ui.add(map);

            zoom(ui, &mut self.map_memory);
            go_to_my_position(ui, &mut self.map_memory);

            if let Some(valid_date) = date_selector(ui, &mut self.datetime) {
                self.datetime = valid_date;
            }
            if let Some(valid_time) = clock_selector(ui, &mut self.datetime) {
                self.datetime = valid_time;
            }
            let (show_incoming, show_departing) = cur_airport_info(
                Arc::clone(&self.client),
                ui,
                self.airlines_details.get_ref_selected_airport(),
                self.airlines_details.get_incoming_flights(),
                self.airlines_details.get_show_incoming_flights(),
                self.airlines_details.get_departing_flights(),
                self.airlines_details.get_show_departing_flights(),
            );
            self.airlines_details
                .set_show_incoming_flights(show_incoming);
            self.airlines_details
                .set_show_departing_flights(show_departing);

            extra_airport_info(
                Arc::clone(&self.client),
                ui,
                self.airlines_details.get_ref_selected_airport(),
                self.airlines_details.get_ref_extra_airport(),
                self.datetime.timestamp(),
            );

            if airps_start < airps_end {
                airports_progress(ui, airps_start, airps_end);
            }
        });
    }
}

impl Drop for AerolineasApp {
    fn drop(&mut self) {
        // Por si se cierra la ventana sin dejar que los hilos hijos del cargador
        // se cierren antes.
        self.airports_loader.wait_children();
        self.flights_loader.wait_children();
    }
}
