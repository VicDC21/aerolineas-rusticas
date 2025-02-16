//! Módulo para el tipo de escritura de errores.

use {
    crate::{
        aliases::{results::Result, types::Byte},
        errors::error::Error,
        traits::Byteable,
        utils::{encode_string_to_bytes, parse_bytes_to_string},
    },
    std::{
        convert::TryFrom,
        fmt::{Display, Formatter, Result as FmtResult},
    },
};

/// Es un [String] que representa el tipo de escritura que se estaba intentando realizar.

#[derive(Clone, Debug)]
pub enum WriteType {
    /// La escritura no fue de tipo batch ni de tipo counter.
    Simple,

    /// La escritura fue de tipo batch (logged). Esto signifca que el log del batch fue escrito correctamente, caso contrario, se debería haber enviado el tipo [BATCH_LOG](crate::errors::write_type::WriteType::BatchLog).
    Batch,

    /// La escritura fue de tipo batch (unlogged). No hubo intento de escritura en el log del batch.
    UnloggedBatch,

    /// La escritura fue de tipo counter (batch o no).
    Counter,

    /// El timeout ocurrió durante la escritura en el log del batch cuando una escritura de batch (logged) fue pedida.
    BatchLog,

    /// El timeout ocurrió durante el "Compare And Set write/update" (escritura/actualización).
    Cas,

    /// El timeout ocurrió durante una escritura que involucra una actualización de VIEW (vista) y falló en adquirir el lock de vista local (MV) para la clave dentro del timeout.
    View,

    /// El timeout ocurrió cuando la cantidad total de espacio en disco (en MB) que se puede utilizar para almacenar los logs de CDC (Change Data Capture) fue excedida cuando se intentaba escribir en dicho logs.
    Cdc,
}

impl Display for WriteType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Simple => write!(f, "SIMPLE"),
            Self::Batch => write!(f, "BATCH"),
            Self::UnloggedBatch => write!(f, "UNLOGGED_BATCH"),
            Self::Counter => write!(f, "COUNTER"),
            Self::BatchLog => write!(f, "BATCH_LOG"),
            Self::Cas => write!(f, "CAS"),
            Self::View => write!(f, "VIEW"),
            Self::Cdc => write!(f, "CDC"),
        }
    }
}

impl Byteable for WriteType {
    fn as_bytes(&self) -> Vec<Byte> {
        encode_string_to_bytes(&self.to_string())
    }
}

impl TryFrom<&[Byte]> for WriteType {
    type Error = Error;
    fn try_from(bytes_vec: &[Byte]) -> Result<Self> {
        let inner_str = parse_bytes_to_string(bytes_vec, &mut 0)?;
        match inner_str.as_str() {
            "SIMPLE" => Ok(Self::Simple),
            "BATCH" => Ok(Self::Batch),
            "UNLOGGED_BATCH" => Ok(Self::UnloggedBatch),
            "COUNTER" => Ok(Self::Counter),
            "BATCH_LOG" => Ok(Self::BatchLog),
            "CAS" => Ok(Self::Cas),
            "VIEW" => Ok(Self::View),
            "CDC" => Ok(Self::Cdc),
            _ => Err(Error::ConfigError(format!(
                "'{}' no corresponde a ninguna variante.",
                inner_str
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_serializar() {
        let write_types = [
            WriteType::Simple,
            WriteType::UnloggedBatch,
            WriteType::Counter,
            WriteType::Cdc,
        ];

        let expected = [
            vec![0x0, 0x6, 0x53, 0x49, 0x4D, 0x50, 0x4C, 0x45],
            vec![
                0x0, 0xE, 0x55, 0x4E, 0x4C, 0x4F, 0x47, 0x47, 0x45, 0x44, 0x5F, 0x42, 0x41, 0x54,
                0x43, 0x48,
            ],
            vec![0x0, 0x7, 0x43, 0x4F, 0x55, 0x4E, 0x54, 0x45, 0x52],
            vec![0x0, 0x3, 0x43, 0x44, 0x43],
        ];

        for i in 0..expected.len() {
            assert_eq!(write_types[i].as_bytes(), expected[i]);
        }
    }

    #[test]
    fn test_2_deserializar() {
        let write_res = WriteType::try_from(&[0x0, 0x6, 0x53, 0x49, 0x4D, 0x50, 0x4C, 0x45][..]);

        assert!(write_res.is_ok());
        if let Ok(write) = write_res {
            assert!(matches!(write, WriteType::Simple));
        }
    }

    #[test]
    fn test_3_deserializar_error() {
        let write_res = WriteType::try_from(&[0x0, 0x0, 0x0, 0x0][..]);

        assert!(write_res.is_err());
        if let Err(err) = write_res {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }
}
