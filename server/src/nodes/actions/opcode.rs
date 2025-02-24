//! Módulo para una acción especial de los nodos.

use {
    crate::nodes::{
        node::{NodeId, NodesMap},
        states::{endpoints::EndpointState, heartbeat::HeartbeatState},
    },
    protocol::{
        aliases::{
            results::Result as SvResult,
            types::{Byte, Int},
        },
        errors::error::Error,
        traits::Byteable,
        utils::{
            encode_iter_to_bytes, encode_long_string_to_bytes, encode_string_to_bytes,
            parse_bytes_to_long_string, parse_bytes_to_string,
        },
    },
    std::{
        collections::{HashMap, HashSet},
        convert::TryFrom,
    },
};

/// Un mapa de [EndpointState]s, tal que se puedan pasar entre nodos.
pub type EndpointsVec = Vec<EndpointState>;

/// Contiene los metadatos mínimos para comparar versiones de nodos.
pub type GossipInfo = HashMap<NodeId, HeartbeatState>;

//const ACTION_MASK: Byte = 0xF0;
const ACTION_MASK: Byte = 0xE0;

/// Una "acción" de servidor es un mensaje especial que no entra en ninguna especificaión
/// del protocolo de Cassandra, y en su lugar es usado para acciones especiales fuera
/// del parseo de _queries_.
#[derive(Debug)]
pub enum SvAction {
    /// Finalizar la conexión actual.
    Exit,

    /// Aumentar en el tiempo los estados de los nodos.
    Beat,

    /// Iniciar ronda de _Gossip_.
    ///
    /// Contiene un set de [ID](crate::server::nodes::node::NodeId)s que son los vecinos con los
    /// que este nodo debe interactuar.
    Gossip(HashSet<NodeId>),

    /// Inicia el _handshake_ en un intercambio de _gossip_.
    ///
    /// Acá todavía no se mandan estados de nodo, sino sólo metadatos que son lo mínimo y necesario
    /// para comparar versiones.
    Syn(NodeId, GossipInfo),

    /// Potencial primera respuesta en un intercambio de _gossip_.
    ///
    /// Acá se devuelven tanto los estados que el nodo receptor pide actualizar en un [GossipInfo],
    /// así como los [EndpointState] actualizados que le hacen falta al nodo emisor, en un [NodesMap].
    Ack(NodeId, GossipInfo, NodesMap),

    /// Potencial segunda respuesta en un intercambio de _gossip_.
    ///
    /// A estas alturas sólo se mandan estados que el nodo receptor dijo que le hacían falta,
    /// en un [NodesMap].
    Ack2(NodesMap),

    /// Añadir un nuevo vecino.
    NewNeighbour(NodeId, EndpointState),

    /// Pedirle a este nodo que envie su endpoint state a otro nodo, dado el ID de este último.
    SendEndpointState(NodeId, String),

    /// Query enviada internamente por otro nodo.
    InternalQuery(Vec<Byte>),

    /// Guardar metadatos de un nodo en la carpeta de metadatos de los nodos `nodes_metadata`.
    StoreMetadata,

    /// Obtiene unicamente las filas de la tabla solicitada, pero con sus timestamps.
    DirectReadRequest(Vec<Byte>),

    /// Pide unicamente el valor hasheado de una query normal.
    DigestReadRequest(Vec<Byte>),

    /// Arregla la tabla mientras se realiza el read-repair.
    ///
    /// _(table_name, node_id, rows)_
    RepairRows(String, Byte, Vec<Byte>),

    /// Agrega una relacion de tabla con un partition value
    AddPartitionValueToMetadata(String, String),

    /// Pide los metadatos del nodo receptor para el nodo del ID dado, específicamente
    /// `keyspaces`, `tables`, `tables_and_partitions_keys_values` y `default_keyspace_name`.
    SendMetadata(NodeId),

    /// Recibe la metadata `keyspaces`, `tables`, `tables_and_partitions_keys_values` y `default_keyspace_name`,
    /// para luego actualizar los atributos propios.
    ReceiveMetadata(Vec<Byte>),

    /// Aviso de que se reacomodará el clúster, para no seguir realizando operaciones cliente-servidor.
    RelocationNeeded,

    /// Actualiza las réplicas para adaptarse al nodo nuevo o borrado.
    UpdateReplicas(NodeId, bool),

    /// Agrega las filas dadas al nodo receptor, que fueron relocalizadas.
    AddRelocatedRows(NodeId, String),

    /// Pide todas las filas de todas las tablas al nodo receptor.
    GetAllTablesOfReplica(NodeId, bool),

    /// Aviso al nodo receptor que debe ser dado de baja del clúster.
    DeleteNode,

    /// Avisa que el ID del nodo dado ya fue dado de baja.
    NodeIsLeaving(NodeId),

    /// Le avisa a los demas nodos que ya se fue del cluster y que es seguro borrarlo.
    NodeDeleted(NodeId),

    /// Le reenvia el mensaje al nodo correspondiente que tenga que ser borrado.
    NodeToDelete(NodeId),
}

impl SvAction {
    /// Consulta si el conjunto de bytes dados empieza por el prefijo relevante.
    ///
    /// Esto es, si al menos los 3 bits más significativos son todos `1`.
    pub fn is_action(bytes: &[Byte]) -> bool {
        if bytes.is_empty() {
            return false;
        }

        let opcode = bytes[0];
        (opcode & ACTION_MASK) == ACTION_MASK
    }

    /// Potencialmente consigue una acción, ignorando un error de ser el caso.
    pub fn get_action(bytes: &[Byte]) -> Option<Self> {
        match Self::try_from(bytes) {
            Ok(action) => Some(action),
            Err(_) => None,
        }
    }

    /// Serializa la información de _gossip_.
    fn encode_gossip_info_to_bytes(gossip_info: &GossipInfo) -> Vec<Byte> {
        let mut bytes_vec: Vec<Byte> = Vec::new();
        let gossip_len_bytes = gossip_info.len().to_le_bytes();
        bytes_vec.extend_from_slice(&[
            gossip_len_bytes[3],
            gossip_len_bytes[2],
            gossip_len_bytes[1],
            gossip_len_bytes[0],
        ]);

        for (node_id, node_heartbeat) in gossip_info {
            bytes_vec.push(node_id.to_owned());
            bytes_vec.extend(node_heartbeat.as_bytes());
        }

        bytes_vec
    }

    /// Serializa un mapa de nodos.
    fn encode_nodes_map_to_bytes(nodes_map: &NodesMap) -> Vec<Byte> {
        let mut bytes_vec: Vec<Byte> = Vec::new();
        let nodes_len_bytes = nodes_map.len().to_le_bytes();
        bytes_vec.extend_from_slice(&[
            nodes_len_bytes[3],
            nodes_len_bytes[2],
            nodes_len_bytes[1],
            nodes_len_bytes[0],
        ]);

        for (node_id, endpoint_state) in nodes_map {
            bytes_vec.push(node_id.to_owned());
            bytes_vec.extend(endpoint_state.as_bytes());
        }

        bytes_vec
    }

    /// Deserializa una secuencia de [Byte]s de vuelta a un [GossipInfo].
    pub fn parse_bytes_to_gossip_info(bytes: &[Byte], i: &mut usize) -> SvResult<GossipInfo> {
        let mut j: usize = 0;
        let mut gossip_info = GossipInfo::new();

        let gossip_len = Int::from_be_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
        j += 4;

        for _ in 0..gossip_len {
            let node_id = bytes[j]; // El nodo siempre tendrá un ID de un byte.
            let heartbeat = HeartbeatState::try_from(&bytes[j + 1..])?;
            j += heartbeat.as_bytes().len() + 1;
            gossip_info.insert(node_id, heartbeat);
        }

        *i += j;
        Ok(gossip_info)
    }

    /// Deserializa una secuencia de [Byte]s de vuelta a un [NodesMap].
    pub fn parse_bytes_to_nodes_map(bytes: &[Byte], i: &mut usize) -> SvResult<NodesMap> {
        let mut j: usize = 0;
        let mut nodes_map = NodesMap::new();

        let nodes_len = Int::from_be_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
        j += 4;

        for _ in 0..nodes_len {
            let node_id = bytes[j]; // El nodo siempre tendrá un ID de un byte.
            let endpoint_state = EndpointState::try_from(&bytes[j + 1..])?;
            j += endpoint_state.as_bytes().len() + 1;
            nodes_map.insert(node_id, endpoint_state);
        }

        *i += j;
        Ok(nodes_map)
    }
}

impl Byteable for SvAction {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::Exit => vec![0xF0],
            Self::Beat => vec![0xF1],
            Self::Gossip(neighbours) => {
                let mut bytes_vec = vec![0xF2];
                let neighbours_iter = neighbours.iter().map(|byte| vec![byte.to_owned()]);
                bytes_vec.extend(encode_iter_to_bytes(neighbours_iter));
                bytes_vec
            }
            Self::Syn(emissor_id, gossip_info) => {
                let mut bytes_vec = vec![0xF3, *emissor_id];
                bytes_vec.extend(Self::encode_gossip_info_to_bytes(gossip_info));
                bytes_vec
            }
            Self::Ack(receptor_id, gossip_info, nodes_map) => {
                let mut bytes_vec = vec![0xF4, *receptor_id];
                bytes_vec.extend(Self::encode_gossip_info_to_bytes(gossip_info));
                bytes_vec.extend(Self::encode_nodes_map_to_bytes(nodes_map));
                bytes_vec
            }
            Self::Ack2(nodes_map) => {
                let mut bytes_vec = vec![0xF5];
                bytes_vec.extend(Self::encode_nodes_map_to_bytes(nodes_map));
                bytes_vec
            }
            Self::NewNeighbour(id, state) => {
                let mut bytes = vec![0xF6, *id];
                bytes.extend(state.as_bytes());
                bytes
            }
            Self::SendEndpointState(id, string) => {
                let str_as_bytes = encode_string_to_bytes(string);
                let mut bytes = vec![0xF7, *id];
                bytes.extend(str_as_bytes);
                bytes
            }
            Self::InternalQuery(query_bytes) => {
                let mut bytes = vec![0xF8];
                bytes.extend(query_bytes);
                bytes
            }
            Self::StoreMetadata => vec![0xF9],
            Self::DirectReadRequest(query_bytes) => {
                let mut bytes = vec![0xFA];
                bytes.extend(query_bytes);
                bytes
            }
            Self::DigestReadRequest(query_bytes) => {
                let mut bytes = vec![0xFB];
                bytes.extend(query_bytes);
                bytes
            }
            Self::RepairRows(table_name, node_id, rows) => {
                let mut bytes = vec![0xFC];
                bytes.extend(encode_string_to_bytes(table_name));
                bytes.push(*node_id);
                bytes.extend(rows);
                bytes
            }
            Self::AddPartitionValueToMetadata(table_name, partition_value) => {
                let mut bytes = vec![0xFD];
                bytes.extend(encode_string_to_bytes(table_name));
                bytes.extend(encode_string_to_bytes(partition_value));
                bytes
            }
            Self::SendMetadata(node_id) => vec![0xFE, *node_id],
            Self::ReceiveMetadata(metadata) => {
                let mut bytes = vec![0xFF];
                bytes.extend(metadata);
                bytes
            }
            Self::RelocationNeeded => vec![0xE0],
            Self::UpdateReplicas(new_node_id, is_deletion) => {
                let bool_to_byte = if *is_deletion { 1 } else { 0 };
                vec![0xE1, *new_node_id, bool_to_byte]
            }
            Self::AddRelocatedRows(node_id, rows) => {
                let mut bytes = vec![0xE2, *node_id];
                bytes.extend(encode_long_string_to_bytes(rows));
                bytes
            }
            Self::GetAllTablesOfReplica(node_id, only_farthest_replica) => {
                let bool_to_byte = if *only_farthest_replica { 1 } else { 0 };
                vec![0xE3, *node_id, bool_to_byte]
            }
            Self::DeleteNode => vec![0xE4],
            Self::NodeIsLeaving(node_id) => vec![0xE5, *node_id],
            Self::NodeDeleted(node_id) => vec![0xE6, *node_id],
            Self::NodeToDelete(node_id) => vec![0xE7, *node_id],
        }
    }
}

impl TryFrom<&[Byte]> for SvAction {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            return Err(Error::ServerError(
                "Conjunto de bytes demasiado chico.".to_string(),
            ));
        }

        let mut i = 0;
        let first = bytes[i];
        if !Self::is_action(bytes) {
            return Err(Error::ServerError(format!(
                "Conjunto de bytes no empieza por `111...`. En su lugar se recibió {:#b}",
                first
            )));
        }

        match first {
            0xF0 => Ok(Self::Exit),
            0xF1 => Ok(Self::Beat),
            0xF2 => {
                i += 1;
                let mut set_bytes: HashSet<Byte> = HashSet::new();
                let set_len =
                    Int::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                i += 4;
                for _ in 0..set_len {
                    let byte_len =
                        Int::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                    i += 4;
                    if byte_len != 1 {
                        return Err(Error::ServerError(format!(
                            "Se esperaba que el valor del set ocupara sólo un byte, no {}",
                            byte_len
                        )));
                    }

                    set_bytes.insert(bytes[i]);
                    i += 1;
                }
                Ok(Self::Gossip(set_bytes))
            }
            0xF3 => {
                i += 1;
                let emissor_id = bytes[i];
                i += 1;
                Ok(Self::Syn(
                    emissor_id,
                    Self::parse_bytes_to_gossip_info(&bytes[i..], &mut i)?,
                ))
            }
            0xF4 => {
                i += 1;
                let receptor_id = bytes[i];
                i += 1;
                let gossip_info = Self::parse_bytes_to_gossip_info(&bytes[i..], &mut i)?;
                let nodes_map = Self::parse_bytes_to_nodes_map(&bytes[i..], &mut i)?;
                Ok(Self::Ack(receptor_id, gossip_info, nodes_map))
            }
            0xF5 => {
                i += 1;
                Ok(Self::Ack2(Self::parse_bytes_to_nodes_map(
                    &bytes[i..],
                    &mut i,
                )?))
            }
            0xF6 => {
                if bytes.len() < 3 {
                    return Err(Error::ServerError(
                        "Conjunto de bytes demasiado chico para `NewNeighbour`.".to_string(),
                    ));
                }
                i += 1;
                let id = bytes[i];
                i += 1;
                let state = EndpointState::try_from(&bytes[i..])?;
                Ok(Self::NewNeighbour(id, state))
            }
            0xF7 => {
                if bytes.len() < 2 {
                    return Err(Error::ServerError(
                        "Conjunto de bytes demasiado chico para `SendEndpointState`.".to_string(),
                    ));
                }
                let string_ip = parse_bytes_to_string(&bytes[2..], &mut i)?;
                Ok(Self::SendEndpointState(bytes[1], string_ip))
            }
            0xF8 => Ok(Self::InternalQuery(bytes[1..].to_vec())),
            0xF9 => Ok(Self::StoreMetadata),
            0xFA => Ok(Self::DirectReadRequest(bytes[1..].to_vec())),
            0xFB => Ok(Self::DigestReadRequest(bytes[1..].to_vec())),
            0xFC => {
                let table_name = parse_bytes_to_string(&bytes[1..], &mut i)?;
                let node_id = bytes[i + 1];
                let rows = bytes[table_name.len() + 1 + 2 + 1..].to_vec();
                Ok(Self::RepairRows(table_name, node_id, rows))
            }
            0xFD => {
                let table_name = parse_bytes_to_string(&bytes[1..], &mut i)?;
                let partition_value = parse_bytes_to_string(&bytes[(i + 1)..], &mut i)?;
                Ok(Self::AddPartitionValueToMetadata(
                    table_name,
                    partition_value,
                ))
            }
            0xFE => Ok(Self::SendMetadata(bytes[1])),
            0xFF => Ok(Self::ReceiveMetadata(bytes[1..].to_vec())),
            0xE0 => Ok(Self::RelocationNeeded),
            0xE1 => {
                let mut is_deletion = true;
                if bytes[2] == 0 {
                    is_deletion = false;
                }
                Ok(Self::UpdateReplicas(bytes[1], is_deletion))
            }
            0xE2 => {
                let node_id = bytes[1];
                let rows = parse_bytes_to_long_string(&bytes[2..], &mut i)?;
                Ok(Self::AddRelocatedRows(node_id, rows))
            }
            0xE3 => {
                let mut only_farthest_replica = true;
                if bytes[2] == 0 {
                    only_farthest_replica = false;
                }
                Ok(Self::GetAllTablesOfReplica(bytes[1], only_farthest_replica))
            }
            0xE4 => Ok(Self::DeleteNode),
            0xE5 => Ok(Self::NodeIsLeaving(bytes[1])),
            0xE6 => Ok(Self::NodeDeleted(bytes[1])),
            0xE7 => Ok(Self::NodeToDelete(bytes[1])),
            _ => Err(Error::ServerError(format!(
                "'{:#b}' no es un id de acción válida.",
                first
            ))),
        }
    }
}

impl std::fmt::Display for SvAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exit => write!(f, "Exit"),
            Self::Beat => write!(f, "Beat"),
            Self::Gossip(neighbours) => write!(f, "Gossip({:?})", neighbours),
            Self::Syn(emissor_id, gossip_info) => {
                write!(f, "Syn({}, {:?})", emissor_id, gossip_info)
            }
            Self::Ack(receptor_id, gossip_info, nodes_map) => {
                write!(
                    f,
                    "Ack({}, {:?}, {:?})",
                    receptor_id, gossip_info, nodes_map
                )
            }
            Self::Ack2(nodes_map) => write!(f, "Ack2({:?})", nodes_map),
            Self::NewNeighbour(id, state) => write!(f, "NewNeighbour({}, {:?})", id, state),
            Self::SendEndpointState(id) => write!(f, "SendEndpointState({})", id),
            Self::InternalQuery(query_bytes) => write!(f, "InternalQuery({:?})", query_bytes),
            Self::StoreMetadata => write!(f, "StoreMetadata"),
            Self::DirectReadRequest(query_bytes) => {
                write!(f, "DirectReadRequest({:?})", query_bytes)
            }
            Self::DigestReadRequest(query_bytes) => {
                write!(f, "DigestReadRequest({:?})", query_bytes)
            }
            Self::RepairRows(table_name, node_id, rows) => {
                write!(f, "RepairRows({}, {}, {:?})", table_name, node_id, rows)
            }
            Self::AddPartitionValueToMetadata(table_name, partition_value) => {
                write!(
                    f,
                    "AddPartitionValueToMetadata({}, {})",
                    table_name, partition_value
                )
            }
            Self::SendMetadata(node_id) => write!(f, "SendMetadata({})", node_id),
            Self::ReceiveMetadata(metadata) => write!(f, "ReceiveMetadata({:?})", metadata),
            Self::ReallocationNeeded => write!(f, "ReallocationNeeded"),
            Self::UpdateReplicas(new_node_id) => write!(f, "UpdateReplicas({})", new_node_id),
        }
    }
}
