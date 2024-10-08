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
                let keyspace_name = parse_bytes_to_string(bytes, &mut i)?;
                Ok(Self::Keyspace(keyspace_name))
            }
            SchemaChangeTarget::Table | SchemaChangeTarget::Type => {
                let keyspace_name = parse_bytes_to_string(bytes, &mut i)?;
                let elem_name = parse_bytes_to_string(bytes, &mut i)?;
                Ok(Self::TableOrType(keyspace_name, elem_name))
            }
            SchemaChangeTarget::Function | SchemaChangeTarget::Aggregate => {
                let keyspace_name = parse_bytes_to_string(bytes, &mut i)?;
                let func_name = parse_bytes_to_string(bytes, &mut i)?;
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
