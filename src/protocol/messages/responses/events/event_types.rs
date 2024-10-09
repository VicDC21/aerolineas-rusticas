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

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::protocol::traits::Byteable;
    use crate::protocol::errors::error::Error;
    use crate::protocol::messages::responses::events::event_types::EventType;

    #[test]
    fn test_1_serializar() {
        assert_eq!(EventType::TopologyChange("NEW_NODE".to_string(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))).as_bytes(),
                   [0x0, 0x8, 0x4E, 0x45, 0x57, 0x5F, 0x4E, 0x4F, 0x44, 0x45,
                    0x4, 0x7F, 0x0, 0x0, 0x1]);
        assert_eq!(EventType::TopologyChange("REMOVED_NODE".to_string(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2))).as_bytes(),
                   [0x0, 0xC, 0x52, 0x45, 0x4D, 0x4F, 0x56, 0x45, 0x44, 0x5F, 0x4E, 0x4F, 0x44, 0x45,
                    0x4, 0x7F, 0x0, 0x0, 0x2]);

        assert_eq!(EventType::StatusChange("UP".to_string(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3))).as_bytes(),
                   [0x0, 0x2, 0x55, 0x50,
                    0x4, 0x7F, 0x0, 0x0, 0x3]);
        assert_eq!(EventType::StatusChange("DOWN".to_string(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 4))).as_bytes(),
                   [0x0, 0x4, 0x44, 0x4F, 0x57, 0x4E,
                    0x4, 0x7F, 0x0, 0x0, 0x4]);
    }

    #[test]
    fn test_2_deserializar() {
        let new_node_res = EventType::try_from(&[0x0, 0xF, 0x54, 0x4F, 0x50, 0x4F, 0x4C, 0x4F, 0x47, 0x59, 0x5F, 0x43, 0x48, 0x41, 0x4E, 0x47, 0x45, 
                                                                           0x0, 0x8, 0x4E, 0x45, 0x57, 0x5F, 0x4E, 0x4F, 0x44, 0x45,
                                                                           0x4, 0x7F, 0x0, 0x0, 0x1][..]);

        assert!(new_node_res.is_ok());
        if let Ok(new_node) = new_node_res {
            assert!((matches!(new_node, EventType::TopologyChange(_, _))));
        }
    }

    #[test]
    fn test_3_serial_incorrecto() {
        let mal = EventType::try_from(&[0x0, 0xF, 0x69, 0x4F, 0x50, 0x4F, 0x4C, 0x4F, 0x47, 0x59, 0x5F, 0x43, 0x48, 0x41, 0x4E, 0x47, 0x45, 
                                                                  0x0, 0x8, 0x4E, 0x45, 0x57, 0x5F, 0x4E, 0x4F, 0x44, 0x45,
                                                                  0x4, 0x7F, 0x0, 0x0, 0x1][..]);

        assert!(mal.is_err());
        if let Err(err) = mal {
            assert!(matches!(err, Error::ConfigError(_)))
        }
    }
}
