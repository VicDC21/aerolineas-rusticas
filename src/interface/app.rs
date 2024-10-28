//! Módulo para la estructura de la aplicación en sí.

use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use eframe::egui::{CentralPanel, Context};
use eframe::{App, Frame};
use egui_extras::install_image_loaders;
use walkers::{Map, MapMemory, Position};

use crate::data::airports::Airport;
use crate::interface::map::{
    panels::{cur_airport_info, extra_airport_info},
    providers::{Provider, ProvidersMap},
    windows::{controls, date_selector, go_to_my_position, zoom},
};
use crate::interface::plugins::airports::{
    clicker::ScreenClicker, drawer::AirportsDrawer, loader::AirportsLoader,
};

/// Latitud de la coordenada de origen de nuestro mapa.
pub const ORIG_LAT: f64 = -34.61760464833609;
/// Longitud de la coordenada de origen de nuestro mapa.
pub const ORIG_LONG: f64 = -58.36909719124974;

/// La app de aerolíneas misma.
pub struct AerolineasApp {
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

    /// El puerto seleccionado actualmente.
    selected_airport: Option<Airport>,

    /// Un aeropuerto extra, para acciones especiales.
    extra_airport: Option<Airport>,

    /// La fecha actual.
    date: NaiveDate,
}

impl AerolineasApp {
    /// Crea una nueva instancia de la aplicación.
    pub fn new(egui_ctx: Context) -> Self {
        install_image_loaders(&egui_ctx);

        let mut mem = MapMemory::default();
        let _ = mem.set_zoom(8.0); // Queremos un zoom más lejos

        Self {
            map_memory: mem,
            map_providers: Provider::providers(egui_ctx.to_owned()),
            selected_provider: Provider::OpenStreetMap,
            airports_loader: AirportsLoader::default(),
            airports_drawer: AirportsDrawer::with_ctx(&egui_ctx),
            screen_clicker: ScreenClicker::default(),
            selected_airport: None,
            extra_airport: None,
            date: Utc::now().date_naive(),
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
            let cur_airports = Arc::new(self.airports_loader.take_airports());
            self.airports_drawer
                .sync_airports(Arc::clone(&cur_airports))
                .sync_zoom(zoom_lvl);
            self.screen_clicker
                .sync_airports(Arc::clone(&cur_airports))
                .sync_zoom(zoom_lvl);

            // necesariamente antes de agregar al mapa
            if let Some(cur_airport) = self.screen_clicker.take_cur_airport() {
                self.selected_airport = cur_airport;
            }
            if let Some(ex_airport) = self.screen_clicker.take_extra_airport() {
                self.extra_airport = ex_airport;
            }

            match &self.selected_airport {
                Some(cur_airport) => {
                    if let Some(ex_airport) = &self.extra_airport {
                        if cur_airport == ex_airport {
                            // Si los aeropuertos son iguales, anular la selección.
                            self.extra_airport = None;
                        }
                    }
                }
                None => {
                    // Y si no hay aeropuerto seleccionado también
                    self.extra_airport = None;
                }
            }

            let map = map
                .with_plugin(&mut self.airports_loader)
                .with_plugin(&mut self.airports_drawer)
                .with_plugin(&mut self.screen_clicker);

            ui.add(map);

            zoom(ui, &mut self.map_memory);
            go_to_my_position(ui, &mut self.map_memory);
            controls(
                ui,
                &mut self.selected_provider,
                &mut self.map_providers.keys(),
            );
        });

        date_selector(ctx, &mut self.date);
        cur_airport_info(ctx, &self.selected_airport);
        extra_airport_info(ctx, &self.selected_airport, &self.extra_airport);
    }
}

impl Drop for AerolineasApp {
    fn drop(&mut self) {
        // Por si se cierra la ventana sin dejar que los hilos hijos del cargador
        // se cierren antes.
        self.airports_loader.wait_children();
    }
}
