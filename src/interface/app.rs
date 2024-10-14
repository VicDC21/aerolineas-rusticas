//! Módulo para la estructura de la aplicación en sí.

use eframe::egui::{CentralPanel, Context};
use eframe::{App, Frame};
use walkers::{sources::OpenStreetMap, Map, MapMemory, Position, Tiles};

/// La app de aerolíneas misma.
pub struct AerolineasApp {
    /// Descarga tiles del mapa de un proveedor y las guarda en un chache.
    map_tiles: Tiles,

    /// Guarda el estado del widget del mapa.
    map_memory: MapMemory,
}

impl AerolineasApp {
    /// Crea una nueva instancia de la aplicación.
    pub fn new(egui_ctx: Context) -> Self {
        Self {
            map_tiles: Tiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(),
        }
    }
}

impl App for AerolineasApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.add(Map::new(
                Some(&mut self.map_tiles),
                &mut self.map_memory,
                Position::from_lat_lon(-34.61760464833609, -58.36909719124974),
            ))
        });
    }
}
