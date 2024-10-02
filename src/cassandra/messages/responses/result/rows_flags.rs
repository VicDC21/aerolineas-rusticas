//! Flags para una _response_ RESULT de filas.

use crate::cassandra::traits::{Byteable, Maskable};

/// Las flags a ser incluidas en el mensaje de una _response_ RESULT de tipo [ROWS](crate::cassandra::messages::responses::result_kinds::ResultKind::Rows).
pub enum RowsFlag {
    /// Sólo un table spec es provisto.
    GlobalTablesSpec,

    /// Indica si esta es la última página del resultado y se debería pedir más datos.
    HasMorePages,

    /// Si se activa, los metadatos del mensaje incluyen sólo estos flags.
    NoMetadata,
}

impl Byteable for RowsFlag {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::GlobalTablesSpec => vec![0, 0, 0, 1],
            Self::HasMorePages => vec![0, 0, 0, 2],
            Self::NoMetadata => vec![0, 0, 0, 4],
        }
    }
}

impl Maskable<i32> for RowsFlag {
    fn base_mask() -> i32 {
        0
    }

    fn collapse(&self) -> i32 {
        let bytes = self.as_bytes();
        i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}
