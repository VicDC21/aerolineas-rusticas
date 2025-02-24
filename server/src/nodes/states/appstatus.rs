//! Módulo para el estado de un nodo.

use {
    protocol::{
        aliases::{results::Result, types::Byte},
        errors::error::Error,
        traits::Byteable,
    },
    std::convert::TryFrom,
};

/// El estado actual de un nodo.
#[derive(Debug, Clone, PartialEq)]
pub enum AppStatus {
    /// El nodo funciona normalmente.
    Normal,

    /// El nodo se está conectando.
    Bootstrap,

    /// El nodo esta siendo dado de baja.
    Left,

    /// El nodo esta siendo dado de baja porque no se puede acceder a él.
    Remove,

    /// El nodo no está respondiendo a los mensajes.
    Offline,

    /// El nodo esta listo para empezar la realocacion.
    RelocationIsNeeded,

    /// El nodo está relocalizando su data.
    RelocatingData,

    /// El nodo esta listo para pasarse a estado `Normal`.
    Ready,

    /// El nodo es nuevo en el cluster.
    NewNode,

    /// El nodo está actualizando las tablas de sus réplicas.
    UpdatingReplicas,
}

impl Byteable for AppStatus {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Normal => vec![0x0],
            Self::Bootstrap => vec![0x1],
            Self::Left => vec![0x2],
            Self::Remove => vec![0x3],
            Self::Offline => vec![0x4],
            Self::RelocatingData => vec![0x5],
            Self::Ready => vec![0x6],
            Self::RelocationIsNeeded => vec![0x7],
            Self::NewNode => vec![0x8],
            Self::UpdatingReplicas => vec![0x9],
        }
    }
}

impl TryFrom<&[Byte]> for AppStatus {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(Error::ServerError(
                "El conjunto de bytes está vacío.".to_string(),
            ));
        }

        let first = bytes[0];
        match first {
            0x0 => Ok(Self::Normal),
            0x1 => Ok(Self::Bootstrap),
            0x2 => Ok(Self::Left),
            0x3 => Ok(Self::Remove),
            0x4 => Ok(Self::Offline),
            0x5 => Ok(Self::RelocatingData),
            0x6 => Ok(Self::Ready),
            0x7 => Ok(Self::RelocationIsNeeded),
            0x8 => Ok(Self::NewNode),
            0x9 => Ok(Self::UpdatingReplicas),
            _ => Err(Error::ServerError(format!(
                "El ID '{}' no corresponde a ningún estado de aplicación.",
                first
            ))),
        }
    }
}
