//! Flags para una _response_ RESULT de filas.

use crate::protocol::{
    aliases::{
        results::Result,
        types::{Byte, Int},
    },
    errors::error::Error,
    traits::{Byteable, Maskable},
};

/// Las flags a ser incluidas en el mensaje de una _response_ RESULT de tipo [ROWS](crate::protocol::messages::responses::result_kinds::ResultKind::Rows).
/// ```rust
/// # use aerolineas_rusticas::protocol::messages::responses::result::rows_flags::RowsFlag;
/// # use aerolineas_rusticas::protocol::traits::Maskable;
/// # use aerolineas_rusticas::protocol::aliases::types::Int;
/// let b_flags = [&RowsFlag::GlobalTablesSpec, &RowsFlag::HasMorePages];
/// let expected: Int = 0b00000011; // 00000001 | 00000010 = 00000011
/// assert_eq!(RowsFlag::accumulate(&b_flags[..]), expected);
/// ```
pub enum RowsFlag {
    /// Flag por default
    Default,

    /// Sólo un table spec es provisto.
    GlobalTablesSpec,

    /// Indica si esta es la última página del resultado y se debería pedir más datos.
    HasMorePages,

    /// Si se activa, los metadatos del mensaje incluyen sólo estos flags.
    NoMetadata,
}

impl Byteable for RowsFlag {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Default => vec![0x0, 0x0, 0x0, 0x0],
            Self::GlobalTablesSpec => vec![0x0, 0x0, 0x0, 0x1],
            Self::HasMorePages => vec![0x0, 0x0, 0x0, 0x2],
            Self::NoMetadata => vec![0x0, 0x0, 0x0, 0x4],
        }
    }
}

impl TryFrom<Vec<Byte>> for RowsFlag {
    type Error = Error;
    fn try_from(int: Vec<Byte>) -> Result<Self> {
        let bytes_array: [Byte; 4] = match int.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en RowsFlag".to_string(),
                ))
            }
        };

        let value = Int::from_be_bytes(bytes_array);
        match value {
            0x0000 => Ok(RowsFlag::Default),
            0x0001 => Ok(RowsFlag::GlobalTablesSpec),
            0x0002 => Ok(RowsFlag::HasMorePages),
            0x0004 => Ok(RowsFlag::NoMetadata),
            _ => Err(Error::ConfigError(
                "La flag indicada para rows no existe".to_string(),
            )),
        }
    }
}

impl Maskable<Int> for RowsFlag {
    fn base_mask() -> Int {
        0
    }

    fn collapse(&self) -> Int {
        let bytes = self.as_bytes();
        Int::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_serializar() {
        let rows_flags = [
            RowsFlag::GlobalTablesSpec,
            RowsFlag::HasMorePages,
            RowsFlag::NoMetadata,
        ];
        let expected = [
            vec![0x0, 0x0, 0x0, 0x1],
            vec![0x0, 0x0, 0x0, 0x2],
            vec![0x0, 0x0, 0x0, 0x4],
        ];

        for i in 0..expected.len() {
            let serialized = rows_flags[i].as_bytes();
            assert_eq!(serialized.len(), 4);
            assert_eq!(serialized, expected[i]);
        }
    }

    #[test]
    fn test_2_deserializar() {
        let rows_res = RowsFlag::try_from(vec![0x0, 0x0, 0x0, 0x2]);
        assert!(rows_res.is_ok());
        if let Ok(void) = rows_res {
            assert!(matches!(void, RowsFlag::HasMorePages));
        }
    }

    #[test]
    fn test_3_deserializar_error() {
        let rows_res = RowsFlag::try_from(vec![0x0, 0x0, 0x0, 0x3]);

        assert!(rows_res.is_err());
        if let Err(err) = rows_res {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }
}
