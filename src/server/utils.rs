//! Módulo para funciones auxiliares del servidor.

use std::fs::{self, read, write, File};

use serde::{Deserialize, Serialize};

use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
};

use super::traits::Serializable;

/// Toma un vector con elementos de tipo genérico T, que pueden serializarse,
/// y crea un nuevo vector con cada elemento serializado. Cada elemento
/// termina siendo una linea de un archivo csv.
pub fn serialize_vec<T: Serializable>(vec: &Vec<T>) -> Vec<Byte> {
    vec.serialize()
}

/// Toma un conjunto de bytes que se pueden deserializar en elementos, y crea
/// un vector con elementos de tipo genérico leido de los bytes.
pub fn deserialize_vec<T: Serializable>(data: &[Byte]) -> Result<Vec<T>> {
    Vec::<T>::deserialize(data)
}

/// Toma un elemento genérico serializable, lo serializa, y escribe
/// el contenido serializable a la ruta recibida por parámetro
pub fn store_serializable<T: Serializable>(serializable: &T, path: &str) -> Result<()> {
    let data: Vec<Byte> = serializable.serialize();

    write(path, data).map_err(|e| Error::ServerError(format!("Error escribiendo datos: {}", e)))
}

/// Toma la ruta al nombre de un archivo, cuyo contenido es serializable,
/// lo deserealiza y devuelve el contenido
pub fn load_serializable<T: Serializable>(path: &str) -> Result<T> {
    let data: Vec<Byte> =
        read(path).map_err(|e| Error::ServerError(format!("Error leyendo datos: {}", e)))?;

    T::deserialize(&data)
        .map_err(|e| Error::ServerError(format!("Error deserializando datos: {}", e)))
}

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
