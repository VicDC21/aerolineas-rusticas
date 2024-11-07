//! Módulo para enumerar niveles de consistencia.

use crate::protocol::aliases::types::{Byte, Short};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;

/// Nivela los modos de consistencia para los _read request_.
#[derive(Debug, Clone, Copy)]
pub enum Consistency {
    /// Buscar cualquier nodo
    Any,

    /// Buscar un único nodo
    One,

    /// Buscar dos nodos
    Two,

    /// Buscar tres nodos
    Three,

    /// Decidir por mayoría, la mitad + 1 (51%)
    Quorum,

    /// Buscar TODOS los nodos disponibles
    All,

    /// Decidir por mayoría, en el data center local únicamente
    LocalQuorum,

    /// Decidir por mayoría, en cada data center
    EachQuorum,

    /// Bloquea la escritura hasta que la escritura se haya propagado a todos los nodos réplica
    Serial,

    /// Bloquea la escritura hasta que la escritura se haya propagado a todos los nodos réplica, en el data center local únicamente
    LocalSerial,

    /// Buscar un único nodo, en el data center local únicamente
    LocalOne,
}

impl Byteable for Consistency {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Any => vec![0x0, 0x0],
            Self::One => vec![0x0, 0x1],
            Self::Two => vec![0x0, 0x2],
            Self::Three => vec![0x0, 0x3],
            Self::Quorum => vec![0x0, 0x4],
            Self::All => vec![0x0, 0x5],
            Self::LocalQuorum => vec![0x0, 0x6],
            Self::EachQuorum => vec![0x0, 0x7],
            Self::Serial => vec![0x0, 0x8],
            Self::LocalSerial => vec![0x0, 0x9],
            Self::LocalOne => vec![0x0, 0xA],
        }
    }
}

impl TryFrom<&[Byte]> for Consistency {
    type Error = Error;
    fn try_from(short: &[Byte]) -> Result<Self, Self::Error> {
        if short.len() < 2 {
            return Err(Error::ConfigError(
                "El vector de bytes no tiene 2 bytes".to_string(),
            ));
        }

        let value = Short::from_be_bytes([short[0], short[1]]);
        match value {
            0x0000 => Ok(Consistency::Any),
            0x0001 => Ok(Consistency::One),
            0x0002 => Ok(Consistency::Two),
            0x0003 => Ok(Consistency::Three),
            0x0004 => Ok(Consistency::Quorum),
            0x0005 => Ok(Consistency::All),
            0x0006 => Ok(Consistency::LocalQuorum),
            0x0007 => Ok(Consistency::EachQuorum),
            0x0008 => Ok(Consistency::Serial),
            0x0009 => Ok(Consistency::LocalSerial),
            0x000A => Ok(Consistency::LocalOne),
            _ => Err(Error::ConfigError(
                "La correspondencia indicada para consistency no existe".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::errors::error::Error;
    use crate::protocol::notations::consistency::Consistency;
    use crate::protocol::traits::Byteable;

    #[test]
    fn test_1_serializar() {
        let consistencys = [
            Consistency::One,
            Consistency::Two,
            Consistency::Three,
            Consistency::Quorum,
            Consistency::LocalQuorum,
        ];
        let expected_bytes = [[0x0, 0x1], [0x0, 0x2], [0x0, 0x3], [0x0, 0x4], [0x0, 0x6]];

        for i in 0..expected_bytes.len() {
            let serialized = consistencys[i].as_bytes();
            assert_eq!(serialized.len(), 2);
            assert_eq!(serialized, expected_bytes[i]);
        }
    }

    #[test]
    fn test_2_deserializar() {
        let consistency_res = Consistency::try_from(&[0x0, 0x3][..]);

        assert!(consistency_res.is_ok());
        if let Ok(consistency) = consistency_res {
            assert!(matches!(consistency, Consistency::Three));
        }
    }

    #[test]
    fn test_3_deserializar_error() {
        let consistency_res = Consistency::try_from(&[0x0, 0xF][..]);

        assert!(consistency_res.is_err());
        if let Err(err) = consistency_res {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }
}
