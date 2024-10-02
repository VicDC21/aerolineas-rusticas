//! Módulo para las flags de un opcode PREPARE.

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

impl Maskable<i32> for PrepareFlag {
    fn base_mask() -> i32 {
        0
    }

    fn collapse(&self) -> i32 {
        let bytes = self.as_bytes();
        i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}
