//! Traits en común con objetos del servidor.

use std::str::Lines;

use crate::protocol::{aliases::results::Result, errors::error::Error};

/// Serializa o deserializa un objeto en bytes para escribirlo o leerlo en un archivo `.csv`.
pub trait Serializable {
    /// Serializa el objeto en bytes para que sea compatible escribirlo en un archivo `.csv`.
    fn serialize(&self) -> Vec<u8>;

    /// Deserializa los bytes en un objeto del tipo deseado desde un archivo `.csv`.
    fn deserialize(data: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

impl<T: Serializable> Serializable for Vec<T> {
    /// Toma un vector de elementos que pueden serializarse.
    /// Cada elemento del vector termina siendo una linea de un archivo csv.
    fn serialize(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        for elemento in self {
            data.extend(elemento.serialize());
            data.push(b'\n');
        }

        data
    }

    /// Toma un conjunto de bytes, lo convierte a un string, se toma cada
    /// linea en formato csv, se deserializa la linea obteniendo un elemento
    /// de tipo genérico, y ese elemento se añade a un vector.
    fn deserialize(data: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut res: Vec<T> = Vec::new();

        let text = String::from_utf8(data.to_vec())
            .map_err(|_| Error::ServerError("No se pudieron deserializar los datos".to_string()))?;
        let lines: Lines<'_> = text.lines();

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            let bytes: &[u8] = line.as_bytes();
            let elem: T = T::deserialize(bytes)?;
            res.push(elem);
        }

        Ok(res)
    }
}
