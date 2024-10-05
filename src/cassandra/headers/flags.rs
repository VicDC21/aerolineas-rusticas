//! Módulo para las flags de un mensaje.

use crate::cassandra::aliases::types::Byte;
use crate::cassandra::errors::error::Error;
use crate::cassandra::traits::{Byteable, Maskable};

/// Una flag afecta al frame del mensaje.
///
/// También se puede acumular las flags en un sólo byte:
/// ```rust
/// # use aerolineas::cassandra::headers::flags::Flag;
/// # use aerolineas::cassandra::traits::Maskable;
/// # use aerolineas::cassandra::aliases::types::Byte;
/// let flags = [&Flag::Compression, &Flag::Tracing, &Flag::Beta];
/// let expected: Byte = 0b00010011; // 00000001 | 00000010 | 00010000 = 00010011
/// assert_eq!(Flag::accumulate(&flags[..]), expected);
/// ```
pub enum Flag {
    /// El body del frame es comprimido.
    Compression,

    /// Cuando el cliente pide un tracing del request.
    Tracing,

    /// Indica un payload para un KeyHandler personalizado.
    CustomPayload,

    /// Contiene warnings del server a ser mandados en el response.
    Warning,

    /// Indica que se opta por usar una versión del protocolo en estado de desarrollo BETA.
    Beta,

    /// Flag estandar a devolver para otro caso distinto a los esperados
    Default,
}

impl Byteable for Flag {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Compression => vec![0x1],
            Self::Tracing => vec![0x2],
            Self::CustomPayload => vec![0x4],
            Self::Warning => vec![0x8],
            Self::Beta => vec![0x10],
            Self::Default => vec![],
        }
    }
}

impl Maskable<Byte> for Flag {
    fn base_mask() -> Byte {
        0
    }

    fn collapse(&self) -> Byte {
        self.as_bytes()[0]
    }
}

impl TryFrom<Byte> for Flag {
    type Error = Error;
    fn try_from(byte: Byte) -> Result<Self, Self::Error> {
        match byte {
            0x01 => Ok(Flag::Compression),
            0x02 => Ok(Flag::Tracing),
            0x04 => Ok(Flag::CustomPayload),
            0x08 => Ok(Flag::Warning),
            0x10 => Ok(Flag::Beta),
            _ => Ok(Flag::Default),
        }
    }
}
