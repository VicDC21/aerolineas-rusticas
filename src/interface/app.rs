//! Módulo para la estructura de la aplicación en sí.

use chrono::{DateTime, Local};
use eframe::egui::{CentralPanel, Context};
use eframe::{App, Frame};
use egui_extras::install_image_loaders;
use walkers::{Map, MapMemory, Position};

use crate::client::cli::Client;
use crate::interface::{
    data::app_details::AirlinesDetails,
    map::{
        panels::{cur_airport_info, extra_airport_info},
        providers::{Provider, ProvidersMap},
        windows::{clock_selector, controls, date_selector, go_to_my_position, zoom},
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
    client: Client,

    /// Guarda el estado del widget del mapa.
    map_memory: MapMemory,

    /// Lista de potenciales proveedores de tiles.
    map_providers: ProvidersMap,

    /// El proveedor actualmente en uso.
    selected_provider: Provider,

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
            client: Client::default(),
            map_memory: mem,
            map_providers: Provider::providers(egui_ctx.to_owned()),
            selected_provider: Provider::OpenStreetMap,
            airports_loader: AirportsLoader::default(),
            airports_drawer: AirportsDrawer::with_ctx(&egui_ctx),
            screen_clicker: ScreenClicker::default(),
            flights_loader: FlightsLoader::default(),
            datetime: Local::now(),
            airlines_details: AirlinesDetails::default(),
        }
    }
}

impl App for AerolineasApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let tiles = self
                .map_providers
                .get_mut(&self.selected_provider)
                .unwrap()
                .as_mut();

            let zoom_lvl = self.map_memory.zoom(); // necesariamente antes de crear el mapa
            let map = Map::new(
                Some(tiles),
                &mut self.map_memory,
                Position::from_lat_lon(ORIG_LAT, ORIG_LONG),
            );

            // Añadimos los plugins.
            self.airlines_details
                .set_airports(self.airports_loader.take_airports());
            self.airlines_details
                .set_incoming_flights(self.flights_loader.take_incoming());
            self.airlines_details
                .set_departing_flights(self.flights_loader.take_departing());

            self.airports_drawer
                .sync_airports(self.airlines_details.get_airports())
                .sync_zoom(zoom_lvl);
            self.screen_clicker
                .sync_airports(self.airlines_details.get_airports())
                .sync_zoom(zoom_lvl);
            self.flights_loader
                .sync_date(self.datetime)
                .sync_client(self.client.clone());

            // necesariamente antes de agregar al mapa
            if let Some(cur_airport) = self.screen_clicker.take_cur_airport() {
                self.airlines_details.set_selected_airport(cur_airport);
            }
            if let Some(ex_airport) = self.screen_clicker.take_extra_airport() {
                self.airlines_details.set_extra_airport(ex_airport);
            }

            let map = map
                .with_plugin(&mut self.airports_loader)
                .with_plugin(&mut self.airports_drawer)
                .with_plugin(&mut self.screen_clicker)
                .with_plugin(&mut self.flights_loader);

            ui.add(map);

            zoom(ui, &mut self.map_memory);
            go_to_my_position(ui, &mut self.map_memory);
            controls(
                ui,
                &mut self.selected_provider,
                &mut self.map_providers.keys(),
            );
        });

        if let Some(valid_date) = date_selector(ctx, &mut self.datetime) {
            self.datetime = valid_date;
        }
        if let Some(valid_time) = clock_selector(ctx, &mut self.datetime) {
            self.datetime = valid_time;
        }
        self.airlines_details
            .set_show_incoming_flights(cur_airport_info(
                ctx,
                self.airlines_details.get_selected_airport(),
                self.airlines_details.get_incoming_flights(),
                self.airlines_details.get_show_incoming_flights(),
            ));
        extra_airport_info(
            ctx,
            self.airlines_details.get_selected_airport(),
            self.airlines_details.get_extra_airport(),
            self.client.clone(),
            self.datetime.timestamp(),
        );
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
