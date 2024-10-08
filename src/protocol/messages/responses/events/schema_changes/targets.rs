//! Módulo para _targets_ de cambios de _schema_.

use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::protocol::aliases::types::Byte;
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;
use crate::protocol::utils::{encode_string_to_bytes, parse_bytes_to_string};

/// Denota un _target_ en un evento [SCHEMA_CHANGE](crate::protocol::messages::responses::events::event_types::EventType::SchemaChange).
pub enum SchemaChangeTarget {
    /// El _target_ es un __keyspace_.
    Keyspace,

    /// El _target_ es una tabla.
    Table,

    /// El _target_ está relacionado a un dato de tipo de usuario.
    Type,

    /// El _target_ es una función.
    Function,

    /// El _target_ es un _aggregate_.
    Aggregate,
}

impl Byteable for SchemaChangeTarget {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Keyspace => encode_string_to_bytes("KEYSPACE"),
            Self::Table => encode_string_to_bytes("TABLE"),
            Self::Type => encode_string_to_bytes("TYPE"),
            Self::Function => encode_string_to_bytes("FUNCTION"),
            Self::Aggregate => encode_string_to_bytes("AGGREGATE"),
        }
    }
}

impl TryFrom<&[Byte]> for SchemaChangeTarget {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self, Self::Error> {
        let string = parse_bytes_to_string(bytes, &mut 0)?;
        match string.as_str() {
            "KEYSPACE" => Ok(Self::Keyspace),
            "TABLE" => Ok(Self::Table),
            "TYPE" => Ok(Self::Type),
            "FUNCTION" => Ok(Self::Function),
            "AGGREGATE" => Ok(Self::Aggregate),
            _ => Err(Error::ConfigError(format!(
                "'{}' no parece ser un tipo válido de target de cambio de schema.",
                string
            ))),
        }
    }
}

impl Display for SchemaChangeTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Keyspace => write!(f, "KEYSPACE"),
            Self::Table => write!(f, "TABLE"),
            Self::Type => write!(f, "TYPE"),
            Self::Function => write!(f, "FUNCTION"),
            Self::Aggregate => write!(f, "AGGREGATE"),
        }
    }
}
