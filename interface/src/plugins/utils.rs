//! Módulo para funciones auxiliares de plugins.

use {
    data::airports::{airp::Airport, types::AirportType},
    eframe::egui::{ColorImage, Context},
    image::{ImageError, ImageReader},
    protocol::{
        aliases::{results::Result, types::Float},
        errors::error::Error,
    },
    std::result::Result as StdResult,
    walkers::{
        Position,
        {extras::Image, Texture},
    },
};

/// Intenta cargar una [imagen](eframe::egui::ColorImage) de EGUI.
pub fn load_egui_img(path: &str) -> StdResult<ColorImage, ImageError> {
    let img = ImageReader::open(path)?.decode()?;

    let size = [img.width() as _, img.height() as _];
    let image_buffer = img.to_rgba8();
    let pixels = image_buffer.as_flat_samples();

    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

/// Intenta cargar una [imagen](walkers::extras::Image).
pub fn load_airport_image(path: &str, airport: &Airport, context: &Context) -> Result<Image> {
    match load_egui_img(path) {
        Err(err) => Err(Error::ServerError(format!(
            "No se pudo cargar la imagen:\n{}",
            err
        ))),
        Ok(img) => {
            let texture = Texture::from_color_image(img, context);
            let (airp_lat, airp_lon) = airport.position;
            Ok(Image::new(
                texture,
                Position::from_lat_lon(airp_lat, airp_lon),
            ))
        }
    }
}

/// Devuelve el nivel de zoom aceptable para mostrar el aeropuerto según el [tipo](AirportType).
pub fn zoom_is_showable(airport_type: &AirportType, zoom: Float) -> bool {
    match airport_type {
        AirportType::LargeAirport => zoom >= 0.0,
        AirportType::MediumAirport => zoom >= 5.0,
        AirportType::SmallAirport => zoom >= 10.0,
        AirportType::Heliport => zoom >= 10.0,
        AirportType::SeaplaneBase => zoom >= 10.0,
        AirportType::BalloonBase => zoom >= 10.0,
        AirportType::Closed => zoom >= 10.0,
    }
}
