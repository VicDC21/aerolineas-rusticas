//! Tipos de _requests_ de tipo BATCH.

use crate::protocol::aliases::types::Byte;
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;

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

#[cfg(test)]
mod tests {
    use crate::protocol::{
        errors::error::Error, messages::requests::batch_types::BatchType, traits::Byteable,
    };

    #[test]
    fn test_1_serializar() {
        let batch_types = [BatchType::Logged, BatchType::Unlogged, BatchType::Counter];

        let expected = [vec![0x0], vec![0x1], vec![0x2]];

        for i in 0..expected.len() {
            assert_eq!(batch_types[i].as_bytes(), expected[i]);
        }
    }

    #[test]
    fn test_2_deserializar() {
        let batch_res = BatchType::try_from(0x01);

        assert!(batch_res.is_ok());
        if let Ok(batch) = batch_res {
            assert!(matches!(batch, BatchType::Unlogged));
        }
    }

    #[test]
    fn test_3_deserializar_error() {
        let batch_res = BatchType::try_from(0x03);

        assert!(batch_res.is_err());
        if let Err(err) = batch_res {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }
}
