//! Módulo para un "valor" como lo es en la notaciones del protocolo de Cassandra.

use std::convert::TryFrom;

use crate::protocol::aliases::types::{Byte, Int};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;

/// Un valor cambia de significado según el [Int] utilizado para inicializarlo.
pub enum Value {
    /// Si un [Int] `n` cumple `n >= 0`, el valor será un vector de `n` [Byte]s.
    Regular(Vec<Byte>),

    /// Si un [Int] `n` cumple `n == -1`, el valor refiere a un `null`.
    Null,

    /// Si un [Int] `n` cumple `n == -2`, el valor refiere a un `not set`, indicando que no
    /// se deberían aplicar cambios.
    NotSet,
}

impl Byteable for Value {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Regular(bytes) => {
                let bytes_len = bytes.len().to_le_bytes();
                let mut bytes_vec = vec![bytes_len[3], bytes_len[2], bytes_len[1], bytes_len[0]];
                bytes_vec.extend(bytes);
                bytes_vec
            }
            Self::Null => {
                vec![0xFF, 0xFF, 0xFF, 0xFF]
            }
            Self::NotSet => {
                vec![0xFF, 0xFF, 0xFF, 0xFE]
            }
        }
    }
}

impl TryFrom<Vec<Byte>> for Value {
    type Error = Error;
    fn try_from(bytes_vec: Vec<Byte>) -> Result<Self, Self::Error> {
        if bytes_vec.len() < 4 {
            return Err(Error::ProtocolError(
                "Se esperan al menos 4 bytes para denominar la longitud del valor.".to_string(),
            ));
        }

        let length = &bytes_vec[0..4];
        if length[0] == 0xFF {
            // es negativo
            match length {
                [0xFF, 0xFF, 0xFF, 0xFF] => {
                    return Ok(Self::Null);
                }
                [0xFF, 0xFF, 0xFF, 0xFE] => {
                    return Ok(Self::NotSet);
                }
                _ => {
                    return Err(Error::ProtocolError(format!(
                        "valor '{}' no aceptado como ID de value.",
                        Int::from_be_bytes([length[0], length[1], length[2], length[3]])
                    )));
                }
            }
        }
        Ok(Self::Regular(bytes_vec[5..].to_vec()))
    }
}
