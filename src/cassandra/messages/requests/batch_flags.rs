//! Módulo para las flags de una instrucción BATCH.

use crate::cassandra::traits::{Byteable, Maskable};

/// Flags para una instrucción de tipo BATCH.
///
/// Es similar a una Flag de una QUERY, sólo que las flags `0x01`, `0x02`, `0x04` y `0x08`
/// son aquí omitidas porque no tienen sentido en el contexto de un BATCH.
///
/// /// ```rust
/// # use aerolineas::cassandra::messages::requests::batch_flags::BatchFlag;
/// # use aerolineas::cassandra::traits::Maskable;
/// let b_flags = [&BatchFlag::WithSerialConsistency, &BatchFlag::WithKeySpace];
/// let expected: i32 = 144; // 00010000 | 10000000 = 10010000
/// assert_eq!(QueryFlag::accumulate(&b_flags[..]), expected);
/// ```
pub enum BatchFlag {
    /// Usa consistencia del tipo [SERIAL](crate::cassandra::notations::consistency::Consistency::Serial) o [LOCAL_SERIAL](crate::cassandra::notations::consistency::Consistency::LocalSerial).
    WithSerialConsistency,

    /// Indica que el BATCH_ trae un _timestamp_ en microsegundos.
    WithDefaultTimestamp,

    /// Esto tiene un significado similar al de las [Query Flags](crate::cassandra::messages::requests::query_flags::QueryFlag::Values).
    /// Sin embargo, esta _feature_ **NO FUNCIONA**, y seguún el protocolo de Cassandra, será arreglado en el futuro.
    ///
    /// _(Ergo, usar esto debería levantar errores)_
    WithNamesForValues,

    /// Un _string_ indicando en qué _keyspace_ debería ejecutarse esta _query_.
    WithKeyspace,

    /// ***(Opcional)*** Indica el tiempo actual de la _query_. Diseñada para casos de _testing_.
    WithNowInSeconds,
}

impl Byteable for BatchFlag {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::WithSerialConsistency => vec![0, 0, 0, 16],
            Self::WithDefaultTimestamp => vec![0, 0, 0, 32],
            Self::WithNamesForValues => vec![0, 0, 0, 64],
            Self::WithKeyspace => vec![0, 0, 0, 128],
            Self::WithNowInSeconds => vec![0, 0, 1, 0],
        }
    }
}

impl Maskable<i32> for BatchFlag {
    fn base_mask() -> i32 {
        0
    }

    fn collapse(&self) -> i32 {
        let bytes = self.as_bytes();
        i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}
