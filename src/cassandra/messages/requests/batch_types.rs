//! Tipos de _requests_ de tipo BATCH.

use crate::cassandra::aliases::types::Byte;
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
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Logged => vec![0x0],
            Self::Unlogged => vec![0x1],
            Self::Counter => vec![0x2],
        }
    }
}

impl TryFrom<Byte> for BatchType {
    type Error = Error;
    fn try_from(byte: Byte) -> Result<Self, Self::Error> {
        match byte {
            0x00 => Ok(BatchType::Logged),
            0x01 => Ok(BatchType::Unlogged),
            0x02 => Ok(BatchType::Counter),
            _ => Err(Error::ConfigError("Tipo de Batch no existente".to_string())),
        }
    }
}
