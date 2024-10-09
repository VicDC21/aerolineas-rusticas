//! Módulo para las opciones de un cambio de _schema_.

use std::convert::TryFrom;

use crate::protocol::aliases::types::{Byte, Short};
use crate::protocol::errors::error::Error;
use crate::protocol::messages::responses::events::schema_changes::targets::SchemaChangeTarget;
use crate::protocol::traits::Byteable;
use crate::protocol::utils::{encode_string_to_bytes, parse_bytes_to_string};

/// Denota una opción en un evento [SCHEMA_CHANGE](crate::protocol::messages::responses::events::event_types::EventType::SchemaChange).
pub enum SchemaChangeOption {
    /// Cuando el [_target_](crate::protocol::messages::responses::events::schema_changes::targets::SchemaChangeTarget)
    /// es un [_keyspace_](crate::protocol::messages::responses::events::schema_changes::targets::SchemaChangeTarget::Keyspace).
    ///
    /// El contenido es un único [String] nombrando al _keyspace_ cambiado.
    Keyspace(String),

    /// Cuando el [_target_](crate::protocol::messages::responses::events::schema_changes::targets::SchemaChangeTarget)
    /// es [Table](crate::protocol::messages::responses::events::schema_changes::targets::SchemaChangeTarget::Table)
    /// o [Type](crate::protocol::messages::responses::events::schema_changes::targets::SchemaChangeTarget::Type).
    ///
    /// El contenido son dos [String]s denotando el _keyspace_ en donde vive el cambio y el nombre del objeto cambiado.
    TableOrType(String, String),

    /// Cuando el [_target_](crate::protocol::messages::responses::events::schema_changes::targets::SchemaChangeTarget)
    /// es [Function](crate::protocol::messages::responses::events::schema_changes::targets::SchemaChangeTarget::Function)
    /// o [Aggregate](crate::protocol::messages::responses::events::schema_changes::targets::SchemaChangeTarget::Aggregate).
    ///
    /// El contenido toma la forma de los siguientes argumentos.
    /// * Un [String] denotando el _keyspace_ donde está definida la función o _aggregate_.
    /// * Un [String] denotando el nombre de la función o _aggregate_.
    /// * Un vector de [String]s denotando los tipos de los argumentos a usar en la función/_aggregate_.
    FunctionOrAggregate(String, String, Vec<String>),
}

impl Byteable for SchemaChangeOption {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Keyspace(keyspace_name) => encode_string_to_bytes(keyspace_name),
            Self::TableOrType(keyspace_name, elem_name) => {
                let mut bytes_vec = vec![];
                bytes_vec.extend(encode_string_to_bytes(keyspace_name));
                bytes_vec.extend(encode_string_to_bytes(elem_name));
                bytes_vec
            }
            Self::FunctionOrAggregate(keyspace_name, func_name, args_types) => {
                let mut bytes_vec = vec![];
                bytes_vec.extend(encode_string_to_bytes(keyspace_name));
                bytes_vec.extend(encode_string_to_bytes(func_name));

                let args_len = args_types.len().to_le_bytes();
                bytes_vec.extend_from_slice(&[args_len[1], args_len[0]]);
                for arg in args_types {
                    bytes_vec.extend(encode_string_to_bytes(arg));
                }

                bytes_vec
            }
        }
    }
}

impl TryFrom<(&SchemaChangeTarget, &[Byte])> for SchemaChangeOption {
    type Error = Error;
    fn try_from(tupla: (&SchemaChangeTarget, &[Byte])) -> Result<Self, Self::Error> {
        let (target, bytes) = tupla;
        let mut i = 0;
        match target {
            SchemaChangeTarget::Keyspace => {
                let keyspace_name = parse_bytes_to_string(&bytes[i..], &mut i)?;
                Ok(Self::Keyspace(keyspace_name))
            }
            SchemaChangeTarget::Table | SchemaChangeTarget::Type => {
                let keyspace_name = parse_bytes_to_string(&bytes[i..], &mut i)?;
                let elem_name = parse_bytes_to_string(&bytes[i..], &mut i)?;
                Ok(Self::TableOrType(keyspace_name, elem_name))
            }
            SchemaChangeTarget::Function | SchemaChangeTarget::Aggregate => {
                let keyspace_name = parse_bytes_to_string(&bytes[i..], &mut i)?;
                let func_name = parse_bytes_to_string(&bytes[i..], &mut i)?;
                let list_len = Short::from_be_bytes([bytes[i], bytes[i + 1]]);
                i += 2;
                let mut arg_types: Vec<String> = vec![];
                for _ in 0..list_len {
                    let arg = parse_bytes_to_string(&bytes[i..], &mut i)?;
                    arg_types.push(arg);
                }
                Ok(Self::FunctionOrAggregate(
                    keyspace_name,
                    func_name,
                    arg_types,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::aliases::types::Byte;
    use crate::protocol::traits::Byteable;
    use crate::protocol::messages::responses::events::schema_changes::{targets::SchemaChangeTarget, options::SchemaChangeOption};

    #[test]
    fn test_1_serializar() {
        let keyspace = SchemaChangeOption::Keyspace("Bonito Keyspace".to_string());
        let tabl_typ = SchemaChangeOption::TableOrType("Otro Keyspace".to_string(), "valor bien feo".to_string());
        let func_agg = SchemaChangeOption::FunctionOrAggregate("keyspace de func".to_string(), "func bien fea".to_string(), vec![
            "Boolean".to_string(),
            "BigInt".to_string(),
            "Cornucopia".to_string(),
        ]);

        assert_eq!(keyspace.as_bytes(), [0x0, 0xF, 0x42, 0x6F, 0x6E, 0x69, 0x74, 0x6F, 0x20, 0x4B, 0x65, 0x79, 0x73, 0x70, 0x61, 0x63, 0x65]);
        assert_eq!(tabl_typ.as_bytes(), [0x0, 0xD, 0x4F, 0x74, 0x72, 0x6F, 0x20, 0x4B, 0x65, 0x79, 0x73, 0x70, 0x61, 0x63, 0x65,
                                         0x0, 0xE, 0x76, 0x61, 0x6C, 0x6F, 0x72, 0x20, 0x62, 0x69, 0x65, 0x6E, 0x20, 0x66, 0x65, 0x6F]);
        assert_eq!(func_agg.as_bytes(), [0x0, 0x10, 0x6B, 0x65, 0x79, 0x73, 0x70, 0x61, 0x63, 0x65, 0x20, 0x64, 0x65, 0x20, 0x66, 0x75, 0x6E, 0x63,
                                         0x0, 0xD, 0x66, 0x75, 0x6E, 0x63, 0x20, 0x62, 0x69, 0x65, 0x6E, 0x20, 0x66, 0x65, 0x61,
                                         0x0, 0x3,
                                         0x0, 0x7, 0x42, 0x6F, 0x6F, 0x6C, 0x65, 0x61, 0x6E,
                                         0x0, 0x6, 0x42, 0x69, 0x67, 0x49, 0x6E, 0x74,
                                         0x0, 0xA, 0x43, 0x6F, 0x72, 0x6E, 0x75, 0x63, 0x6F, 0x70, 0x69, 0x61]);
    }

    #[test]
    fn test_2_deserializar() {
        let func_agg_bytes: Vec<Byte> = vec![0x0, 0x10, 0x6B, 0x65, 0x79, 0x73, 0x70, 0x61, 0x63, 0x65, 0x20, 0x64, 0x65, 0x20, 0x66, 0x75, 0x6E, 0x63,
                                             0x0, 0xD, 0x66, 0x75, 0x6E, 0x63, 0x20, 0x62, 0x69, 0x65, 0x6E, 0x20, 0x66, 0x65, 0x61,
                                             0x0, 0x3,
                                             0x0, 0x7, 0x42, 0x6F, 0x6F, 0x6C, 0x65, 0x61, 0x6E,
                                             0x0, 0x6, 0x42, 0x69, 0x67, 0x49, 0x6E, 0x74,
                                             0x0, 0xA, 0x43, 0x6F, 0x72, 0x6E, 0x75, 0x63, 0x6F, 0x70, 0x69, 0x61];

        let func_agg_res = SchemaChangeOption::try_from((&SchemaChangeTarget::Function, &func_agg_bytes[..]));

        assert!(func_agg_res.is_ok());
        if let Ok(func_agg) = func_agg_res {
            assert!(matches!(func_agg, SchemaChangeOption::FunctionOrAggregate(_, _, _)));
        }
    }
}