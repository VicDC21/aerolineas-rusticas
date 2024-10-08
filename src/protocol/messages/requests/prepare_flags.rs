//! Módulo para las flags de un opcode PREPARE.

use crate::protocol::aliases::types::{Byte, Int};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::{Byteable, Maskable};

/// Flags para preparar una _query_ para posterior ejecución.
pub enum PrepareFlag {
    /// La _query_ tiene un _namespace_.
    WithKeyspace,
}

impl Byteable for PrepareFlag {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::WithKeyspace => vec![0x0, 0x0, 0x0, 0x1],
        }
    }
}

impl TryFrom<Vec<Byte>> for PrepareFlag {
    type Error = Error;
    fn try_from(int: Vec<Byte>) -> Result<Self, Self::Error> {
        let bytes_array: [Byte; 4] = match int.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en PrepareFlag".to_string(),
                ))
            }
        };

        let value = Int::from_be_bytes(bytes_array);
        match value {
            0x0001 => Ok(PrepareFlag::WithKeyspace),
            _ => Err(Error::ConfigError(
                "La flag indicada para prepare no existe".to_string(),
            )),
        }
    }
}

impl Maskable<Int> for PrepareFlag {
    fn base_mask() -> Int {
        0
    }

    fn collapse(&self) -> Int {
        let bytes = self.as_bytes();
        Int::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}
