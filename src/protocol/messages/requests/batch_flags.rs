//! Módulo para las flags de una instrucción BATCH.

use crate::protocol::aliases::types::{Byte, Int};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::{Byteable, Maskable};

/// Flags para una instrucción de tipo BATCH.
///
/// Es similar a una Flag de una QUERY, sólo que las flags `0x01`, `0x02`, `0x04` y `0x08`
/// son aquí omitidas porque no tienen sentido en el contexto de un BATCH.
///
/// ```rust
/// # use aerolineas_rusticas::protocol::messages::requests::batch_flags::BatchFlag;
/// # use aerolineas_rusticas::protocol::traits::Maskable;
/// # use aerolineas_rusticas::protocol::aliases::types::Int;
/// let b_flags = [&BatchFlag::WithSerialConsistency, &BatchFlag::WithKeyspace];
/// let expected: Int = 0b10010000; // 00010000 | 10000000 = 10010000
/// assert_eq!(BatchFlag::accumulate(&b_flags[..]), expected);
/// ```
pub enum BatchFlag {
    /// Usa consistencia del tipo [SERIAL](crate::protocol::notations::consistency::Consistency::Serial) o [LOCAL_SERIAL](crate::protocol::notations::consistency::Consistency::LocalSerial).
    WithSerialConsistency,

    /// Indica que el BATCH_ trae un _timestamp_ en microsegundos.
    WithDefaultTimestamp,

    /// Esto tiene un significado similar al de las [Query Flags](crate::protocol::messages::requests::query_flags::QueryFlag::Values).
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
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::WithSerialConsistency => vec![0x0, 0x0, 0x0, 0x10],
            Self::WithDefaultTimestamp => vec![0x0, 0x0, 0x0, 0x20],
            Self::WithNamesForValues => vec![0x0, 0x0, 0x0, 0x40],
            Self::WithKeyspace => vec![0x0, 0x0, 0x0, 0x80],
            Self::WithNowInSeconds => vec![0x0, 0x0, 0x1, 0x0],
        }
    }
}

impl TryFrom<Vec<Byte>> for BatchFlag {
    type Error = Error;
    fn try_from(int: Vec<Byte>) -> Result<Self, Self::Error> {
        let bytes_array: [Byte; 4] = match int.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en BatchFlag".to_string(),
                ))
            }
        };

        let value = Int::from_be_bytes(bytes_array);
        match value {
            0x0010 => Ok(BatchFlag::WithSerialConsistency),
            0x0020 => Ok(BatchFlag::WithDefaultTimestamp),
            0x0040 => Ok(BatchFlag::WithNamesForValues),
            0x0080 => Ok(BatchFlag::WithKeyspace),
            0x0100 => Ok(BatchFlag::WithNowInSeconds),
            n if n < 0x0010 => Err(Error::ConfigError(
                "Las flags de batch deben tener los 4 bits mas a la derecha en 0".to_string(),
            )),
            _ => Err(Error::ConfigError(
                "La flag indicada para batch no existe".to_string(),
            )),
        }
    }
}

impl Maskable<Int> for BatchFlag {
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
    use super::BatchFlag;
    use crate::protocol::traits::Byteable;

    #[test]
    fn test_1_serializar() {
        let batch_flags = [
            BatchFlag::WithSerialConsistency,
            BatchFlag::WithDefaultTimestamp,
            BatchFlag::WithNamesForValues,
            BatchFlag::WithKeyspace,
            BatchFlag::WithNowInSeconds,
        ];
        let expected_bytes = [
            [0x0, 0x0, 0x0, 0x10],
            [0x0, 0x0, 0x0, 0x20],
            [0x0, 0x0, 0x0, 0x40],
            [0x0, 0x0, 0x0, 0x80],
            [0x0, 0x0, 0x1, 0x0],
        ];

        for i in 0..expected_bytes.len() {
            let serialized = batch_flags[i].as_bytes();
            assert_eq!(serialized.len(), 4);
            assert_eq!(serialized, expected_bytes[i]);
        }
    }

    #[test]
    fn test_2_deserializar() {
        let batch_res = BatchFlag::try_from([0x0, 0x0, 0x0, 0x20].to_vec());

        assert!(batch_res.is_ok());
        if let Ok(batch) = batch_res {
            assert!(matches!(batch, BatchFlag::WithDefaultTimestamp));
        }
    }

    #[test]
    fn test_3_deserializar_error() {
        let batch_res = BatchFlag::try_from([0x0, 0x0, 0x0, 0xF, 0x0, 0x0].to_vec());

        assert!(batch_res.is_err());
    }
}
