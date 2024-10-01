//! Módulo para las flags de un mensaje.

use crate::cassandra::traits::{Byteable, Maskable};

/// Una flag afecta al frame del mensaje.
///
/// También se puede acumular las flags en un sólo byte:
/// ```rust
/// # use aerolineas::cassandra::flags::Flag;
/// let flags = [Flag::Compression, Flag::Tracing, Flag::Beta];
/// let expected = 19; // 00000001 | 00000010 | 00010000 = 00010011
/// assert_eq!(Flag::mask(&flags), expected);
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
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Compression => &[1],
            Self::Tracing => &[2],
            Self::CustomPayload => &[4],
            Self::Warning => &[8],
            Self::Beta => &[16],
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
