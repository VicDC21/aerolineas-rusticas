//! Módulo para _targets_ de cambios de _schema_.

use {
    crate::protocol::{
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
        encode_string_to_bytes(&self.to_string())
    }
}

impl TryFrom<&[Byte]> for SchemaChangeTarget {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_mostrar() {
        assert_eq!(
            SchemaChangeTarget::Keyspace.to_string(),
            "KEYSPACE".to_string()
        );
        assert_eq!(SchemaChangeTarget::Table.to_string(), "TABLE".to_string());
        assert_eq!(SchemaChangeTarget::Type.to_string(), "TYPE".to_string());
        assert_eq!(
            SchemaChangeTarget::Function.to_string(),
            "FUNCTION".to_string()
        );
        assert_eq!(
            SchemaChangeTarget::Aggregate.to_string(),
            "AGGREGATE".to_string()
        );
    }

    #[test]
    fn test_2_serializar() {
        let targets = [
            SchemaChangeTarget::Keyspace,
            SchemaChangeTarget::Table,
            SchemaChangeTarget::Type,
            SchemaChangeTarget::Function,
            SchemaChangeTarget::Aggregate,
        ];
        let target_bytes = [
            vec![0x0, 0x8, 0x4B, 0x45, 0x59, 0x53, 0x50, 0x41, 0x43, 0x45],
            vec![0x0, 0x5, 0x54, 0x41, 0x42, 0x4C, 0x45],
            vec![0x0, 0x4, 0x54, 0x59, 0x50, 0x45],
            vec![0x0, 0x8, 0x46, 0x55, 0x4E, 0x43, 0x54, 0x49, 0x4F, 0x4E],
            vec![
                0x0, 0x9, 0x41, 0x47, 0x47, 0x52, 0x45, 0x47, 0x41, 0x54, 0x45,
            ],
        ];

        for i in 0..targets.len() {
            let bytes = targets[i].as_bytes();

            assert_eq!(bytes.len(), targets[i].to_string().len() + 2);
            assert_eq!(bytes, target_bytes[i]);
        }
    }

    #[test]
    fn test_3_deserializar() {
        let target_res = SchemaChangeTarget::try_from(
            &[
                0x0, 0x9, 0x41, 0x47, 0x47, 0x52, 0x45, 0x47, 0x41, 0x54, 0x45,
            ][..],
        );

        assert!(target_res.is_ok());
        if let Ok(target) = target_res {
            assert!(matches!(target, SchemaChangeTarget::Aggregate));
        }
    }

    #[test]
    fn test_4_serial_incorrecto() {
        let dont_decode = SchemaChangeTarget::try_from(
            &[
                0x0, 0x17, 0x4E, 0x65, 0x76, 0x65, 0x72, 0x20, 0x67, 0x6F, 0x6E, 0x6E, 0x61, 0x20,
                0x67, 0x69, 0x76, 0x65, 0x20, 0x79, 0x6F, 0x75, 0x20, 0x75, 0x70,
            ][..],
        );

        assert!(dont_decode.is_err());
        if let Err(err) = dont_decode {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }
}
