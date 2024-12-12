//! Módulo para un "valor" como lo es en la notaciones del protocolo de Cassandra.

use std::convert::TryFrom;

use crate::protocol::{
    aliases::types::{Byte, Int},
    errors::error::Error,
    traits::Byteable,
};

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
        Ok(Self::Regular(bytes_vec[4..].to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_serializar() {
        let value = Value::Regular(vec![0x0, 0x2, 0x0, 0x1]);
        let serialized = value.as_bytes();
        assert_eq!(serialized.len(), 8);
        assert_eq!(serialized, [0x0, 0x0, 0x0, 0x4, 0x0, 0x2, 0x0, 0x1]);

        let value = Value::Null;
        let serialized = value.as_bytes();
        assert_eq!(serialized.len(), 4);
        assert_eq!(serialized, [0xFF, 0xFF, 0xFF, 0xFF]);

        let value = Value::NotSet;
        let serialized = value.as_bytes();
        assert_eq!(serialized.len(), 4);
        assert_eq!(serialized, [0xFF, 0xFF, 0xFF, 0xFE]);
    }

    #[test]
    fn test_2_deserializar() {
        let value_res = Value::try_from([0x0, 0x0, 0x0, 0x4, 0x0, 0x2, 0x0, 0x1].to_vec());
        assert!(value_res.is_ok());
        if let Ok(Value::Regular(bytes)) = value_res {
            assert_eq!(bytes.len(), 4);
            assert_eq!(bytes, [0x0, 0x2, 0x0, 0x1]);
        }

        let value_res = Value::try_from([0xFF, 0xFF, 0xFF, 0xFF].to_vec());
        assert!(value_res.is_ok());
        if let Ok(value_ok) = value_res {
            assert!(matches!(value_ok, Value::Null));
        }

        let value_res = Value::try_from([0xFF, 0xFF, 0xFF, 0xFE].to_vec());
        assert!(value_res.is_ok());
        if let Ok(value_ok) = value_res {
            assert!(matches!(value_ok, Value::NotSet));
        }
    }

    #[test]
    fn test_3_deserializar_error() {
        let value_res = Value::try_from([0xFF, 0x2, 0x0, 0x1].to_vec());

        assert!(value_res.is_err());
        if let Err(err) = value_res {
            assert!(matches!(err, Error::ProtocolError(_)));
        }
    }
}
