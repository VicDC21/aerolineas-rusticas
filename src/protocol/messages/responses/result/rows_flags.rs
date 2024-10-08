//! Flags para una _response_ RESULT de filas.

use crate::protocol::aliases::types::{Byte, Int};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::{Byteable, Maskable};

/// Las flags a ser incluidas en el mensaje de una _response_ RESULT de tipo [ROWS](crate::protocol::messages::responses::result_kinds::ResultKind::Rows).
pub enum RowsFlag {
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
            Self::GlobalTablesSpec => vec![0x0, 0x0, 0x0, 0x1],
            Self::HasMorePages => vec![0x0, 0x0, 0x0, 0x2],
            Self::NoMetadata => vec![0x0, 0x0, 0x0, 0x4],
        }
    }
}

impl TryFrom<Vec<Byte>> for RowsFlag {
    type Error = Error;
    fn try_from(int: Vec<Byte>) -> Result<Self, Self::Error> {
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
