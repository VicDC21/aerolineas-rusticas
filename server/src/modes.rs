//! Módulo para modos de conección al servidor.

use {
    protocol::{
        aliases::{results::Result, types::Byte},
        errors::error::Error,
        traits::Byteable,
    },
    std::convert::TryFrom,
};

/// Indica el modo de conexión al instanciar el servidor.
#[derive(Clone, Debug)]
pub enum ConnectionMode {
    /// Modo de prueba para testear conexión.
    Echo,

    /// El modo general para parsear _queries_ de CQL.
    Parsing,
}

impl Byteable for ConnectionMode {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Echo => vec![0x0],
            Self::Parsing => vec![0x1],
        }
    }
}

impl TryFrom<&[Byte]> for ConnectionMode {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(Error::ServerError(
                "El conjunto de bytes está vacío.".to_string(),
            ));
        }

        let first = bytes[0];
        match first {
            0x0 => Ok(Self::Echo),
            0x1 => Ok(Self::Parsing),
            _ => Err(Error::ServerError(format!(
                "El ID '{first}' no corresponde a ningún modo de conexión."
            ))),
        }
    }
}
