//! Tipos de _requests_ de tipo BATCH.

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
