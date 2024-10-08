//! Módulo para tipos de eventos.

use std::net::IpAddr;

use std::convert::TryFrom;

use crate::protocol::aliases::types::Byte;
use crate::protocol::errors::error::Error;
use crate::protocol::messages::responses::events::schema_changes::{
    options::SchemaChangeOption, targets::SchemaChangeTarget, types::SchemaChangeType,
};
use crate::protocol::traits::Byteable;
use crate::protocol::utils::{
    encode_ipaddr_to_bytes, encode_string_to_bytes, parse_bytes_to_ipaddr, parse_bytes_to_string,
};

// use crate::protocol::traits::Byteable;

/// Tipos de mensaje [EVENT](crate::protocol::headers::opcode::Opcode::Event).
///
/// Un cliente sólo escuchará eventos a los que se ha [registrado](crate::protocol::headers::opcode::Opcode::Register).
pub enum EventType {
    /// Cambios relacionados a la topología del clúster de nodos.
    /// Por ejemplo, cuando un nodo es agregado o removido.
    TopologyChange(String, IpAddr),

    /// Refiere al cambio de estado de un nodo:
    /// * `"UP"` cuando un nodo está disponible.
    /// * `"DOWN"` cuando un nodo deja de estarlo.
    StatusChange(String, IpAddr),

    /// Relacionado a cambios de _schemas_.
    ///
    /// El resto del mensaje tendrá el formato `<change_type><target><options>`, donde:
    /// * `<change_type>` es un [String] indicando el tipo de cambio (`"CREATED"`, `"UPDATED"` o `"DROPPED"`).
    /// * `<target>` es un [String] que describe lo que se modificó. Puede tomar los valores
    ///   `"KEYSPACE"`, `"TABLE"`, `"TYPE"`, `"FUNCTION"` o `"AGGREGATE"`.
    /// * `<options>` depende del valor de `<target>`:
    ///     - Si `<target>` tiene valor `"KEYSPACE"`, entonces `<options>` es un único [String]
    ///       nombrando al _keyspace_ cambiado.
    ///     - Si `<target>` tiene valores `"TABLE"` o `"TYPE"`, entonces `<options>` serán dos [String]s
    ///       denotando el _keyspace_ en donde vive el cambio y el nombre del objeto cambiado.
    ///     - Si `<target>` tiene valores `"FUNCTION"` o `"AGGREGATE"`, `<options>` toma la forma
    ///       de los siguientes argumentos.
    ///         * Un [String] denotando el _keyspace_ donde está definida la función o _aggregate_.
    ///         * Un [String] denotando el nombre de la función o _aggregate_.
    ///         * Un vector de [String]s denotando los tipos de los argumentos a usar en la función/_aggregate_.
    SchemaChange(SchemaChangeType, SchemaChangeTarget, SchemaChangeOption),
}

impl Byteable for EventType {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes_vec = vec![];
        match self {
            Self::TopologyChange(change_type, ipaddr) => {
                bytes_vec.extend(encode_string_to_bytes(change_type));
                bytes_vec.extend(encode_ipaddr_to_bytes(ipaddr));
            }
            Self::StatusChange(change_type, ipaddr) => {
                bytes_vec.extend(encode_string_to_bytes(change_type));
                bytes_vec.extend(encode_ipaddr_to_bytes(ipaddr));
            }
            Self::SchemaChange(schema_type, schema_target, schema_option) => {
                bytes_vec.extend(schema_type.as_bytes());
                bytes_vec.extend(schema_target.as_bytes());
                bytes_vec.extend(schema_option.as_bytes());
            }
        }
        bytes_vec
    }
}

impl TryFrom<&[Byte]> for EventType {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self, Self::Error> {
        let mut i = 0;
        let event_type = parse_bytes_to_string(&bytes[i..], &mut i)?;

        match event_type.as_str() {
            "TOPOLOGY_CHANGE" => {
                let change_type = parse_bytes_to_string(&bytes[i..], &mut i)?;
                match change_type.as_str() {
                    "NEW_NODE" | "REMOVED_NODE" => {
                        let ipaddr = parse_bytes_to_ipaddr(&bytes[i..], &mut i)?;
                        Ok(Self::TopologyChange(change_type, ipaddr))
                    }
                    _ => Err(Error::ConfigError(format!(
                        "'{}' no es un tipo de cambio válido para {}.",
                        change_type, event_type
                    ))),
                }
            }
            "STATUS_CHANGE" => {
                let change_type = parse_bytes_to_string(&bytes[i..], &mut i)?;
                match change_type.as_str() {
                    "UP" | "DOWN" => {
                        let ipaddr = parse_bytes_to_ipaddr(&bytes[i..], &mut i)?;
                        Ok(Self::StatusChange(change_type, ipaddr))
                    }
                    _ => Err(Error::ConfigError(format!(
                        "'{}' no es un tipo de cambio válido para {}.",
                        change_type, event_type
                    ))),
                }
            }
            "SCHEMA_CHANGE" => {
                let schema_type = SchemaChangeType::try_from(&bytes[i..])?;
                i += schema_type.to_string().len() + 2;

                let schema_target = SchemaChangeTarget::try_from(&bytes[i..])?;
                i += schema_target.to_string().len() + 2;

                let schema_option = SchemaChangeOption::try_from((&schema_target, &bytes[i..]))?;

                Ok(Self::SchemaChange(
                    schema_type,
                    schema_target,
                    schema_option,
                ))
            }
            _ => Err(Error::ConfigError(format!(
                "'{}' no parece ser un tipo de evento válido.",
                event_type
            ))),
        }
    }
}
