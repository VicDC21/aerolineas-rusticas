//! Tipos de _requests_ de tipo BATCH.

use crate::cassandra::errors::error::Error;
use crate::cassandra::traits::Byteable;

/// El tipo de una instrucción BATCH, que es un conjunto de _queries_.
pub enum BatchType {
    /// Equivalente a una instrucción normal de BATCH CQL3.
    Logged,

    /// El BATCH será unlogueado.
    Unlogged,

    /// Sentencias _"non-counter"_ son ignoradas.
    Counter,
}

impl Byteable for BatchType {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Logged => vec![0],
            Self::Unlogged => vec![1],
            Self::Counter => vec![2],
        }
    }
}

impl TryFrom<u8> for BatchType {
    type Error = Error;
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0x0 => Ok(BatchType::Logged),
            0x1 => Ok(BatchType::Unlogged),
            0x2 => Ok(BatchType::Counter),
            _ => Err(Error::ConfigError("Tipo de Batch no existente".to_string())),
        }
    }
}
