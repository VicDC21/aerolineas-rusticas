//! Módulo para la estructura de la aplicación en sí.

use {
    crate::{
        data::{app_details::AirlinesDetails, widget_details::WidgetDetails},
        panels::show::{cur_airport_info, extra_airport_info},
        plugins::{
            airports::{clicker::ScreenClicker, drawer::AirportsDrawer, loader::AirportsLoader},
            flights::{loader::FlightsLoader, updater::FlightsUpdater},
        },
        windows::{
            airp::{airports_progress, clock_selector, date_selector, login_window},
            util::{go_to_my_position, zoom},
        },
    },
    chrono::{DateTime, Local},
    client::{cli::Client, conn_holder::ConnectionHolder},
    data::{flights::types::FlightType, tracking::live_flight_data::LiveFlightData},
    eframe::{
        egui::{CentralPanel, Context},
        App, Frame,
    },
    egui_extras::install_image_loaders,
    protocol::aliases::{results::Result, types::Double},
    std::env::var,
    walkers::{sources::OpenStreetMap, HttpOptions, HttpTiles, Map, MapMemory, Position},
};

/// Latitud de la coordenada de origen de nuestro mapa.
pub const ORIG_LAT: Double = -34.61760464833609;
/// Longitud de la coordenada de origen de nuestro mapa.
pub const ORIG_LONG: Double = -58.36909719124974;

/// La app de aerolíneas misma.
pub struct AerolineasApp {
    /// La información de la conexión actual.
    con_info: ConnectionHolder,

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

    /// El actualizador de vuelos.
    flights_updater: FlightsUpdater,

    /// La fecha actual.
    datetime: DateTime<Local>,

    /// Los detalles de lso aeropuertos.
    airlines_details: AirlinesDetails,

    /// Detalles de widgets auxiliares.
    widget_details: WidgetDetails,
}

impl AerolineasApp {
    /// Crea una nueva instancia de la aplicación.
    pub fn new(egui_ctx: Context) -> Result<Self> {
        install_image_loaders(&egui_ctx);

        let mut mem = MapMemory::default();
        let _ = mem.set_zoom(8.0); // Queremos un zoom más lejos

        let con_info = match ConnectionHolder::with_cli(Client::default(), "QUORUM") {
            Ok(con) => con,
            Err(e) => {
                return Err(e);
            }
        };

        Ok(Self {
            con_info,
            map_memory: mem,
            map_tiles: Self::open_street_tiles(&egui_ctx),
            airports_loader: AirportsLoader::default(),
            airports_drawer: AirportsDrawer::with_ctx(&egui_ctx),
            screen_clicker: ScreenClicker::default(),
            flights_loader: FlightsLoader::default(),
            flights_updater: FlightsUpdater::with_ctx(egui_ctx.clone())?,
            datetime: Local::now(),
            airlines_details: AirlinesDetails::default(),
            widget_details: WidgetDetails::default(),
        })
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
            let login_info = self.widget_details.login_info.to_owned();

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
            self.flights_updater
                .sync_airport(self.airlines_details.get_selected_airport())
                .sync_incoming_tracking(self.airlines_details.get_incoming_tracking())
                .sync_departing_tracking(self.airlines_details.get_departing_tracking());

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
                .sync_login_info(&login_info, &self.widget_details.has_logged_in)
                .sync_selected_airport(self.airlines_details.get_selected_airport());

            let map = map
                .with_plugin(&mut self.airports_loader)
                .with_plugin(&mut self.airports_drawer)
                .with_plugin(&mut self.screen_clicker)
                .with_plugin(&mut self.flights_loader)
                .with_plugin(&mut self.flights_updater);

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
                &mut self.con_info,
                ui,
                self.airlines_details.get_ref_selected_airport(),
                (
                    self.airlines_details.get_incoming_flights(),
                    self.airlines_details.get_show_incoming_flights(),
                ),
                (
                    self.airlines_details.get_departing_flights(),
                    self.airlines_details.get_show_departing_flights(),
                ),
                &mut self.widget_details,
            );
            self.airlines_details
                .set_show_incoming_flights(show_incoming);
            self.airlines_details
                .set_show_departing_flights(show_departing);

            extra_airport_info(
                &mut self.con_info,
                ui,
                self.airlines_details.get_ref_selected_airport(),
                self.airlines_details.get_ref_extra_airport(),
                self.datetime.timestamp(),
            );

            if airps_start < airps_end {
                airports_progress(ui, airps_start, airps_end);
            }

            login_window(ui, &mut self.con_info, &mut self.widget_details);

            if let Some(editor) = &mut self.widget_details.flight_editor {
                let mut live_data = None;
                if let Some(flight) = &editor.held_flight {
                    let live_data_map = match flight.flight_type {
                        FlightType::Incoming => self.airlines_details.get_incoming_tracking(),
                        FlightType::Departing => self.airlines_details.get_departing_tracking(),
                    };
                    live_data = match live_data_map.get(&flight.id) {
                        Some(candidates) => LiveFlightData::most_recent(candidates).cloned(),
                        None => None,
                    };
                }
                if !editor.show(&mut self.con_info, ui, self.datetime, live_data) {
                    self.widget_details.flight_editor = None;
                }
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
