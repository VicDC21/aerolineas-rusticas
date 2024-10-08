//! Módulo para el tipo de un cambio de _schema_.

use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::protocol::aliases::types::Byte;
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;
use crate::protocol::utils::{encode_string_to_bytes, parse_bytes_to_string};

/// Denota un tipo de cambio en el evento [SCHEMA_CHANGE](crate::protocol::messages::responses::events::event_types::EventType::SchemaChange).
pub enum SchemaChangeType {
    /// Denota la creación de un _schema_.
    Created,

    /// Denota la edición de un _schema_.
    Updated,

    /// Denota el borrado de una _schema_.
    Dropped,
}

impl Byteable for SchemaChangeType {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Created => encode_string_to_bytes("CREATED"),
            Self::Updated => encode_string_to_bytes("UPDATED"),
            Self::Dropped => encode_string_to_bytes("DROPPED"),
        }
    }
}

impl TryFrom<&[Byte]> for SchemaChangeType {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self, Self::Error> {
        let string = parse_bytes_to_string(bytes, &mut 0)?;
        match string.as_str() {
            "CREATED" => Ok(Self::Created),
            "UPDATED" => Ok(Self::Updated),
            "DROPPED" => Ok(Self::Dropped),
            _ => Err(Error::ConfigError(format!(
                "'{}' no es un valor válido para un tipo de cambio de schema.",
                string
            ))),
        }
    }
}

impl Display for SchemaChangeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Created => write!(f, "CREATED"),
            Self::Updated => write!(f, "UPDATED"),
            Self::Dropped => write!(f, "DROPPED"),
        }
    }
}
