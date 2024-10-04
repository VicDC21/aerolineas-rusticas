//! Módulo para las flags de una instrucción BATCH.

use crate::cassandra::errors::error::Error;
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

impl TryFrom<Vec<u8>> for BatchFlag {
    type Error = Error;
    fn try_from(int: Vec<u8>) -> Result<Self, Self::Error> {
        let bytes_array: [u8; 4] =  match int.try_into(){
            Ok(bytes_array) => bytes_array,
            Err(_e) => return Err(Error::ConfigError(
                "No se pudo castear el vector de bytes en un array en BatchFlag".to_string()
            ))
        };
    
        let value = i32::from_be_bytes(bytes_array);
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

impl Maskable<i32> for BatchFlag {
    fn base_mask() -> i32 {
        0
    }

    fn collapse(&self) -> i32 {
        let bytes = self.as_bytes();
        i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}
