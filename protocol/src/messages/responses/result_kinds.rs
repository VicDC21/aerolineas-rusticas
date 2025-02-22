//! Módulo para los tipos de _responses_ de tipo RESULT.

use crate::{
    aliases::{
        results::Result,
        types::{Byte, Int},
    },
    errors::error::Error,
    traits::Byteable,
};

/// Tipos de resultados de una _query_ (mensajes QUERY, PREPARE, EXECUTE o BATCH).
pub enum ResultKind {
    /// El resultado no contiene información adicional en el cuerpo.
    Void,

    /// Resultado de SELECT, que devuelve las filas pedidas.
    Rows,

    /// El resultado de una _query_ `use`.
    SetKeyspace,

    /// El resultado de una _query_ de tipo PREPARE.
    Prepared,

    /// El resultado de una _query_ que altera un _schema_.
    SchemaChange,
}

impl Byteable for ResultKind {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Void => vec![0x0, 0x0, 0x0, 0x1],
            Self::Rows => vec![0x0, 0x0, 0x0, 0x2],
            Self::SetKeyspace => vec![0x0, 0x0, 0x0, 0x3],
            Self::Prepared => vec![0x0, 0x0, 0x0, 0x4],
            Self::SchemaChange => vec![0x0, 0x0, 0x0, 0x5],
        }
    }
}

impl TryFrom<Vec<Byte>> for ResultKind {
    type Error = Error;
    fn try_from(integer_in_bytes: Vec<Byte>) -> Result<Self> {
        let bytes_array: [Byte; 4] = match integer_in_bytes.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en ResultKind".to_string(),
                ))
            }
        };
        let value = Int::from_be_bytes(bytes_array);
        let res = match value {
            0x01 => ResultKind::Void,
            0x02 => ResultKind::Rows,
            0x03 => ResultKind::SetKeyspace,
            0x04 => ResultKind::Prepared,
            0x05 => ResultKind::SchemaChange,
            _ => {
                return Err(Error::Invalid(
                    "El tipo de RESULT recibido no existe".to_string(),
                ))
            }
        };
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_serializar() {
        let result_kinds = [
            ResultKind::Void,
            ResultKind::Rows,
            ResultKind::SetKeyspace,
            ResultKind::Prepared,
            ResultKind::SchemaChange,
        ];
        let expected_bytes = [
            vec![0x0, 0x0, 0x0, 0x1],
            vec![0x0, 0x0, 0x0, 0x2],
            vec![0x0, 0x0, 0x0, 0x3],
            vec![0x0, 0x0, 0x0, 0x4],
            vec![0x0, 0x0, 0x0, 0x5],
        ];

        for i in 0..expected_bytes.len() {
            let serialized = result_kinds[i].as_bytes();
            assert_eq!(serialized.len(), 4);
            assert_eq!(serialized, expected_bytes[i]);
        }
    }

    #[test]
    fn test_2_deserializar() {
        let keyspace_res = ResultKind::try_from(vec![0x0, 0x0, 0x0, 0x3]);

        assert!(keyspace_res.is_ok());
        if let Ok(void) = keyspace_res {
            assert!(matches!(void, ResultKind::SetKeyspace));
        }
    }

    #[test]
    fn test_3_deserializar_error() {
        let err_res = ResultKind::try_from(vec![0x0, 0x0, 0x0, 0x6]);

        assert!(err_res.is_err());
        if let Err(err) = err_res {
            assert!(matches!(err, Error::Invalid(_)));
        }
    }
}
