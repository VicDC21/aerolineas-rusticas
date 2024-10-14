//! Módulo para un **ejemplo** de proveedor local de tiles.

use eframe::egui::{ColorImage, Context};
use walkers::{sources::Attribution, Texture, TileId, TilesManager};

/// Para tener más control del manejo de tiles, se puede implementar
/// un proveedor local para el mapa, como evidencia este ejemplo.
pub struct LocalTiles {
    egui_ctx: Context,
}

impl LocalTiles {
    /// Crea una nueva instancia de proveedor local.
    pub fn new(egui_ctx: Context) -> Self {
        Self { egui_ctx }
    }
}

impl TilesManager for LocalTiles {
    fn at(&mut self, _tile_id: TileId) -> Option<Texture> {
        let image = ColorImage::example();

        Some(Texture::from_color_image(image, &self.egui_ctx))
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "Local rendering example",
            url: "https://github.com/podusowski/walkers",
            logo_light: None,
            logo_dark: None,
        }
    }

    fn tile_size(&self) -> u32 {
        256
    }
}
