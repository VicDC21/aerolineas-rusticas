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
        encode_string_to_bytes(&self.to_string())
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

#[cfg(test)]
mod tests {
    use crate::protocol::traits::Byteable;
    use crate::protocol::errors::error::Error;
    use crate::protocol::messages::responses::events::schema_changes::types::SchemaChangeType;

    #[test]
    fn test_1_mostrar() {
        assert_eq!(SchemaChangeType::Created.to_string(), "CREATED".to_string());
        assert_eq!(SchemaChangeType::Updated.to_string(), "UPDATED".to_string());
        assert_eq!(SchemaChangeType::Dropped.to_string(), "DROPPED".to_string());
    }

    #[test]
    fn test_2_serializar() {
        let create = SchemaChangeType::Created;
        let update = SchemaChangeType::Updated;
        let drop = SchemaChangeType::Dropped;

        assert_eq!(create.as_bytes(), [0x0, 0x7, 0x43, 0x52, 0x45, 0x41, 0x54, 0x45, 0x44]);
        assert_eq!(update.as_bytes(), [0x0, 0x7, 0x55, 0x50, 0x44, 0x41, 0x54, 0x45, 0x44]);
        assert_eq!(drop.as_bytes(), [0x0, 0x7, 0x44, 0x52, 0x4F, 0x50, 0x50, 0x45, 0x44]);
    }

    #[test]
    fn test_3_deserializar() {
        let cr = [0x0, 0x7, 0x43, 0x52, 0x45, 0x41, 0x54, 0x45, 0x44];
        let up = [0x0, 0x7, 0x55, 0x50, 0x44, 0x41, 0x54, 0x45, 0x44];
        let dr = [0x0, 0x7, 0x44, 0x52, 0x4F, 0x50, 0x50, 0x45, 0x44];

        let cr_res = SchemaChangeType::try_from(&cr[..]);
        assert!(cr_res.is_ok());
        if let Ok(created) = cr_res {
            assert!(matches!(created, SchemaChangeType::Created));
        }

        let up_res = SchemaChangeType::try_from(&up[..]);
        assert!(up_res.is_ok());
        if let Ok(updated) = up_res {
            assert!(matches!(updated, SchemaChangeType::Updated));
        }

        let dr_res = SchemaChangeType::try_from(&dr[..]);
        assert!(dr_res.is_ok());
        if let Ok(dropped) = dr_res {
            assert!(matches!(dropped, SchemaChangeType::Dropped));
        }
    }

    #[test]
    fn test_4_serial_incorrecto() {
        let mal = [0x0, 0x3, 0x4C, 0x4F, 0x4C];

        let err_res = SchemaChangeType::try_from(&mal[..]);
        assert!(err_res.is_err());
        if let Err(err) = err_res {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }
}
