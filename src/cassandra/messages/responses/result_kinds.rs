//! Módulo para los tipos de _responses_ de tipo RESULT.

use crate::cassandra::errors::error::Error;
use crate::cassandra::traits::Byteable;

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
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Void => vec![0, 0, 0, 1],
            Self::Rows => vec![0, 0, 0, 2],
            Self::SetKeyspace => vec![0, 0, 0, 3],
            Self::Prepared => vec![0, 0, 0, 4],
            Self::SchemaChange => vec![0, 0, 0, 5],
        }
    }
}

impl TryFrom<Vec<u8>> for ResultKind {
    type Error = Error;
    fn try_from(integer_in_bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let bytes_array: [u8; 4] =  match integer_in_bytes.try_into(){
            Ok(bytes_array) => bytes_array,
            Err(_e) => return Err(Error::ConfigError(
                "No se pudo castear el vector de bytes en un array en ResultKind".to_string()
            ))
        };
        let value = u32::from_be_bytes(bytes_array);
        let res = match value {
            0x01 => ResultKind::Void,
            0x02 => ResultKind::Rows,
            0x03 => ResultKind::SetKeyspace,
            0x04 => ResultKind::Prepared,
            0x05 => ResultKind::SchemaChange,
            _ => return Err(Error::Invalid(
                "El tipo de RESULT recibido no existe".to_string(),
            )),
        };
        Ok(res)
    }
}