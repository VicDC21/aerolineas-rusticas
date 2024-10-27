//! Módulo para la estructura de la aplicación en sí.

use std::sync::Arc;

use eframe::egui::{
    CentralPanel, Color32, Context, Frame as EguiFrame, Margin, RichText, SidePanel,
};
use eframe::{App, Frame};
use egui_extras::install_image_loaders;
use walkers::{Map, MapMemory, Position};

use crate::data::airports::Airport;
use crate::interface::map::providers::{Provider, ProvidersMap};
use crate::interface::map::windows::{controls, go_to_my_position, zoom};
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

            if let Some(cur_airport) = self.screen_clicker.take_cur_airport() {
                // necesariamente antes de agregar al mapa
                self.selected_airport = cur_airport;
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

        let panel_frame = EguiFrame {
            fill: Color32::from_rgba_unmultiplied(66, 66, 66, 200),
            inner_margin: Margin::ZERO,
            ..Default::default()
        };
        let info_panel = SidePanel::left("airport_info")
            .resizable(false)
            .exact_width(ctx.screen_rect().width() / 3.0)
            .frame(panel_frame);
        info_panel.show_animated(ctx, self.selected_airport.is_some(), |ui| {
            if let Some(airport) = &self.selected_airport {
                let text_color = Color32::from_rgba_unmultiplied(200, 200, 200, 255);
                ui.label(
                    RichText::new(format!("\t{}", &airport.name))
                        .color(text_color)
                        .heading(),
                );
                ui.separator();

                ui.label(
                    RichText::new(format!("\n\n\tIdent:\t{}", &airport.ident)).color(text_color),
                );
                ui.label(
                    RichText::new(format!("\tType:\t{}", &airport.airport_type)).color(text_color),
                );

                ui.label(
                    RichText::new(format!(
                        "\n\tPosition:\t({}, {})",
                        &airport.position.lat(),
                        &airport.position.lon()
                    ))
                    .color(text_color),
                );
                ui.label(
                    RichText::new(format!(
                        "\tElevation (ft):\t{}",
                        &airport.elevation_ft.unwrap_or(-999)
                    ))
                    .color(text_color),
                );

                ui.label(
                    RichText::new(format!("\tContinent:\t{}", &airport.continent))
                        .color(text_color),
                );

                ui.label(
                    RichText::new(format!("\tCountry (ISO):\t{}", &airport.iso_country))
                        .color(text_color),
                );
            }
        });
    }
}

impl Drop for AerolineasApp {
    fn drop(&mut self) {
        // Por si se cierra la ventana sin dejar que los hilos hijos del cargador
        // se cierren antes.
        self.airports_loader.wait_children();
    }
}
