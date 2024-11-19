//! Módulo para las flags de un mensaje.

use crate::protocol::aliases::types::Byte;
use crate::protocol::errors::error::Error;
use crate::protocol::traits::{Byteable, Maskable};

/// Una flag afecta al frame del mensaje.
///
/// También se puede acumular las flags en un sólo byte:
/// ```rust
/// # use aerolineas_rusticas::protocol::headers::flags::Flag;
/// # use aerolineas_rusticas::protocol::traits::Maskable;
/// # use aerolineas_rusticas::protocol::aliases::types::Byte;
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

impl Flag {
    /// Descompone un valor en los tipos de máscaras.
    pub fn decompose(base: &Byte) -> Vec<Self> {
        let mut masks_vec = Vec::<Self>::new();

        for flag_type in [
            Self::Compression,
            Self::Tracing,
            Self::CustomPayload,
            Self::Warning,
            Self::Beta,
            Self::Default,
        ] {
            if Self::has_mask(base, &flag_type) {
                masks_vec.push(flag_type);
            }
        }

        masks_vec
    }
}

impl Byteable for Flag {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Compression => vec![0x1],
            Self::Tracing => vec![0x2],
            Self::CustomPayload => vec![0x4],
            Self::Warning => vec![0x8],
            Self::Beta => vec![0x10],
            Self::Default => vec![0x0],
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

#[cfg(test)]
mod tests {
    use crate::protocol::aliases::types::Byte;
    use crate::protocol::headers::flags::Flag;
    use crate::protocol::traits::{Byteable, Maskable};

    #[test]
    fn test_1_serializar_bien() {
        let flags: [Flag; 4] = [Flag::Compression, Flag::Tracing, Flag::Warning, Flag::Beta];
        let serials: [Byte; 4] = [0x1, 0x2, 0x8, 0x10];

        for i in 0..flags.len() {
            let flag_bytes = flags[i].as_bytes();
            assert_eq!(flag_bytes.len(), 1);
            assert_eq!(flag_bytes[0], serials[i]);
        }
    }

    #[test]
    fn test_2_deserializar_correctamente() {
        let compression_res = Flag::try_from(0x1);
        let tracing_res = Flag::try_from(0x2);
        let custom_payload_res = Flag::try_from(0x4);
        let beta_res = Flag::try_from(0x10);

        assert!(compression_res.is_ok());
        if let Ok(compression) = compression_res {
            assert!(matches!(compression, Flag::Compression));
        }

        assert!(tracing_res.is_ok());
        if let Ok(tracing) = tracing_res {
            assert!(matches!(tracing, Flag::Tracing));
        }

        assert!(custom_payload_res.is_ok());
        if let Ok(custom_payload) = custom_payload_res {
            assert!(matches!(custom_payload, Flag::CustomPayload));
        }

        assert!(beta_res.is_ok());
        if let Ok(beta) = beta_res {
            assert!(matches!(beta, Flag::Beta));
        }
    }

    #[test]
    fn test_3_id_incorrecto_tira_bytes_vacios() {
        let cualquiera = Flag::try_from(0xFF);

        assert!(cualquiera.is_ok());
        if let Ok(default_flag) = cualquiera {
            assert!(matches!(default_flag, Flag::Default));
        }
    }

    #[test]
    fn test_4_combina_mascaras() {
        let valores = [
            &Flag::Compression,
            &Flag::Tracing,
            &Flag::Warning,
            &Flag::Beta,
        ];
        let mascara = Flag::accumulate(&valores[..]);

        assert_eq!(mascara, 0b00011011);
    }
}
