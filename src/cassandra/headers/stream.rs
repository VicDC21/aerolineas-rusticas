//! Módulo para un header de stream.

use crate::cassandra::errors::error::Error;

/// Cada frame tiene un stream id para hacer coincidir el IDs entre las requests y responses.
pub struct Stream {
    /// El ID del stream.
    id: i16,
}

impl Stream {
    /// Crea un nuevo header de Stream.
    pub fn new(id: i16) -> Self {
        Self { id }
    }

    /// Transforma el ID en una secuencia de dos bytes.
    pub fn as_bytes(&self) -> [u8; 2] {
        self.id.to_be_bytes()
    }
}

impl TryFrom<Vec<u8>> for Stream {
    type Error = Error;
    fn try_from(short: Vec<u8>) -> Result<Self, Self::Error> {
        let bytes_array: [u8; 2] = match short.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en Stream".to_string(),
                ))
            }
        };
        let value = i16::from_be_bytes(bytes_array);
        Ok(Stream { id: value })
    }
}
