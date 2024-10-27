//! Módulo para funciones auxiliares de plugins.

use std::result::Result as StdResult;

use eframe::egui::{ColorImage, Context};
use image::{ImageError, ImageReader};
use walkers::{extras::Image, Texture};

use crate::data::airports::Airport;
use crate::protocol::{aliases::results::Result, errors::error::Error};

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
            Ok(Image::new(texture, airport.position))
        }
    }
}
