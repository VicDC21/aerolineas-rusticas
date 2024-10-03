//! Módulo para las flags de un mensaje.

use crate::cassandra::traits::{Byteable, Maskable};

/// Una flag afecta al frame del mensaje.
///
/// También se puede acumular las flags en un sólo byte:
/// ```rust
/// # use aerolineas::cassandra::headers::flags::Flag;
/// # use aerolineas::cassandra::traits::Maskable;
/// let flags = [&Flag::Compression, &Flag::Tracing, &Flag::Beta];
/// let expected: u8 = 19; // 00000001 | 00000010 | 00010000 = 00010011
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
}

impl Byteable for Flag {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Compression => vec![1],
            Self::Tracing => vec![2],
            Self::CustomPayload => vec![4],
            Self::Warning => vec![8],
            Self::Beta => vec![16],
        }
    }
}

impl Maskable<u8> for Flag {
    fn base_mask() -> u8 {
        0
    }

    fn collapse(&self) -> u8 {
        self.as_bytes()[0]
    }
}

impl TryFrom<u8> for Flag {
    type Error = ();
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0x01 => Ok(Flag::Compression),
            0x02 => Ok(Flag::Tracing),
            0x04 => Ok(Flag::CustomPayload),
            0x08 => Ok(Flag::Warning),
            0x10 => Ok(Flag::Beta),
            _ => Err(()), // Todo: Este caso deberia devolver una flag standard, porque los demas casos se ignoran
        }
    }
}
