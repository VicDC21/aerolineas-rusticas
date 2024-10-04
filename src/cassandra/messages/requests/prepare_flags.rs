//! Módulo para las flags de un opcode PREPARE.

use crate::cassandra::errors::error::Error;
use crate::cassandra::traits::{Byteable, Maskable};

/// Flags para preparar una _query_ para posterior ejecución.
pub enum PrepareFlag {
    /// La _query_ tiene un _namespace_.
    WithKeyspace,
}

impl Byteable for PrepareFlag {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::WithKeyspace => vec![0, 0, 0, 1],
        }
    }
}

impl TryFrom<Vec<u8>> for PrepareFlag {
    type Error = Error;
    fn try_from(int: Vec<u8>) -> Result<Self, Self::Error> {
        let bytes_array: [u8; 4] = match int.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en PrepareFlag".to_string(),
                ))
            }
        };

        let value = i32::from_be_bytes(bytes_array);
        match value {
            0x0001 => Ok(PrepareFlag::WithKeyspace),
            _ => Err(Error::ConfigError(
                "La flag indicada para prepare no existe".to_string(),
            )),
        }
    }
}

impl Maskable<i32> for PrepareFlag {
    fn base_mask() -> i32 {
        0
    }

    fn collapse(&self) -> i32 {
        let bytes = self.as_bytes();
        i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}
