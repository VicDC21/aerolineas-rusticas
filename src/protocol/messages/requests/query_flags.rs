//! Módulo para las flags de un _query_ en un _request_.

use crate::protocol::aliases::types::{Byte, Int};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::{Byteable, Maskable};

/// Flags específicas a mandar con un _query_.
///
/// Como flags que son, se pueden acumular en un sólo número:
/// ```rust
/// # use aerolineas_rusticas::protocol::messages::requests::query_flags::QueryFlag;
/// # use aerolineas_rusticas::protocol::traits::Maskable;
/// # use aerolineas_rusticas::protocol::aliases::types::Int;
/// let q_flags = [&QueryFlag::Values, &QueryFlag::SkipMetadata, &QueryFlag::WithKeyspace];
/// let expected: Int = 0b10000011; // 00000001 | 00000010 | 10000000 = 10000011
/// assert_eq!(QueryFlag::accumulate(&q_flags[..]), expected);
/// ```
pub enum QueryFlag {
    /// Valores son dados como variables para el _query_.
    Values,

    /// El resultado de la _response_ tendrá activada la flag [NO_METADATA](crate::protocol::messages::responses::result::rows_flags::RowsFlag::NoMetadata).
    SkipMetadata,

    /// Controla la cantidad de filas a devolver por vez.
    PageSize,

    /// La _query_ se ejecuta usando un _paging state_ dado en un _response_ anterior.
    WithPagingState,

    /// Usa consistencia del tipo [SERIAL](crate::protocol::notations::consistency::Consistency::Serial) o [LOCAL_SERIAL](crate::protocol::notations::consistency::Consistency::LocalSerial).
    WithSerialConsistency,

    /// Indica que el _query_ trae un _timestamp_ en microsegundos.
    WithDefaultTimestamp,

    /// _(Sólo tiene sentido si [VALUES](crate::protocol::messages::requests::query_flags::QueryFlag::Values) está seteado)_.
    /// Los valores son precedidos por un nombre.
    WithNamesForValues,

    /// Un _string_ indicando en qué _keyspace_ debería ejecutarse esta _query_.
    WithKeyspace,

    /// ***(Opcional)*** Indica el tiempo actual de la _query_. Diseñada para casos de _testing_.
    WithNowInSeconds,
}

impl Byteable for QueryFlag {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Values => vec![0x0, 0x0, 0x0, 0x1],
            Self::SkipMetadata => vec![0x0, 0x0, 0x0, 0x2],
            Self::PageSize => vec![0x0, 0x0, 0x0, 0x4],
            Self::WithPagingState => vec![0x0, 0x0, 0x0, 0x8],
            Self::WithSerialConsistency => vec![0x0, 0x0, 0x0, 0x10],
            Self::WithDefaultTimestamp => vec![0x0, 0x0, 0x0, 0x20],
            Self::WithNamesForValues => vec![0x0, 0x0, 0x0, 0x40],
            Self::WithKeyspace => vec![0x0, 0x0, 0x0, 0x80],
            Self::WithNowInSeconds => vec![0x0, 0x0, 0x1, 0x0],
        }
    }
}

impl TryFrom<Vec<Byte>> for QueryFlag {
    type Error = Error;
    fn try_from(int: Vec<Byte>) -> Result<Self, Self::Error> {
        let bytes_array: [Byte; 4] = match int.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en QueryFlag".to_string(),
                ))
            }
        };

        let value = Int::from_be_bytes(bytes_array);
        match value {
            0x0001 => Ok(QueryFlag::Values),
            0x0002 => Ok(QueryFlag::SkipMetadata),
            0x0004 => Ok(QueryFlag::PageSize),
            0x0008 => Ok(QueryFlag::WithPagingState),
            0x0010 => Ok(QueryFlag::WithSerialConsistency),
            0x0020 => Ok(QueryFlag::WithDefaultTimestamp),
            0x0040 => Ok(QueryFlag::WithNamesForValues),
            0x0080 => Ok(QueryFlag::WithKeyspace),
            0x0100 => Ok(QueryFlag::WithNowInSeconds),
            _ => Err(Error::ConfigError(
                "La flag indicada para query no existe".to_string(),
            )),
        }
    }
}

impl Maskable<Int> for QueryFlag {
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
    use super::QueryFlag;
    use crate::protocol::{errors::error::Error, traits::Byteable};

    #[test]
    fn test_1_serializar() {
        let query_flags = [
            QueryFlag::Values,
            QueryFlag::WithPagingState,
            QueryFlag::WithSerialConsistency,
            QueryFlag::WithDefaultTimestamp,
            QueryFlag::WithNowInSeconds,
        ];
        let expected = [
            vec![0x0, 0x0, 0x0, 0x1],
            vec![0x0, 0x0, 0x0, 0x8],
            vec![0x0, 0x0, 0x0, 0x10],
            vec![0x0, 0x0, 0x0, 0x20],
            vec![0x0, 0x0, 0x1, 0x0],
        ];
        for i in 0..expected.len() {
            assert_eq!(query_flags[i].as_bytes(), expected[i]);
        }
    }

    #[test]
    fn test_2_deserializar() {
        let query_res = QueryFlag::try_from(vec![0x0, 0x0, 0x0, 0x2]);

        assert!(query_res.is_ok());
        if let Ok(query) = query_res {
            assert!(matches!(query, QueryFlag::SkipMetadata));
        }
    }

    #[test]
    fn test_3_deserializar_error() {
        let query_res = QueryFlag::try_from(vec![0x0, 0x0, 0x0, 0x3, 0x0, 0x0]);

        assert!(query_res.is_err());
        if let Err(err) = query_res {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }
}
