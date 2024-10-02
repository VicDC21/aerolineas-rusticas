//! Módulo para las flags de un _query_ en un _request_.

use crate::cassandra::traits::{Byteable, Maskable};

/// Flags específicas a mandar con un _query_.
///
/// Como flags que son, se pueden acumular en un sólo número:
/// ```rust
/// # use aerolineas::cassandra::messages::requests::query_flags::QueryFlag;
/// # use aerolineas::cassandra::traits::Maskable;
/// let q_flags = [&QueryFlag::Values, &QueryFlag::SkipMetadata, &QueryFlag::WithKeyspace];
/// let expected: i32 = 131; // 00000001 | 00000010 | 10000000 = 10000011
/// assert_eq!(QueryFlag::accumulate(&q_flags[..]), expected);
/// ```
pub enum QueryFlag {
    /// Valores son dados como variables para el _query_.
    Values,

    /// El resultado de la _response_ tendrá activada la flag [NO_METADATA](crate::cassandra::messages::responses::result::rows_flags::RowsFlag::NoMetadata).
    SkipMetadata,

    /// Controla la cantidad de filas a devolver por vez.
    PageSize,

    /// La _query_ se ejecuta usando un _paging state_ dado en un _response_ anterior.
    WithPagingState,

    /// Usa consistencia del tipo [SERIAL](crate::cassandra::notations::consistency::Consistency::Serial) o [LOCAL_SERIAL](crate::cassandra::notations::consistency::Consistency::LocalSerial).
    WithSerialConsistency,

    /// Indica que el _query_ trae un _timestamp_ en microsegundos.
    WithDefaultTimestamp,

    /// _(Sólo tiene sentido si [VALUES](crate::cassandra::messages::requests::query_flags::QueryFlag::Values) está seteado)_.
    /// Los valores son precedidos por un nombre.
    WithNamesForValues,

    /// Un _string_ indicando en qué _keyspace_ debería ejecutarse esta _query_.
    WithKeyspace,

    /// ***(Opcional)*** Indica el tiempo actual de la _query_. Diseñada para casos de _testing_.
    WithNowInSeconds,
}

impl Byteable for QueryFlag {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Values => vec![0, 0, 0, 1],
            Self::SkipMetadata => vec![0, 0, 0, 2],
            Self::PageSize => vec![0, 0, 0, 4],
            Self::WithPagingState => vec![0, 0, 0, 8],
            Self::WithSerialConsistency => vec![0, 0, 0, 16],
            Self::WithDefaultTimestamp => vec![0, 0, 0, 32],
            Self::WithNamesForValues => vec![0, 0, 0, 64],
            Self::WithKeyspace => vec![0, 0, 0, 128],
            Self::WithNowInSeconds => vec![0, 0, 1, 0],
        }
    }
}

impl Maskable<i32> for QueryFlag {
    fn base_mask() -> i32 {
        0
    }

    fn collapse(&self) -> i32 {
        let bytes = self.as_bytes();
        i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}
