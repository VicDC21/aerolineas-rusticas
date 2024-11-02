//! Traits en común con objetos del protocolo de Cassandra.

use std::{
    ops::{BitAnd, BitOrAssign},
    str::Lines,
};

use crate::protocol::aliases::{results::Result, types::Byte};

use super::errors::error::Error;

/// Colapsa una propiedad en una colección de bytes.
pub trait Byteable {
    /// Transforma el objeto en un vector de bytes.
    fn as_bytes(&self) -> Vec<Byte>;
}

/// Une muchas propiedades (pensadas como máscaras) en un sólo número.
pub trait Maskable<T: BitOrAssign + BitAnd<Output = T> + Copy + PartialEq> {
    /// Devuelve un acumulador del tipo T para las máscaras.
    fn base_mask() -> T;

    /// Convierte una propiedad en un número binario.
    fn collapse(&self) -> T;

    /// Comprueba si el elemento acumulado tiene los bits especificados.
    fn has_bits(base: T, mask: T) -> bool {
        base & mask == mask
    }

    /// Verifica si en una colección de máscaras se encuentra una en particular.
    fn has_mask(masks: &T, mask: &Self) -> bool {
        Self::has_bits(masks.to_owned(), mask.collapse())
    }

    /// Une todas las máscaras.
    fn accumulate(masks: &[&Self]) -> T {
        let mut accumulator = Self::base_mask();
        for msk in masks {
            accumulator |= msk.collapse();
        }
        accumulator
    }
}

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
