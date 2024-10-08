//! Módulo para enumerar niveles de consistencia.

use crate::protocol::aliases::types::{Byte, Short};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;

/// Nivela los modos de consistencia para los _read request_.
///
/// TODO: dejar mejores descripciones.
pub enum Consistency {
    /// Buscar cualquier nodo
    Any,

    /// Buscar un único nodo
    One,

    /// Buscar dos nodos
    Two,

    /// Buscar tres nodos
    Three,

    /// Decidir por mayoría
    Quorum,

    /// Buscar TODOS los nodos disponibles
    All,

    /// Decidir por mayoría local
    LocalQuorum,

    /// Decidir por mayoría _#NoTengoNiIdeaDeLaDiferencia_
    EachQuorum,

    /// SERIAL Variant
    Serial,

    /// LOCAL_SERIAL Variant
    LocalSerial,

    /// LOCAL_ONE Variant
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

impl TryFrom<Vec<Byte>> for Consistency {
    type Error = Error;
    fn try_from(short: Vec<Byte>) -> Result<Self, Self::Error> {
        let bytes_array: [Byte; 2] = match short.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en Consistency".to_string(),
                ))
            }
        };
        let value = Short::from_be_bytes(bytes_array);
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
