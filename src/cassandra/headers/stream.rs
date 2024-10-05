//! Módulo para un header de stream.

use crate::cassandra::aliases::types::{Byte, Short};
use crate::cassandra::errors::error::Error;

/// Cada frame tiene un stream id para hacer coincidir el IDs entre las requests y responses.
pub struct Stream {
    /// El ID del stream.
    id: Short,
}

impl Stream {
    /// Crea un nuevo header de Stream.
    pub fn new(id: Short) -> Self {
        Self { id }
    }

    /// Transforma el ID en una secuencia de dos bytes.
    pub fn as_bytes(&self) -> Vec<Byte> {
        self.id.to_be_bytes().to_vec()
    }
}

impl TryFrom<Vec<Byte>> for Stream {
    type Error = Error;
    fn try_from(short: Vec<Byte>) -> Result<Self, Self::Error> {
        let bytes_array: [Byte; 2] = match short.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en Stream".to_string(),
                ))
            }
        };
        let value = Short::from_be_bytes(bytes_array);
        Ok(Stream { id: value })
    }
}
