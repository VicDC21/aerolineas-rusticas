//! Módulo para los proveedores de tiles.

use std::collections::HashMap;

use eframe::egui::Context;
use walkers::{HttpOptions, HttpTiles, Tiles};

use crate::interface::map::local_tiles::LocalTiles;

/// _Hashmap_ de proveedores de tiles para un mapa geográfico.
pub type ProvidersMap = HashMap<Provider, Box<dyn Tiles + Send>>;

/// Un servicio del que descargar las tiles de un mapa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    /// Usa [OpenstreetMap](walkers::sources::OpenStreetMap).
    OpenStreetMap,

    /// Usa [Geoportal](walkers::sources::Geoportal).
    Geoportal,

    /// Usa [Mapbox](walkers::sources::Mapbox) con [estilo](walkers::sources::MapboxStyle)
    /// de "streets".
    MapboxStreets,

    /// Usa [Mapbox](walkers::sources::Mapbox) con [estilo](walkers::sources::MapboxStyle)
    /// de "satellite".
    MapboxSatellite,

    /// Usa un proveedor local.
    LocalTiles,
}

impl Provider {
    /// Opciones para HTTP en modo pro defecto.
    fn http_options() -> HttpOptions {
        HttpOptions {
            cache: if cfg!(target_os = "android") || std::env::var("NO_HTTP_CACHE").is_ok() {
                None
            } else {
                Some(".cache".into())
            },
            ..Default::default()
        }
    }

    /// Devuelve un [_hashmap_](ProvidersMap) de proveedores tal que quede listo para usar.
    pub fn providers(egui_ctx: Context) -> ProvidersMap {
        let mut providers = ProvidersMap::new();

        providers.insert(
            Provider::OpenStreetMap,
            Box::new(HttpTiles::with_options(
                walkers::sources::OpenStreetMap,
                Self::http_options(),
                egui_ctx.to_owned(),
            )),
        );

        providers.insert(
            Provider::Geoportal,
            Box::new(HttpTiles::with_options(
                walkers::sources::Geoportal,
                Self::http_options(),
                egui_ctx.to_owned(),
            )),
        );

        providers.insert(
            Provider::LocalTiles,
            Box::new(LocalTiles::new(egui_ctx.to_owned())),
        );

        // Cargar proveedores de Mapbox sólo si se tiene uyn token (es un servicio premium).
        let mapbox_access_token = std::option_env!("MAPBOX_ACCESS_TOKEN");

        if let Some(token) = mapbox_access_token {
            providers.insert(
                Provider::MapboxStreets,
                Box::new(HttpTiles::with_options(
                    walkers::sources::Mapbox {
                        style: walkers::sources::MapboxStyle::Streets,
                        access_token: token.to_string(),
                        high_resolution: false,
                    },
                    Self::http_options(),
                    egui_ctx.to_owned(),
                )),
            );
            providers.insert(
                Provider::MapboxSatellite,
                Box::new(HttpTiles::with_options(
                    walkers::sources::Mapbox {
                        style: walkers::sources::MapboxStyle::Satellite,
                        access_token: token.to_string(),
                        high_resolution: true,
                    },
                    Self::http_options(),
                    egui_ctx.to_owned(),
                )),
            );
        }

        providers
    }
}
