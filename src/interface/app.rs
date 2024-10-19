//! Módulo para la estructura de la aplicación en sí.

use eframe::egui::{CentralPanel, Context};
use eframe::{App, Frame};
use walkers::{Map, MapMemory, Position};

use crate::interface::map::providers::{Provider, ProvidersMap};
use crate::interface::map::windows::{acknowledge, controls, go_to_my_position, zoom};

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
}

impl AerolineasApp {
    /// Crea una nueva instancia de la aplicación.
    pub fn new(egui_ctx: Context) -> Self {
        let mut mem = MapMemory::default();
        let _ = mem.set_zoom(7.0); // Queremos un zoom más lejos
        Self {
            map_memory: mem,
            map_providers: Provider::providers(egui_ctx.to_owned()),
            selected_provider: Provider::OpenStreetMap,
        }
    }
}

impl App for AerolineasApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let scroll_delta = ctx.input(|input_state| input_state.raw_scroll_delta);
            if scroll_delta.y > 0.0 {
                // scrollea para arriba
                let _ = self.map_memory.zoom_in();
            } else if scroll_delta.y < 0.0 {
                let _ = self.map_memory.zoom_out();
            }

            let tiles = self
                .map_providers
                .get_mut(&self.selected_provider)
                .unwrap()
                .as_mut();
            let attribution = tiles.attribution();

            let map = Map::new(
                Some(tiles),
                &mut self.map_memory,
                Position::from_lat_lon(ORIG_LAT, ORIG_LONG),
            );

            ui.add(map);

            zoom(ui, &mut self.map_memory);
            go_to_my_position(ui, &mut self.map_memory);
            controls(
                ui,
                &mut self.selected_provider,
                &mut self.map_providers.keys(),
                // &mut self.images_plugin_data,
            );
            acknowledge(ui, attribution);
        });
    }
}
