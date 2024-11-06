//! Módulo para funciones auxiliares del servidor.

use std::fs::{read, write};

use crate::protocol::{aliases::results::Result, errors::error::Error};

use super::traits::Serializable;

/// Toma un vector con elementos de tipo genérico T, que pueden serializarse,
/// y crea un nuevo vector con cada elemento serializado. Cada elemento
/// termina siendo una linea de un archivo csv.
pub fn serialize_vec<T: Serializable>(vec: &Vec<T>) -> Vec<u8> {
    vec.serialize()
}

/// Toma un conjunto de bytes que se pueden deserializar en elementos, y crea
/// un vector con elementos de tipo genérico leido de los bytes.
pub fn deserialize_vec<T: Serializable>(data: &[u8]) -> Result<Vec<T>> {
    Vec::<T>::deserialize(data)
}

/// Toma un elemento genérico serializable, lo serializa, y escribe
/// el contenido serializable a la ruta recibida por parámetro
pub fn store_serializable<T: Serializable>(serializable: &T, path: &str) -> Result<()> {
    let data: Vec<u8> = serializable.serialize();

    write(path, data).map_err(|_| Error::ServerError("Error escribiendo datos".to_string()))
}

/// Toma la ruta al nombre de un archivo, cuyo contenido es serializable,
/// lo deserealiza y devuelve el contenido
pub fn load_serializable<T: Serializable>(path: &str) -> Result<T> {
    let data: Vec<u8> =
        read(path).map_err(|_| Error::ServerError("Error leyendo datos".to_string()))?;

    T::deserialize(&data).map_err(|_| Error::ServerError("Error deserializando datos".to_string()))
}
