//! Módulo para el tipo de escritura de errores.

use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::cassandra::aliases::types::Byte;
use crate::cassandra::errors::error::Error;
use crate::cassandra::traits::Byteable;
use crate::cassandra::utils::{encode_string_to_bytes, parse_bytes_to_string};

/// Es un [String] que representa el tipo de escritura que se estaba intentando realizar.
pub enum WriteType {
    /// La escritura no fue de tipo batch ni de tipo counter.
    Simple,

    /// La escritura fue de tipo batch (logged). Esto signifca que el log del batch fue escrito correctamente, caso contrario, se debería haber enviado el tipo [BATCH_LOG](crate::cassandra::errors::write_type::WriteType::BatchLog).
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
    fn try_from(bytes_vec: &[Byte]) -> Result<Self, Self::Error> {
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
