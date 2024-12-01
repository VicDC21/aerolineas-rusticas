//! Módulo para funciones auxiliares del servidor.

use std::fs::{self, File};

use serde::{Deserialize, Serialize};

use crate::protocol::{
    aliases::results::Result,
    errors::error::Error,
};

/// Toma un elemento serializable y lo convierte a JSON, escribiendo el contenido en un archivo en la ruta recibida.
pub fn store_json<T: Serialize>(serializable: &T, path: &str) -> Result<()> {
    let file = File::create(path)
        .map_err(|e| Error::ServerError(format!("Error creando el archivo JSON: {}", e)))?;
    serde_json::to_writer_pretty(file, serializable)
        .map_err(|e| Error::ServerError(format!("Error escribiendo datos JSON: {}", e)))
}

/// Toma la ruta al nombre de un archivo JSON, cuyo contenido es serializable, lo deserealiza y devuelve el contenido.
pub fn load_json<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T> {
    let content = fs::read_to_string(path)
        .map_err(|e| Error::ServerError(format!("Error leyendo datos JSON: {}", e)))?;
    serde_json::from_str(&content)
        .map_err(|e| Error::ServerError(format!("Error deserializando datos JSON: {}", e)))
}
