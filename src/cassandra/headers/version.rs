//! Módulo para la versión del protocolo.

use crate::cassandra::errors::error::Error;
use std::convert::TryFrom;

use crate::cassandra::traits::Byteable;

/// La 'versión' indica tanto la versión del protocolo a usar,
/// así como si se trata con un _request_ o un _response_.
pub enum Version {
    /// _Request_ del protocolo nativo de Cassandra (Versión 3).
    RequestV3,

    /// _Response_ del protocolo nativo de Cassandra (Versión 3).
    ResponseV3,

    /// _Request_ del protocolo nativo de Cassandra (Versión 4).
    RequestV4,

    /// _Response_ del protocolo nativo de Cassandra (Versión 4).
    ResponseV4,

    /// _Request_ del protocolo nativo de Cassandra (Versión 5).
    RequestV5,

    /// _Response_ del protocolo nativo de Cassandra (Versión 5).
    ResponseV5,
}

impl Byteable for Version {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::RequestV3 => vec![3],
            Self::ResponseV3 => vec![131],
            Self::RequestV4 => vec![4],
            Self::ResponseV4 => vec![132],
            Self::RequestV5 => vec![5],
            Self::ResponseV5 => vec![133],
        }
    }
}

impl TryFrom<u8> for Version {
    type Error = Error;
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0x03 => Ok(Version::RequestV3),
            0x83 => Ok(Version::ResponseV3),
            0x04 => Ok(Version::RequestV4),
            0x84 => Ok(Version::ResponseV4),
            0x05 => Ok(Version::RequestV5),
            0x85 => Ok(Version::ResponseV5),
            _ => Err(Error::ConfigError(
                "La version del protocolo especificada no existe".to_string(),
            )), // Falta definir el error
                // Puede que falte el caso en el que el startup manda una version mas alta a la actual,
                // en ese caso devolvemos la version actual.
        }
    }
}
