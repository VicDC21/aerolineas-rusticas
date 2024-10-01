//! Módulo para las flags de un _query_ en un _request_.

use crate::cassandra::traits::{Byteable, Maskable};

/// Flags específicas a mandar con un _query_.
pub enum QueryFlag {
    /// Valores son dados como variables para el _query_.
    Values,

    /// El resultado de la _response_ tendrá activada la flag [NO_METADATA](crate::cassandra::messages::responses::).
    SkipMetadata,

    /// Controla la cantidad de filas a devolver por vez.
    PageSize,

    /// La _query_ se ejecuta usando un _paging state_ dado en un _response_ anterior.
    WithPagingState,

    /// Usa consistencia del tipo [SERIAL](crate::cassandra::notations::consistency::Consistency::Serial) o [LOCAL_SERIAL](crate::cassandra::notations::consistency::Consistency::LocalSerial).
    WithSerialConsistency,

    /// Indica que el _query_ trae un _timestamp_ en microsegundos.
    WithDefaultTimestamp,

    /// Un _string_ indicando en qué _keyspace_ debería ejecutarse esta _query_.
    WithKeyspace,

    /// ***(Opcional)*** Indica el tiempo actual de la _query_. Diseñada para casos de _testing_.
    WithNowInSeconds
}

impl Byteable for QueryFlag {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Values => &[0, 1],
            Self::SkipMetadata => &[0, 2],
            Self::PageSize => &[0, 4],
            Self::WithPagingState => &[0, 8],
            Self::WithSerialConsistency => &[0, 16],
            Self::WithDefaultTimestamp => &[0, 32],
            Self::WithKeyspace => &[0, 64],
            Self::WithNowInSeconds => &[0, 128]
        }
    }
}

impl Maskable<u16> for QueryFlag {
    fn base_mask() -> u16 {
        0
    }

    fn collapse(&self) -> u16 {
        let bytes = self.as_bytes();
        u16::from_be_bytes([bytes[0], bytes[1]])
    }
}